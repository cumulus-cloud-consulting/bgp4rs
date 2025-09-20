// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
//! Impplementation of the management API server providing endpoints to control router operations:
//!
//! The supported ReST endpoints are:
//! - **POST /stop**: Stop the routing processes, terminate all active connections and stop the
//! *bgp4rs* process.
//!
use crate::shared::error::Error::{InvalidBindAddressError, IoError};
use crate::shared::network::local_address_matcher::LocalAddressMatcher;
use crate::shared::network::socket_addr_spec::SocketAddressSpec;
use crate::shared::prelude::Result;
use crate::shared::services::router_engine::RouterEngine;
use crate::shared::services::subsystem::Subsystem;
use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

const SUBSYSTEM: &'static str = "Management API";

/// The overall structure of the API management server.
pub struct ManagementApiServer {
    termination: RwLock<Box<dyn Fn() + Send + Sync>>,
}

/// Internal structure holding information required to run the management API server.
#[derive(Clone)]
struct ManagementAppState {
    /// Reference to the main router engine.
    router_engine: Arc<Box<dyn RouterEngine + Send + Sync>>,
}

impl ManagementApiServer {
    /// Create a new instance of the management API server.
    ///
    /// Arguments:
    /// * `bind_addr_spec` - The socket address (IP address and port number) to bind the
    /// management API server to.
    /// * `router_engine` - Reference to the main router engine. This provides the backing
    /// functionality required to implement the management endpoints.
    /// * `local_address_matcher` - Match the provided binding socket address against the
    /// list of available network interfaces (and their respective IP addresses). If the
    /// binding address cannot be matched with an available network interface, the management
    /// API server will not be started and an error is raised to the caller.
    ///
    pub async fn new(
        bind_addr_spec: SocketAddressSpec,
        router_engine: &Arc<Box<dyn RouterEngine + Send + Sync>>,
        local_address_matcher: &Arc<Box<dyn LocalAddressMatcher + Send + Sync>>,
    ) -> Result<Box<dyn Subsystem>> {
        let bind_addr = TryInto::<SocketAddr>::try_into(bind_addr_spec).unwrap();

        if local_address_matcher
            .is_local_address(&bind_addr.ip())
            .await
        {
            info!("Starting management HTTP server on {}", &bind_addr);

            let shared_state = Arc::new(ManagementAppState {
                router_engine: router_engine.clone(),
            });

            let app = Router::new()
                .route("/stop", post(Self::stop_router))
                .with_state(shared_state);

            match tokio::net::TcpListener::bind(bind_addr).await {
                Ok(listener) => {
                    let join_handle = tokio::spawn(async move { axum::serve(listener, app).await });

                    info!("Management server listening on {}", bind_addr);

                    Ok(Box::new(ManagementApiServer {
                        termination: RwLock::new(Box::new(move || join_handle.abort())),
                    }))
                }
                Err(err) => {
                    error!("Cannot bind management server to {}: {}", bind_addr, err);

                    Err(IoError(err))
                }
            }
        } else {
            Err(InvalidBindAddressError {
                bind_addr: bind_addr.clone(),
            })
        }
    }

    /// Implement the */stop* endpoint by executing the *stop()* method of the provided
    /// main router engine reference.
    ///
    /// Arguments:
    /// * `state` - The shared state containing references and data values required across all
    /// management API endpoints.
    async fn stop_router(State(state): State<Arc<ManagementAppState>>) -> impl IntoResponse {
        match state.router_engine.stop().await {
            Ok(_) => {
                info!("Stopped router from management server");

                StatusCode::OK.into_response()
            }
            Err(err) => {
                error!("Cannot stop router: {}", err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    /// Abort the spawned listener task
    fn abort_server(&self) {
        match self.termination.read() {
            Ok(termination) => {
                (termination.as_ref())();

                info!("Management API server task terminated");
            }
            Err(err) => {
                error!("Cannot abort management API server: {}", err);
            }
        }
    }
}

#[async_trait]
impl Subsystem for ManagementApiServer {
    async fn stop(&self) -> () {
        self.abort_server()
    }

    fn name(&self) -> &'static str {
        SUBSYSTEM
    }
}
