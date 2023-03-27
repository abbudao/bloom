// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{http::Request, http::Response, service::Service};

use crate::proxy::serve::{ProxyServe, ProxyServeResponseFuture};

pub struct ServerRequestHandle;

impl Service<Request<Vec<u8>>> for ServerRequestHandle {
    type Response = Response<Vec<u8>>;
    type Error = hyper::Error;
    type Future = ProxyServeResponseFuture;

    fn call(&self, req: Request<Vec<u8>>) -> ProxyServeResponseFuture {
        debug!("called proxy serve");

        ProxyServe::handle(req)
    }
}
