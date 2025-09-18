// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::config::router_configuration::{PeerConfiguration, RouterConfiguration};
use crate::shared::error::Error::UnspecifiedError;
use crate::shared::network::local_address_matcher::LocalAddressMatcher;
use crate::shared::prelude::Result;
use crate::shared::services::router_engine::RouterEngine;
use async_trait::async_trait;
use log::{error, info, warn};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

pub struct MainRouterEngine {
    local_address_matcher: Arc<Box<dyn LocalAddressMatcher + Send + Sync>>,
    verb_tx: Sender<RouterControlVerb>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum RouterControlVerb {
    StartRouting,
    StopRouting,
    AddPeer(PeerConfiguration),
    RemovePeer(Uuid),
}

#[derive(Debug)]
pub enum RouterStatusVerb {
    Terminated,
}

#[async_trait]
impl RouterEngine for MainRouterEngine {
    async fn start(&self) -> Result<()> {
        match self.verb_tx.send(RouterControlVerb::StartRouting).await {
            Ok(()) => {
                info!("Successfully sent start routing verb");

                Ok(())
            }
            Err(err) => {
                warn!("Cannot send start routing verb: {}", err);

                Err(UnspecifiedError(anyhow::Error::new(err)))
            }
        }
    }

    async fn stop(&self) -> Result<()> {
        match self.verb_tx.send(RouterControlVerb::StopRouting).await {
            Ok(()) => {
                info!("Successfully sent stop routing verb");

                Ok(())
            }
            Err(err) => {
                warn!("Cannot send stop routing verb: {}", err);

                Err(UnspecifiedError(anyhow::Error::new(err)))
            }
        }
    }

    async fn initial_configuration(&self, router_configuration: RouterConfiguration) -> Result<()> {
        for peer_configuration in router_configuration.peer_configurations {
            if Self::verify_local_address_rule(&peer_configuration, &self.local_address_matcher)
                .await
            {
                info!("Peer {} passed local address rule", &peer_configuration);

                match self
                    .verb_tx
                    .send(RouterControlVerb::AddPeer(peer_configuration))
                    .await
                {
                    Ok(()) => {
                        info!("Successfully sent add peer message");
                    }
                    Err(err) => return Err(UnspecifiedError(anyhow::Error::new(err))),
                }
            } else {
                warn!(
                    "Peer {} does not pass local address rule",
                    &peer_configuration
                );
            }
        }

        Ok(())
    }

    async fn add_peer(&self, peer_configuration: PeerConfiguration) -> Result<()> {
        if Self::verify_local_address_rule(&peer_configuration, &self.local_address_matcher).await {
            info!("Peer {} passed local address rule", &peer_configuration);
        } else {
            warn!(
                "Peer {} does not pass local address rule",
                &peer_configuration
            );
        }

        Ok(())
    }

    async fn remove_peer(&self, peer_id: &Uuid) -> Result<()> {
        match self
            .verb_tx
            .send(RouterControlVerb::RemovePeer(peer_id.clone()))
            .await
        {
            Ok(()) => {
                info!("Successfully send removal message for peer id {peer_id}");

                Ok(())
            }
            Err(err) => {
                error!("Cannot send peer removal message for peer id {peer_id}");

                Err(UnspecifiedError(anyhow::Error::new(err)))
            }
        }
    }

    async fn await_termination(&self, status_rx: &mut Receiver<RouterStatusVerb>) -> () {
        info!("Awaiting router loop termination signal");

        if let Some(verb) = status_rx.recv().await {
            info!("Router event loop finished by verb {}", verb);
        }

        info!("Done waiting for termination signal");

        ()
    }
}

impl MainRouterEngine {
    pub fn new(
        local_address_matcher: &Arc<Box<dyn LocalAddressMatcher + Send + Sync>>,
    ) -> Result<(
        Box<dyn RouterEngine + Send + Sync>,
        Receiver<RouterStatusVerb>,
    )> {
        let (verb_tx, verb_rx) = channel(32);
        let (status_tx, status_rx) = channel(32);

        tokio::spawn(async move { Self::run_event_loop(verb_rx, status_tx).await });

        Ok((
            Box::new(MainRouterEngine {
                local_address_matcher: local_address_matcher.clone(),
                verb_tx,
            }),
            status_rx,
        ))
    }

    async fn verify_local_address_rule(
        peer_configuration: &PeerConfiguration,
        local_address_matcher: &Box<dyn LocalAddressMatcher + Send + Sync>,
    ) -> bool {
        local_address_matcher
            .is_local_address(&peer_configuration.local_address.ip())
            .await
            && !local_address_matcher
                .is_local_address(&peer_configuration.remote_address.ip())
                .await
    }

    async fn run_event_loop(
        mut verb_rx: Receiver<RouterControlVerb>,
        status_tx: Sender<RouterStatusVerb>,
    ) {
        loop {
            match verb_rx.recv().await {
                Some(router_verb) => {
                    info!("Received router control verb {}", router_verb);

                    match router_verb {
                        RouterControlVerb::StopRouting => {
                            info!("Stopping main router event loop");

                            break;
                        }
                        RouterControlVerb::AddPeer(peer_configuration) => {
                            info!("Adding new peer {} to router engine", &peer_configuration);
                        }
                        RouterControlVerb::RemovePeer(peer_id) => {
                            info!("Removing peer {} from router engine", &peer_id);
                        }
                        RouterControlVerb::StartRouting => {
                            info!("Start routing processes");
                        }
                    }
                }
                None => {
                    warn!("Router control channel closed");

                    break;
                }
            }
        }

        if let Err(err) = status_tx.send(RouterStatusVerb::Terminated).await {
            warn!("Cannot send router termination verb: {}", err);
        }

        info!("Exiting router event loop");
    }
}

impl Display for RouterControlVerb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterControlVerb::StartRouting => write!(f, "StartRouting"),
            RouterControlVerb::StopRouting => write!(f, "StopRouting"),
            RouterControlVerb::AddPeer(peer_configuration) => {
                write!(f, "AddPeer({peer_configuration})")
            }
            RouterControlVerb::RemovePeer(uuid) => write!(f, "RemovePeer({uuid})"),
        }
    }
}

impl Display for RouterStatusVerb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterStatusVerb::Terminated => write!(f, "Terminated"),
        }
    }
}
