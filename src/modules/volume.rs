use hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::*;

use crate::json::types::*;
use crate::json::utils::get_client_matches;

pub fn handle_volume(
  cmd: hid_io_client::keyboard_capnp::keyboard::signal::volume::Command,
  vol: u16,
  app: Option<&str>,
) -> () {
  match cmd {
    Set => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.volume("", vol as u32);
        })
      } else {
        PactlInput::default().volume("", vol as u32);
      }
    }
    Inc => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.volume("+", vol as u32);
        })
      } else {
        PactlInput::default().volume("+", vol as u32);
      }
    }
    Dec => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.volume("-", vol as u32);
        })
      } else {
        PactlInput::default().volume("-", vol as u32);
      }
    }
    Mute => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.mute();
        })
      } else {
        PactlInput::default().mute();
      }
    }
    UnMute => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.unmute();
        })
      } else {
        PactlInput::default().unmute();
      }
    }
    ToggleMute => {
      if app.is_some() {
        let client = get_client_matches(app.unwrap());
        let sinks = client.condense();
        sinks.iter().for_each(|i| {
          i.toggle_mute();
        })
      } else {
        PactlInput::default().toggle_mute();
      }
    }
  }
}
