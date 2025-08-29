use serde::{Deserialize, Serialize};
use twelf::config;

#[config]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
struct EngineConfigFile {

    peers : Vec<PeerConfigFile>,
}

#[derive(Deserialize, Serialize, Default)]
struct PeerConfigFile {}
