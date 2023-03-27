// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::conn::Http;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use tokio_core::reactor::Remote;

use super::handle::ServerRequestHandle;
use crate::APP_CONF;

lazy_static! {
    pub static ref LISTEN_REMOTE: Arc<Mutex<Cell<Option<Remote>>>> =
        Arc::new(Mutex::new(Cell::new(None)));
}

pub struct ServerListen;

impl ServerListen {
    pub fn run() {
        let addr = APP_CONF.server.inet;
        let server = Http::new()
            .bind(&addr, move || {
                debug!("handled new request");

                Ok(ServerRequestHandle)
            })
            .expect("error binding server");

        // Assign remote, used later on by the proxy client
        LISTEN_REMOTE
            .lock()
            .unwrap()
            .set(Some(server.handle().remote().clone()));

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().expect("error running server");
    }
}
