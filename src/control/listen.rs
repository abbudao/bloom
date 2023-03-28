// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tokio::net::TcpListener;
use std::process;

use super::handle::ControlHandle;
use crate::{APP_CONF};

pub struct ControlListen;

impl ControlListen {
    pub async fn run() {
                match TcpListener::bind(APP_CONF.control.inet).await {
                    Ok(listener) => {
                        info!("listening on tcp://{}", APP_CONF.control.inet);

                        match listener.accept().await {
                            Ok((stream, _)) => {
                                ControlHandle::client(stream);
                            }
                            Err(err) => {
                                warn!("error handling stream: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("error binding control listener: {}", err);

                        // Exit Bloom
                        process::exit(1);
                    }
                }
            }
}
