// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use headers::{ETag, Vary, HeaderMapExt};
use hyper::HeaderMap;
use std::str::from_utf8;
use unicase::Ascii;

use super::defaults;
use crate::{header::request_shard::HeaderRequestBloomRequestShard, APP_CONF};

pub struct ProxyHeader;

impl ProxyHeader {
    pub fn parse_from_request(headers: HeaderMap) -> (HeaderMap, String, u8) {
        // Request header: 'Authorization'
        let auth = match headers.get("authorization") {
            None => defaults::REQUEST_AUTHORIZATION_DEFAULT,
            Some(value) => from_utf8(value.as_bytes())
                .unwrap_or(defaults::REQUEST_AUTHORIZATION_DEFAULT),
        }
        .to_string();

        // Request header: 'Bloom-Request-Shard'
        let shard = match headers.typed_get::<HeaderRequestBloomRequestShard>() {
            None => APP_CONF.proxy.shard_default,
            Some(value) => value.0,
        };

        (headers, auth, shard)
    }

    pub fn set_etag(headers: &mut HeaderMap, etag: ETag) {
        headers.insert::<Vary>(Vary::Items(vec![Ascii::new(
            ETag::name()
        )]));

        headers.typed_insert(etag);
    }
}
