use serde::{Deserialize, Serialize};

use super::utils::{get_clients, get_sink_inputs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PactlClient {
  pub index: u32,
  pub driver: String,
  #[serde(rename = "application.process.binary")]
  pub application_process_binary: String,
}

impl PactlClient {
  pub fn get_inputs(&self) -> Vec<PactlInput> {
    let clients = get_clients();
    let inputs = get_sink_inputs();
    let app = &self.application_process_binary;
    let client_match = clients.iter().filter(|c| c.application_process_binary.contains(app));
    let inputs = inputs.iter().filter(|i| {
      client_match.clone().filter(|c| c.index == i.client.parse::<u32>().unwrap()).count() > 0
    });
    inputs.map(|i| i.clone()).collect()
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PactlInput {
  pub index: u32,
  pub sink: u32,
  pub client: String,
  #[serde(skip)]
  driver: String,
  #[serde(skip)]
  sample_specification: String,
}


