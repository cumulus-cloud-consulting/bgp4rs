// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::router_engine::main_router_engine::MainRouterEngine;
use crate::shared::local_address_matcher::HostInterfacesLocalAddressMatcher;
use crate::shared::subsystem::Subsystem;
use crate::web::management_server::ManagementServer;
use app::args::parse;
use log::{error, info};
use std::process;
use std::sync::Arc;

mod app;
mod config_file;
mod router_engine;
mod shared;
mod web;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    match parse() {
        Ok(args) => match args.initialize_logging() {
            Ok(_) => match HostInterfacesLocalAddressMatcher::new() {
                Ok(local_address_matcher) => {
                    let local_address_matcher = Arc::new(local_address_matcher);
                    let (t_router_engine, join_handle) =
                        MainRouterEngine::new(&local_address_matcher).unwrap();
                    let router_engine = Arc::new(t_router_engine);
                    let mut subsystems: Vec<Box<dyn Subsystem>> = Vec::new();

                    if let Some(http_bind_address) = args.http_server_binding() {
                        match ManagementServer::new(
                            http_bind_address,
                            &router_engine,
                            &local_address_matcher,
                        )
                        .await
                        {
                            Ok(server) => subsystems.push(server),
                            Err(err) => {
                                error!("Error starting management web server: {err}");
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
                                                router_engine.await_termination(join_handle).await;

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
