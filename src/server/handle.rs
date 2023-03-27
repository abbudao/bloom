// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{http::Request, http::Response, service::Service, Body};

use crate::proxy::serve::{ProxyServe, ProxyServeResponseFuture};

pub struct ServerRequestHandle;

pub async fn server_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    debug!("called proxy serve");
    ProxyServe::handle(req)
}
