// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future::{self, Future};
use headers::{ETag, IfNoneMatch, Origin};
use httparse;
use hyper::{http, Body, Error, HeaderMap, Method, StatusCode, Uri};
use hyper::{Request, Response};

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnel;
use crate::cache::read::CacheRead;
use crate::cache::route::CacheRoute;
use crate::cache::write::CacheWrite;
use crate::header::janitor::HeaderJanitor;
use crate::header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use crate::LINE_FEED;

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

type ProxyServeResult = Result<(String, Option<String>), ()>;

type ProxyServeResultFuture = dyn Future<Output = Response<Vec<u8>>>;

pub type ProxyServeResponseFuture = dyn Future<Output = Response<Vec<u8>>>;

impl ProxyServe {
    pub fn handle(req: Request<Vec<u8>>) -> ProxyServeResponseFuture {
        info!("handled request: {} on {}", req.method(), req.path());

        match *req.method() {
            Method::Options
            | Method::Head
            | Method::Get
            | Method::Post
            | Method::Patch
            | Method::Put
            | Method::Delete => Self::accept(req),
            _ => Self::reject(req, StatusCode::MethodNotAllowed),
        }
    }

    fn accept(req: Request<Vec<u8>>) -> ProxyServeResponseFuture {
        Self::tunnel(req)
    }

