use crate::shared::error::Error;
use crate::shared::error::Error::{InvalidBindAddressError, IoError};
use crate::shared::local_address_matcher::LocalAddressMatcher;
use crate::shared::prelude::Result;
use crate::shared::router_engine::RouterEngine;
use crate::shared::socket_addr_spec::SocketAddressSpec;
use crate::shared::subsystem::Subsystem;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;

pub struct ManagementServer {
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
            let stop_engine = router_engine.clone();

            let app = Router::new().route(
                "/stop",
                post(|| async {
                    // stop_router(stop_engine).await;
                }),
            );

            match tokio::net::TcpListener::bind(bind_addr).await {
                Ok(listener) => match axum::serve(listener, app).await {
                    Ok(_) => Ok(Box::new(ManagementServer {
                        router_engine: router_engine.clone(),
                    })),
                    Err(err) => {
                        error!("Cannot start management server: {}", err);

                        Err(IoError(err))
                    }
                },
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

#[async_trait]
impl Subsystem for ManagementServer {
    async fn stop(&self) -> () {
        ()
    }
}

async fn stop_router(router_engine: Arc<Box<dyn RouterEngine + Send + Sync>>) -> Response {
    match router_engine.stop().await {
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
