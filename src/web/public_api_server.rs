use crate::shared::error::Error::{InvalidBindAddressError, IoError};
use crate::shared::network::local_address_matcher::LocalAddressMatcher;
use crate::shared::network::socket_addr_spec::SocketAddressSpec;
use crate::shared::services::router_engine::RouterEngine;
use crate::shared::services::subsystem::Subsystem;
use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

const SUBSYSTEM: &'static str = "Public API";

/// The overall structure of the API management server.
pub struct PublicApiServer {
    termination: RwLock<Box<dyn Fn() + Send + Sync>>,
}

/// Internal structure holding information required to run the management API server.
#[derive(Clone)]
struct PublicAppState {
    /// Reference to the main router engine.
    router_engine: Arc<Box<dyn RouterEngine + Send + Sync>>,
}

/// Payload to be returned as JSON payload
#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub healthy: bool,
}

impl PublicApiServer {
    /// Create a new instance of the public API server.
    ///
    /// Arguments:
    /// * `bind_addr_spec` - The socket address (IP address and port number) to bind the
    /// management API server to.
    /// * `router_engine` - Reference to the main router engine. This provides the backing
    /// functionality required to implement the public endpoints.
    /// * `local_address_matcher` - Match the provided binding socket address against the
    /// list of available network interfaces (and their respective IP addresses). If the
    /// binding address cannot be matched with an available network interface, the public
    /// API server will not be started and an error is raised to the caller.
    ///
    pub async fn new(
        bind_addr_spec: SocketAddressSpec,
        router_engine: &Arc<Box<dyn RouterEngine + Send + Sync>>,
        local_address_matcher: &Arc<Box<dyn LocalAddressMatcher + Send + Sync>>,
    ) -> crate::shared::prelude::Result<Box<dyn Subsystem>> {
        let bind_addr = TryInto::<SocketAddr>::try_into(bind_addr_spec).unwrap();

        if local_address_matcher
            .is_local_address(&bind_addr.ip())
            .await
        {
            info!("Starting management HTTP server on {}", &bind_addr);

            let shared_state = Arc::new(crate::web::public_api_server::PublicAppState {
                router_engine: router_engine.clone(),
            });

            let app = Router::new()
                .route("/health", get(Self::health))
                .with_state(shared_state);

            match tokio::net::TcpListener::bind(bind_addr).await {
                Ok(listener) => {
                    let join_handle = tokio::spawn(async move { axum::serve(listener, app).await });

                    info!("Public API server listening on {}", bind_addr);

                    Ok(Box::new(PublicApiServer {
                        termination: RwLock::new(Box::new(move || join_handle.abort())),
                    }))
                }
                Err(err) => {
                    error!("Cannot bind public API server to {}: {}", bind_addr, err);

                    Err(IoError(err))
                }
            }
        } else {
            Err(InvalidBindAddressError {
                bind_addr: bind_addr.clone(),
            })
        }
    }

    /// Implement the */health* endpoint by executing the *is_running()* method of the provided
    /// main router engine reference.
    ///
    /// Arguments:
    /// * `state` - The shared state containing references and data values required across all
    /// management API endpoints.
    async fn health(
        State(state): State<Arc<crate::web::public_api_server::PublicAppState>>,
    ) -> impl IntoResponse {
        let health = Health {
            healthy: state.router_engine.is_running().await,
        };

        (StatusCode::OK, Json(health))
    }

    /// Abort the spawned listener task
    fn abort_server(&self) {
        match self.termination.read() {
            Ok(termination) => {
                (termination.as_ref())();

                info!("Public API server task terminated");
            }
            Err(err) => {
                error!("Failed to acquire lock for public API server: {}", err);
            }
        }
    }
}

#[async_trait]
impl Subsystem for PublicApiServer {
    async fn stop(&self) -> () {
        self.abort_server()
    }

    fn name(&self) -> &'static str {
        SUBSYSTEM
    }
}
