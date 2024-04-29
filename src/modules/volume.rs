use hid_io_protocol::commands::h0060;

use crate::json::types::*;
use crate::json::utils::get_client_matches;

pub fn handle_volume(out: &str) -> () {
  let splt = out.split(':').collect::<Vec<&str>>();
  let cmdnum = splt[0][7..].to_string();
  let volcmd = h0060::Command::try_from(cmdnum.as_str()).unwrap();
  let vol = splt[1].parse::<u16>().unwrap();
  let app: Option<&str> = if splt.len() > 2 { Some(splt[2]) } else { None };
  match volcmd {
    h0060::Command::Set => {
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
    h0060::Command::Inc => {
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
    h0060::Command::Dec => {
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
    h0060::Command::Mute => {
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
    h0060::Command::UnMute => {
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
    h0060::Command::ToggleMute => {
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
    h0060::Command::InvalidCommand => {
      println!("ERROR: InvalidCommand");
    }
  }
}