    fn reject(req: Request<Vec<u8>>, status: StatusCode) -> ProxyServeResponseFuture {
        let mut headers = HeaderMap::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Reject));

        Self::respond(req.method(), status, headers, format!("{status}"))
    }

    fn tunnel(req: Request<Vec<u8>>) -> ProxyServeResponseFuture {
        let (method, uri, version, headers, body) = req.deconstruct();
        let (headers, auth, shard) = ProxyHeader::parse_from_request(headers);

        let auth_hash = CacheRoute::hash(&auth);

        let (ns, ns_mask) = CacheRoute::gen_key_cache(
            shard,
            &auth_hash,
            version,
            &method,
            uri.path(),
            uri.query(),
            headers.get::<Origin>(),
        );

        info!("tunneling for ns = {}", ns);

        Box::new(
            Self::fetch_cached_data(shard, &ns, &method, &headers)
                .or_else(|_| Err(Error::Incomplete))
                .and_then(move |result| match result {
                    Ok(value) => Self::dispatch_cached(
                        shard, ns, ns_mask, auth_hash, method, &uri, version, &headers, body,
                        value.0, value.1,
                    ),
                    Err(_) => Self::tunnel_over_proxy(
                        shard, ns, ns_mask, auth_hash, method, &uri, version, &headers, body,
                    ),
                }),
        )
    }

    fn fetch_cached_data(
        shard: u8,
        ns: &str,
        method: &Method,
        headers: &HeaderMap,
    ) -> ProxyServeResultFuture {
        // Clone inner If-None-Match header value (pass it to future)
        let header_if_none_match = headers.get::<IfNoneMatch>().map(std::clone::Clone::clone);
        let ns_string = ns.to_string();

        Box::new(
            CacheRead::acquire_meta(shard, ns, method)
                .and_then(move |result| {
                    match result {
                        Ok(fingerprint) => {
                            debug!(
                                "got fingerprint for cached data = {} on ns = {}",
                                &fingerprint, &ns_string
                            );

                            // Check if not modified?
                            let isnt_modified = match header_if_none_match {
                                Some(ref req_if_none_match) => match req_if_none_match {
                                    &IfNoneMatch::Any => true,
                                    IfNoneMatch::Items(req_etags) => {
                                        if let Some(req_etag) = req_etags.first() {
                                            req_etag.weak_eq(&ETag::new(false, fingerprint.clone()))
                                        } else {
                                            false
                                        }
                                    }
                                },
                                _ => false,
                            };

                            debug!(
                                "got not modified status for cached data = {} on ns = {}",
                                &isnt_modified, &ns_string
                            );

                            Self::fetch_cached_data_body(&ns_string, fingerprint, !isnt_modified)
                        }
                        _ => Box::new(future::ok(Err(()))),
                    }
                })
                .or_else(|_| {
                    error!("failed fetching cached data meta");

                    future::ok(Err(()))
                }),
        )
    }

    fn fetch_cached_data_body(
        ns: &str,
        fingerprint: String,
        do_acquire_body: bool,
    ) -> ProxyServeResultFuture {
        let body_fetcher = if do_acquire_body {
            // Will acquire body (modified)
            CacheRead::acquire_body(ns)
        } else {
            Box::new(future::ok(Ok(None)))
        };

        Box::new(
            body_fetcher
                .and_then(|body_result| {
                    body_result
                        .map_err(|_| ())
                        .map(|body| Ok((fingerprint, body)))
                })
                .or_else(|_| {
                    error!("failed fetching cached data body");

                    future::ok(Err(()))
                }),
        )
    }

    fn tunnel_over_proxy(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        uri: &Uri,
        version: http::Version,
        headers: &HeaderMap,
        body: Body,
    ) -> ProxyServeResponseFuture {
        // Clone method value for closures. Sadly, it looks like Rust borrow \
        //   checker doesnt discriminate properly on this check.
        let method_success = method.clone();
        let method_failure = method.clone();

        Box::new(
            ProxyTunnel::run(&method, uri, headers, body, shard)
                .and_then(move |tunnel_res| {
                    CacheWrite::save(
                        ns,
                        ns_mask,
                        auth_hash,
                        shard,
                        method,
                        version,
                        tunnel_res.status(),
                        tunnel_res.headers().clone(),
                        tunnel_res.body(),
                    )
                })
                .and_then(move |mut result| match result.body {
                    Ok(body_string) => Self::dispatch_fetched(
                        &method_success,
                        result.status,
                        result.headers,
                        HeaderBloomStatusValue::Miss,
                        body_string,
                        result.fingerprint,
                    ),
                    Err(body_string_values) => {
                        match body_string_values {
                            Some(body_string) => {
                                // Enforce clean headers, has usually they get \
                                //   cleaned from cache writer
                                HeaderJanitor::clean(&mut result.headers);

                                Self::dispatch_fetched(
                                    &method_success,
                                    result.status,
                                    result.headers,
                                    HeaderBloomStatusValue::Direct,
                                    body_string,
                                    result.fingerprint,
                                )
                            }
                            _ => Self::dispatch_failure(&method_success),
                        }
                    }
                })
                .or_else(move |_| Self::dispatch_failure(&method_failure)),
        )
    }

    fn dispatch_cached(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        req_uri: &Uri,
        req_version: http::Version,
        req_headers: &HeaderMap,
        req_body: Body,
        res_fingerprint: String,
        res_string: Option<String>,
    ) -> ProxyServeResponseFuture {
        // Response modified? (non-empty body)
        if let Some(res_string_value) = res_string {
            let mut headers = [httparse::EMPTY_HEADER; CACHED_PARSE_MAX_HEADERS];
            let mut res = httparse::Response::new(&mut headers);

            // Split headers from body
            let mut res_body_string = String::new();
            let mut is_last_line_empty = false;

            for res_line in res_string_value.lines() {
                if !res_body_string.is_empty() || is_last_line_empty {
                    // Write to body
                    res_body_string.push_str(res_line.as_ref());
                    res_body_string.push_str(LINE_FEED);
                }

                is_last_line_empty = res_line.is_empty();
            }

            match res.parse(res_string_value.as_bytes()) {
                Ok(_) => {
                    // Process cached status
                    let code = res.code.unwrap_or(500u16);
                    let status =
                        StatusCode::try_from(code).unwrap_or(StatusCode::Unregistered(code));

                    // Process cached headers
                    let mut headers = HeaderMap::new();

                    for header in res.headers {
                        if let (Ok(header_name), Ok(header_value)) = (
                            String::from_utf8(Vec::from(header.name)),
                            String::from_utf8(Vec::from(header.value)),
                        ) {
                            headers.set_raw(header_name, header_value);
                        }
                    }

                    ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));

                    headers
                        .set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

                    // Serve cached response
                    Self::respond(&method, status, headers, res_body_string)
                }
                Err(err) => {
                    error!("failed parsing cached response: {}", err);

                    Self::tunnel_over_proxy(
                        shard,
                        ns,
                        ns_mask,
                        auth_hash,
                        method,
                        req_uri,
                        req_version,
                        req_headers,
                        req_body,
                    )
                }
            }
        } else {
            // Response not modified for client, process non-modified + cached headers
            let mut headers = HeaderMap::new();

            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));
            headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

            // Serve non-modified response
            Self::respond(&method, StatusCode::NotModified, headers, String::new())
        }
    }

    fn dispatch_fetched(
        method: &Method,
        status: StatusCode,
        mut headers: HeaderMap,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        fingerprint: Option<String>,
    ) -> ProxyServeResponseFuture {
        // Process ETag for content?
        if let Some(fingerprint_value) = fingerprint {
            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(fingerprint_value));
        }

        headers.set(HeaderBloomStatus(bloom_status));

        Self::respond(method, status, headers, body_string)
    }

    fn dispatch_failure(method: &Method) -> ProxyServeResponseFuture {
        let status = StatusCode::BadGateway;

        let mut headers = HeaderMap::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        Self::respond(method, status, headers, format!("{status}"))
    }

    fn fingerprint_etag(fingerprint: String) -> ETag {
        ETag::new(false, fingerprint)
    }

    fn respond(
        method: &Method,
        status: StatusCode,
        headers: HeaderMap,
        body_string: String,
    ) -> ProxyServeResponseFuture {
        Box::new(future::ok(match method {
            &Method::Get | &Method::Post | &Method::Patch | &Method::Put | &Method::Delete => {
                Response::new()
                    .with_status(status)
                    .with_headers(headers)
                    .with_body(body_string)
            }
            _ => Response::new().with_status(status).with_headers(headers),
        }))
    }
}
