/* Copyright (C) 2019-2023 by Jacob Alexander
 * Copyright (C) 2019 by Rowan Decker
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

extern crate tokio;

use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Output;

use capnp::traits::IntoInternalStructReader;
use hid_io_client::capnp;
use hid_io_client::capnp::capability::Promise;
use hid_io_client::capnp_rpc;
use hid_io_client::common_capnp::NodeType;
use hid_io_client::keyboard_capnp;
use hid_io_client::setup_logging_lite;
use hid_io_protocol::commands::h0060;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PactlClient {
  pub index: u32,
  pub driver: String,
  #[serde(rename = "application.process.binary")]
  pub application_process_binary: String,
}

impl PactlClient {
  fn get_inputs(&self) -> Vec<PactlInput> {
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
struct PactlInput {
  index: u32,
  sink: u32,
  client: String,
  #[serde(skip)]
  driver: String,
  #[serde(skip)]
  sample_specification: String,
}

fn get_sink_inputs() -> Vec<PactlInput> {
  let inputs = Command::new("pactl")
    .arg("--format=json")
    .arg("list")
    .arg("short")
    .arg("sink-inputs")
    .output()
    .unwrap();
  serde_json::from_slice(&inputs.stdout).unwrap()
}

fn get_client_matches(app: &str) -> Vec<PactlClient> {
  let clients = get_clients();
  let client_match = clients.iter().filter(|c| c.application_process_binary.contains(app));
  client_match.map(|c| c.clone()).collect()
}

fn get_clients() -> Vec<PactlClient> {
  let paclients = Command::new("pactl")
    .arg("--format=json")
    .arg("list")
    .arg("short")
    .arg("clients")
    .output()
    .unwrap();
  serde_json::from_slice(&paclients.stdout).unwrap()
}

fn log_cmd(cmd: &Output) {
  if !cmd.status.success() {
    panic!(
      "ERROR: pamixer - {} -- {}",
      String::from_utf8(cmd.stderr.clone()).unwrap(),
      String::from_utf8(cmd.stdout.clone()).unwrap()
    );
  } else {
    println!(
      "ERROR: pamixer - {} -- {}",
      String::from_utf8(cmd.stderr.clone()).unwrap(),
      String::from_utf8(cmd.stdout.clone()).unwrap()
    );
  }
}

#[derive(Default)]
pub struct KeyboardSubscriberImpl {}

impl keyboard_capnp::keyboard::subscriber::Server for KeyboardSubscriberImpl {
  // fn update(
  //     &mut self,
  //     params: keyboard_capnp::keyboard::subscriber::UpdateParams,
  //     _results: keyboard_capnp::keyboard::subscriber::UpdateResults,
  // ) -> Promise<(), capnp::Error> {
  //     let st = capnp_rpc::pry!(capnp_rpc::pry!(params.get()).get_signal())
  //         .get_data()
  //         .to_owned();
  //     // Only read cli messages
  //     if st.which().is_ok() {
  //         let signaltype = st.which().unwrap();
  //         match signaltype {
  //             hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Volume(v) => {
  //                 let v = v.unwrap();
  //                 let cmd = v.get_cmd().unwrap();
  //                 let vol = v.get_vol();
  //                 /* let app = v
  //                  *     .get_app()
  //                  *     .unwrap()
  //                  *     .iter()
  //                  *     .map(|n| n.unwrap().to_string())
  //                  *     .collect::<Vec<String>>();
  //                  * print!("{:?}, {:?}, {:?}", cmd, vol, app);
  //                  */
  //                 print!("{:?}, {:?}", cmd, vol);
  //                 match cmd {
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::Set => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--set-volume")
  //                             .arg(vol.to_string())
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::Inc => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--increase")
  //                             .arg(vol.to_string())
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::Dec => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--decrease")
  //                             .arg(vol.to_string())
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::Mute => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--mute")
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::UnMute => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--unmute")
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                     hid_io_client::keyboard_capnp::keyboard::signal::volume::Command::ToggleMute => {
  //                         match Command::new("pamixer")
  //                             .arg("--default-source")
  //                             .arg("--toggle-mute")
  //                             .output()
  //                         {
  //                             Ok(n) => {
  //                                 println!("pamixer1: {}", String::from_utf8(n.stderr).unwrap());
  //                             }
  //                             Err(e) => {
  //                                 panic!("pamixer: {}", e);
  //                             }
  //                         }
  //                     }
  //                 }
  //                 std::io::stdout().flush().unwrap();
  //             }
  //             _ => {}
  //         }
  //     } else {
  //         println!("Unknown signal");
  //     }
  //
  //     Promise::ok(())
  // }

  fn update(
    &mut self,
    params: keyboard_capnp::keyboard::subscriber::UpdateParams,
    _results: keyboard_capnp::keyboard::subscriber::UpdateResults,
  ) -> Promise<(), capnp::Error> {
    // println!("Data: {}", params.get().unwrap().get_signal().unwrap().get_data().into_internal_struct_reader().get_data_field::<u16>(1));
    let data = params
      .get()
      .unwrap()
      .get_signal()
      .unwrap()
      .get_data()
      .into_internal_struct_reader()
      .get_data_field::<u16>(1);
    let params = capnp_rpc::pry!(capnp_rpc::pry!(params.get()).get_signal()).get_data().to_owned();
    println!("{:?}", data);
    match params.which().unwrap() {
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Volume(v) => {
        let v = v.unwrap();
        println!("Volume: {:?}, {}", v.get_cmd().unwrap(), v.get_vol());
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Cli(c) => {
        if data > 0 {
          let out = c.unwrap().get_output().unwrap();
          println!("Cli: {}", out);
          if out.starts_with("volume-") {
            let splt = out.split(':').collect::<Vec<&str>>();
            let cmdnum = splt[0][7..].to_string();
            let volcmd = h0060::Command::try_from(cmdnum.as_str()).unwrap();
            let vol = splt[1].parse::<u16>().unwrap();
            let app: Option<&str> = if splt.len() > 2 { Some(splt[2]) } else { None };
            match volcmd {
              h0060::Command::Set => {
                // let clients = get_clients();
                // for client in clients.iter() {
                //   println!("Client: {:?}", client);
                // }
                let cmd = Command::new("pactl")
                  .arg("set-sink-volume")
                  .arg("@DEFAULT_SINK@")
                  .arg(format!("{}%", vol))
                  .output()
                  .unwrap();
                log_cmd(&cmd);
              }
              h0060::Command::Inc => {
                let cmd = Command::new("pactl")
                  .arg("set-sink-volume")
                  .arg("@DEFAULT_SINK@")
                  .arg(format!("+{}%", vol))
                  .output()
                  .unwrap();
                log_cmd(&cmd);
              }
              h0060::Command::Dec => {
                let cmd = Command::new("pactl")
                  .arg("set-sink-volume")
                  .arg("@DEFAULT_SINK@")
                  .arg(format!("-{}%", vol))
                  .output()
                  .unwrap();
                log_cmd(&cmd);
              }
              h0060::Command::Mute => {
                let cmd = Command::new("pactl")
                  .arg("set-sink-mute")
                  .arg("@DEFAULT_SINK@")
                  .arg("1")
                  .output()
                  .unwrap();
                log_cmd(&cmd);
              }
              h0060::Command::UnMute => {
                let cmd: Output;
                if app.is_some() {
                  cmd = Command::new("pactl")
                    .arg("set-sink-mute")
                    .arg("@DEFAULT_SINK@")
                    .arg("0")
                    .output()
                    .unwrap();
                } else {
                  cmd = Command::new("pactl")
                    .arg("set-sink-mute")
                    .arg("@DEFAULT_SINK@")
                    .arg("0")
                    .output()
                    .unwrap();
                }
                log_cmd(&cmd);
              }
              h0060::Command::ToggleMute => {
                if app.is_some() {
                  let mut sinks: Vec<PactlInput> = Vec::new();
                  let client = get_client_matches(app.unwrap());
                  client.iter().for_each(|c| {
                    c.get_inputs().iter().for_each(|i| {
                      sinks.push(i.clone());
                    })
                  });
                  sinks = sinks.iter().fold(Vec::new(), |mut acc, i| {
                    if acc.iter().find(|a| a.sink == i.sink).is_none() {
                      acc.push(i.clone());
                      acc
                    } else {
                      acc
                    }
                  });
                  sinks.iter().for_each(|i| {
                    println!("Sink: {:?}", i);
                    let cmd = Command::new("pactl")
                      .arg("set-sink-input-mute")
                      .arg(i.sink.to_string())
                      .arg("toggle")
                      .output()
                      .unwrap();
                    log_cmd(&cmd);
                  })
                } else {
                  let cmd = Command::new("pactl")
                    .arg("set-sink-mute")
                    .arg("@DEFAULT_SINK@")
                    .arg("toggle")
                    .output()
                    .unwrap();
                  log_cmd(&cmd);
                }
              }
              h0060::Command::InvalidCommand => {
                println!("ERROR: InvalidCommand");
              }
            }
          } else {
            println!("Unknown: {}", out);
          }
        } else {
          println!("Cli");
        }
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Kll(_) => {
        println!("Kll");
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Layer(_) => {
        println!("Layer");
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::HostMacro(_) => {
        println!("HostMacro");
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Manufacturing(_) => {
        println!("Manufacturing");
      }
      #[allow(unreachable_patterns)]
      _ => {
        println!("Unknown signal");
      }
    }
    Promise::ok(())
  }
}

#[tokio::main]
pub async fn main() -> Result<(), capnp::Error> {
  setup_logging_lite().ok();
  tokio::task::LocalSet::new().run_until(try_main()).await
}

async fn try_main() -> Result<(), capnp::Error> {
  // Prepare hid-io-core connection
  let mut hidio_conn = hid_io_client::HidioConnection::new().unwrap();
  let mut rng = rand::thread_rng();

  // Serial is used for automatic reconnection if hid-io goes away and comes back
  let serial = "".to_string();

  loop {
    // Connect and authenticate with hid-io-core
    let (hidio_auth, _hidio_server) = hidio_conn
      .connect(
        hid_io_client::AuthType::Priviledged,
        NodeType::HidioApi,
        "HID-IO ErgoOne".to_string(),
        format!("{:x} - pid:{}", rng.gen::<u64>(), std::process::id()),
        true,
        std::time::Duration::from_millis(1000),
      )
      .await?;
    let hidio_auth = hidio_auth.expect("Could not authenticate to hid-io-core");

    let nodes_resp = {
      let request = hidio_auth.nodes_request();
      request.send().promise.await.unwrap()
    };
    let nodes = nodes_resp.get()?.get_nodes()?;

    let args: Vec<_> = std::env::args().collect();
    let nid = match args.get(1) {
      Some(n) => n.parse().unwrap(),
      None => {
        let id;

        let serial_matched: Vec<_> =
          nodes.iter().filter(|n| n.get_serial().unwrap() == serial).collect();
        // First attempt to match serial number
        if !serial.is_empty() && serial_matched.len() == 1 {
          let n = serial_matched[0];
          println!("Re-registering to {}", hid_io_client::format_node(n));
          id = n.get_id();
        } else {
          let keyboards: Vec<_> = nodes
            .iter()
            .filter(|n| {
              n.get_type().unwrap() == NodeType::UsbKeyboard
                || n.get_type().unwrap() == NodeType::BleKeyboard
            })
            .collect();

          // Next, if serial number is unset and there is only one keyboard, automatically attach
          if serial.is_empty() && keyboards.len() == 1 {
            let n = keyboards[0];
            println!("Registering to {}", hid_io_client::format_node(n));
            id = n.get_id();
          // Otherwise display a list of keyboard nodes
          } else {
            println!();
            for n in keyboards {
              println!(" * {} - {}", n.get_id(), hid_io_client::format_node(n));
            }

            print!("Please choose a device: ");
            std::io::stdout().flush()?;

            let mut n = String::new();
            std::io::stdin().read_line(&mut n)?;
            id = n.trim().parse().unwrap();
          }
        }
        id
      }
    };

    let device = nodes.iter().find(|n| n.get_id() == nid);
    if device.is_none() {
      eprintln!("Could not find node: {}", nid);
      std::process::exit(1);
    }
    let device = device.unwrap();
    // serial = device.get_serial().unwrap().to_string();

    // Build subscription callback
    let subscription = capnp_rpc::new_client(KeyboardSubscriberImpl::default());

    // Subscribe to cli messages
    let subscribe_req = {
      let node = match device.get_node().which().unwrap() {
        hid_io_client::common_capnp::destination::node::Which::Keyboard(n) => n.unwrap(),
        hid_io_client::common_capnp::destination::node::Which::Daemon(_) => {
          std::process::exit(1);
        }
      };
      let mut request = node.subscribe_request();
      let mut params = request.get();
      params.set_subscriber(subscription);

      // Build list of options
      params
        .init_options(1)
        .get(0)
        .set_type(keyboard_capnp::keyboard::SubscriptionOptionType::Volume);
      request
    };
    let _callback = subscribe_req.send().promise.await.unwrap();

    println!("READY");
    let (vt_tx, mut vt_rx) = tokio::sync::mpsc::channel::<u8>(100);
    std::thread::spawn(move || loop {
      #[allow(clippy::significant_drop_in_scrutinee)]
      for byte in std::io::stdin().lock().bytes() {
        if let Ok(b) = byte {
          if let Err(e) = vt_tx.blocking_send(b) {
            println!("Restarting stdin loop: {}", e);
            return;
          }
        } else {
          println!("Lost stdin");
          std::process::exit(2);
        }
      }
    });

    loop {
      let mut vt_buf = vec![];
      // Await the first byte
      match vt_rx.recv().await {
        Some(c) => {
          vt_buf.push(c);
        }
        None => {
          println!("Lost socket");
          ::std::process::exit(1);
        }
      }
      // Loop over the rest of the buffer
      loop {
        match vt_rx.try_recv() {
          Ok(c) => {
            vt_buf.push(c);
          }
          Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
            // Done, can begin sending cli message to device
            break;
          }
          Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
            println!("Lost socket (buffer)");
            ::std::process::exit(1);
          }
        }
      }
    }
  }
}
