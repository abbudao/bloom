// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use tokio_core::reactor::Remote;

use super::handle::server_handler;
use crate::APP_CONF;

lazy_static! {
    pub static ref LISTEN_REMOTE: Arc<Mutex<Cell<Option<Remote>>>> =
        Arc::new(Mutex::new(Cell::new(None)));
}

pub struct ServerListen;

impl ServerListen {
    pub async fn run() {
        let addr = APP_CONF.server.inet;
        let make_service =
            make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(server_handler)) });
        let server = Server::bind(&addr).serve(make_service);

        // Assign remote, used later on by the proxy client
        // LISTEN_REMOTE
        //     .lock()
        //     .unwrap()
        //     .set(Some(server.handle().remote().clone()));

        info!("listening on http://{}", addr);

        server.await.expect("error running server");
    }
}
