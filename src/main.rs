use crate::router_engine::main_router_engine::MainRouterEngine;
use app::args::parse;
use std::rc::Rc;
use std::process;

mod shared;
mod app;
mod config_file;
mod router_engine;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    match parse() {
        Ok(args) => {
            let router_engine = Rc::new(MainRouterEngine::new());

            match args.config_provider(&router_engine) {
                Ok(config_provider) => {
                    match config_provider.provide_configuration() {
                        Ok(initial_configuration) => {
                            match router_engine.initial_configuration(&initial_configuration) {
                                Ok(()) => {
                                    match router_engine.start() {
                                        Ok(()) => {}
                                        Err(err) => {
                                            eprintln!("Error starting routing engine: {err}");
                                            process::exit(1);
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Error providing initial configuration to router engine: {err}");
                                    process::exit(1);
                                }
                            }
                        },
                        Err(err) => {
                            eprintln!("Error loading configuration: {err}");
                            process::exit(1);
                        }
                    }
                },
                Err(err) => {
                    eprintln!("Error determining configuration provider: {err}");
                    process::exit(1);
                }
            }
        },
        Err(err) => {
            eprintln!("Error handling command-line argument: {err}");
            process::exit(1);
        }
    }
}
