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
use std::sync::Arc;

pub struct ManagementServer {}

#[derive(Clone)]
struct AppState {
    router_engine: Arc<Box<dyn RouterEngine + Send + Sync>>,
}

impl ManagementServer {
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

            let shared_state = Arc::new(AppState {
                router_engine: router_engine.clone(),
            });

            let app = Router::new()
                .route("/stop", post(stop_router))
                .with_state(shared_state);

            match tokio::net::TcpListener::bind(bind_addr).await {
                Ok(listener) => {
                    tokio::spawn(async move { axum::serve(listener, app).await });

                    info!("Management server listening on {}", bind_addr);

                    Ok(Box::new(ManagementServer {}))
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
}

async fn stop_router(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

#[async_trait]
impl Subsystem for ManagementServer {
    async fn stop(&self) -> () {
        ()
    }
}
