// You need to bring the ToString trait into scope to use it
use std::{any::Any, collections::HashMap, fs};

use process_compose::{
    HttpProbe, LogConfiguration, LogRotationConfig, Probe, Process, Project, ShutdownConfig,
};
use serde::{Deserialize, Serialize};

mod galois;
mod process_compose;
mod voyager;

const LOGS_BASE_PATH: &str = "./.devnet/logs/";

pub fn log_path(process_name: &str) -> String {
    format!("{LOGS_BASE_PATH}/{}.log", process_name)
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Network {
    Union,
    Osmosis,
    Stargaze,
}

impl Network {
    fn to_process(self) -> Process {
        Process {
            name: self.network_id().clone(),
            command: format!("nix run .#{}", self.network_id()),
            is_daemon: None,
            disabled: None,
            depends_on: None,
            liveliness_probe: None,
            readiness_probe: Some(Probe::http_get(self.probe_port(), "/block?height=2")),
            log_configuration: LogConfiguration::default(),
            log_location: log_path(&self.network_id()),
            shutdown: ShutdownConfig::default(),
        }
    }

    fn network_id(&self) -> String {
        format!("devnet-{}", self.to_string().to_lowercase())
    }

    fn probe_port(&self) -> usize {
        match self {
            Network::Union => 26657,
            Network::Stargaze => 26757,
            Network::Osmosis => 26857,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevnetConfig {
    pub networks: Vec<Network>,
    pub connections: Vec<(Network, Network)>,
}

impl DevnetConfig {
    pub fn to_process_compose(&self) -> Project {
        let mut project = Project::default();

        // Add a devnet for each network
        for network in self.networks.clone() {
            project.add_process(network.to_process());
        }

        if !self.connections.is_empty() {
            // There are connections, so we need voyager running with applied migrations
            project.add_process(voyager::queue_process());
            project.add_process(voyager::migrations_process());

            if self.networks.contains(&Network::Union) {
                // There are connections to Union, so we need to prove Union consensus
                project.add_process(galois::download_circuit_process());
                project.add_process(galois::galoisd_process());
            }
        }

        project
    }
}

fn main() {
    use Network::*;
    let config = DevnetConfig {
        networks: vec![Union, Osmosis, Stargaze],
        connections: vec![(Union, Osmosis)],
    };

    let project = config.to_process_compose();
    let project = serde_json::to_string_pretty(&project).expect("failed to serialize project");

    fs::write("process-compose.yml", project).expect("failed to write contents");
}
