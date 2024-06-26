use std::process::Command;

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
pub struct PactlJSONInput {
  pub index: u32,
  pub sink: u32,
  pub client: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PactlInput {
  pub index: String,
  pub sink: u32,
  pub client: String,
}

impl PactlInput {
  pub fn default() -> Self {
    Self {
      index: "".to_string(),
      sink: 0,
      client: String::new(),
    }
  }

  pub fn volume(&self, prefix: &str, volume: u32) {
    match self.index.len() {
      0 => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-volume")
            .arg("@DEFAULT_SINK@")
            .arg(prefix.to_string() + &volume.to_string() + "%")
            .output()
            .unwrap(),
        );
      }
      _ => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-input-volume")
            .arg(self.index.clone())
            .arg(prefix.to_string() + &volume.to_string() + "%")
            .output()
            .unwrap(),
        );
      }
    }
  }

  pub fn mute(&self) {
    match self.index.len() {
      0 => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-mute")
            .arg("@DEFAULT_SINK@")
            .arg("1")
            .output()
            .unwrap(),
        );
      }
      _ => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-input-mute")
            .arg(self.index.to_string())
            .arg("1")
            .output()
            .unwrap(),
        );
      }
    }
  }

  pub fn unmute(&self) {
    match self.index.len() {
      0 => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-mute")
            .arg("@DEFAULT_SINK@")
            .arg("0")
            .output()
            .unwrap(),
        );
      }
      _ => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-input-mute")
            .arg(self.index.to_string())
            .arg("0")
            .output()
            .unwrap(),
        );
      }
    }
  }

  pub fn toggle_mute(&self) {
    match self.index.len() {
      0 => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-mute")
            .arg("@DEFAULT_SINK@")
            .arg("toggle")
            .output()
            .unwrap(),
        );
      }
      _ => {
        crate::util::log_cmd(
          &Command::new("pactl")
            .arg("set-sink-input-mute")
            .arg(self.index.to_string())
            .arg("toggle")
            .output()
            .unwrap(),
        );
      }
    }
  }
}

pub trait Condense {
  fn condense(&self) -> Vec<PactlInput>;
}

impl Condense for Vec<PactlClient> {
  fn condense(&self) -> Vec<PactlInput> {
    self
      .iter()
      .fold(Vec::new(), |mut acc, c| {
        c.get_inputs().iter().for_each(|i| {
          acc.push(i.clone());
        });
        acc
      })
      .iter()
      .fold(Vec::<PactlInput>::new(), |mut acc, i| {
        if acc.iter().find(|a| a.index == i.index).is_none() {
          acc.push(i.clone());
          acc
        } else {
          acc
        }
      })
  }
}
