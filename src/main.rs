// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
//! ## BGP4RS
//! A network daemon implementing the BGP4 protocol
//!
//! This crate implements a standalone network daemon acting as a peer in the communication
//! between network routers exchanging network reachability information aka network routes
//!
//! While this kind of information is usually exchanged between Internet routers, it can become
//! to gain a view into the routing information possessed by a particular network router.
//!
//! The purpose on this daemon implementation is to act as a bridge between one or more entities
//! possessing the reachability information, the *routers*. and one or more entities consuming
//! the reachability information as a snapshot in time or an initial snapshot followed by
//! continuous stream of updates whenever the reachability information changes, the *monitors*.
//!
//! ### Supported RFC
//! This implementation of the BGP4 protocol supports the protocol according to these
//! standardization documents:
//! - [RFC4271](https://datatracker.ietf.org/doc/html/rfc4271 "A Border Gateway Protocol 4 (BGP-4)")
//! - [RFC 6793](https://datatracker.ietf.org/doc/html/rfc6793 "BGP Support for Four-Octet Autonomous System (AS) Number Space")
//!
//! ### Usage
//! The application supports some command-line options for initial configuration.
//!
//! For the arguments supported, refer to the [`main`] entry point
//!
//! ### Builtin management server
//! If enabled, the management server supports the following ReST endpoints:
//! - **POST /stop**: Stop the routing processes, terminate all active connections and stop the
//! *bgp4rs* process.
//!
//! ### Builtin API server
//! If enable, the public API server support the following ReST endpoints:
//! - **GET /health**: Obtain health information about the overall server process.
//!
use crate::router_engine::main_router_engine::MainRouterEngine;
use crate::web::management_api_server::ManagementApiServer;
use crate::web::public_api_server::PublicApiServer;
use app::args::parse;
use log::{error, info};
use shared::network::local_address_matcher::HostInterfacesLocalAddressMatcher;
use shared::services::subsystem::Subsystem;
use std::process;
use std::sync::Arc;

mod app;
mod config_file;
mod router_engine;
mod shared;
mod web;

/// # Main application entry point
/// This is the main application point into the application.
///
/// It performs the following functionality:
/// - Parse command-line arguments
/// - Initialize log system
/// - Initialize the main router engine
/// - Start the built-in management API web server, if configured
/// - Start the built-in public API web server, if configured
/// - Start the main router engine
/// - Await termination of the main router engine
/// - Shutdown any active subsystem
///
/// ## Usage
/// The application support the following command-line options and environment variables:
/// - **-c <CONFIG_TYPE>** or **--config-type <CONFIG_TYPE>**: Select which configuration
/// source is used for obtaining peer configuration information. The available options are:
///     - **file** read configuration file. Requires a peer configuration file to be available
///  - **-r <ROUTER_CONFIG_PATH>** or **--router-config-path <ROUTER_CONFIG_PATH>**: Read the peer
/// configuration file from the path denoted by the argument to this command-line options. It is
/// only evaluated if the configuration type is *file* (default value). Alternatively, the value
/// is read from the environment variable **ROUTER_CONFIG_PATH**.
/// - **--management-server-bind-addr <MANAGEMENT_SERVER_BIND_ADDR>**: Bind the management web
/// the value is derived from the environment variable **MANAGEMENT_SERVER_BIND_ADDR**.
/// *Please note:* The management server is only started if the management server port number is
/// also specified.
/// - **--management-server-port <MANAGEMENT_SERVER_PORT>**: Bind the management web server to
///  the given port number (in the range *1* to *65535*). Alternatively,
/// the value is derived from the environment variable **MANAGEMENT_SERVER_PORT**.
/// *Please note:* The management server is only started if the management server bind address is
/// also specified.
/// - **--api-server-bind-addr <API_SERVER_BIND_ADDR>**: Bind the API web
/// server to the given address in the form of an *IPv4* or an *IPv6* address. Alternatively,
/// the value is derived from the environment variable **API_SERVER_BIND_ADDR**.
/// *Please note:* The API server is only started if the API server port number is also
/// specified.
/// - **--api-server-port <API_SERVER_PORT>**: Bind the API web server to
///  the given port number (in the range *1* to *65535*). Alternatively,
/// the value is derived from the environment variable **API_SERVER_PORT**.
/// *Please note:* The API server is only started if the API server bind address is
/// also specified.
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    match parse() {
        Ok(args) => match args.initialize_logging() {
            Ok(_) => match HostInterfacesLocalAddressMatcher::new() {
                Ok(local_address_matcher) => {
                    let local_address_matcher = Arc::new(local_address_matcher);
                    let (t_router_engine, mut status_rx) =
                        MainRouterEngine::new(&local_address_matcher).unwrap();
                    let router_engine = Arc::new(t_router_engine);
                    let mut subsystems: Vec<Box<dyn Subsystem>> = Vec::new();

                    let opt_mgmt_api_binding = args.management_api_server_binding();
                    let opt_public_api_binding = args.public_api_server_binding();

                    if opt_mgmt_api_binding == opt_public_api_binding {
                        error!("Cannot use identical port bindings for management and public API ");

                        process::exit(1);
                    }

                    if let Some(http_bind_address) = opt_mgmt_api_binding {
                        match ManagementApiServer::new(
                            http_bind_address,
                            &router_engine,
                            &local_address_matcher,
                        )
                        .await
                        {
                            Ok(server) => subsystems.push(server),
                            Err(err) => {
                                error!("Error starting management API web server: {err}");
                                process::exit(1);
                            }
                        }
                    }

                    if let Some(http_bind_address) = opt_public_api_binding {
                        match PublicApiServer::new(
                            http_bind_address,
                            &router_engine,
                            &local_address_matcher,
                        )
                        .await
                        {
                            Ok(server) => subsystems.push(server),
                            Err(err) => {
                                error!("Error starting public API web server: {err}");
                                process::exit(1);
                            }
                        }
                    }

                    match args.config_provider(&router_engine) {
                        Ok(config_provider) => match config_provider.provide_configuration() {
                            Ok(initial_configuration) => {
                                match router_engine
                                    .initial_configuration(initial_configuration)
                                    .await
                                {
                                    Ok(()) => {
                                        info!("Starting router engine");

                                        match router_engine.start().await {
                                            Ok(()) => {
                                                router_engine
                                                    .await_termination(&mut status_rx)
                                                    .await;

                                                for subsystem in subsystems {
                                                    info!(
                                                        "Stopping subsystem {}",
                                                        subsystem.name()
                                                    );

                                                    subsystem.stop().await;
                                                }

                                                info!("Router engine terminated");
                                            }
                                            Err(err) => {
                                                error!("Error starting routing engine: {err}");
                                                process::exit(1);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        error!(
                                            "Error providing initial configuration to router engine: {err}"
                                        );
                                        process::exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Error loading configuration: {err}");
                                process::exit(1);
                            }
                        },
                        Err(err) => {
                            eprintln!("Error determining configuration provider: {err}");
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error initializing local interface address matcher: {err}");
                    process::exit(1);
                }
            },
            Err(error) => {
                eprintln!("Error initializing log system: {error}");
                process::exit(1);
            }
        },
        Err(err) => {
            eprintln!("Error handling command-line argument: {err}");
            process::exit(1);
        }
    }
}
