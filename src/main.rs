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

use std::io::Write;

mod args;
mod commands;
mod json;
mod keysub;
mod modules;
mod util;

use hid_io_client::capnp;
use hid_io_client::capnp_rpc;
use hid_io_client::common_capnp::NodeType;
use hid_io_client::keyboard_capnp;
use hid_io_client::setup_logging_lite;
use rand::Rng;

#[tokio::main]
pub async fn main() -> Result<(), capnp::Error> {
  setup_logging_lite().ok();
  let matches = args::cli().get_matches();
  tokio::task::LocalSet::new().run_until(try_main(matches)).await
}

async fn try_main(matches: clap::ArgMatches) -> Result<(), capnp::Error> {
  // Prepare hid-io-core connection
  let mut hidio_conn = hid_io_client::HidioConnection::new().unwrap();
  let mut rng = rand::thread_rng();

  // Serial is used for automatic reconnection if hid-io goes away and comes back
  let mut serial = "".to_string();

  loop {
    // Connect and authenticate with hid-io-core
    let (hidio_auth, hidio_server) = hidio_conn
      .connect(
        hid_io_client::AuthType::Priviledged,
        NodeType::HidioApi,
        "HID-IO Keyboard".to_string(),
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

    let find_kb = |serial_arg: Option<&String>, name_arg: Option<&String>| -> String {
      match serial_arg {
        Some(n) => {
          println!("Serial specified: {}", n);
          n.to_owned()
        }
        None => {
          let mut ser: String = "".to_string();

          let matched: Vec<_>;
          if name_arg.is_some() && serial_arg.is_some() {
            matched = nodes
              .iter()
              .filter(|n| {
                n.get_serial().unwrap() == serial && n.get_name().unwrap() == name_arg.unwrap()
              })
              .collect();
          } else if name_arg.is_some() {
            matched = nodes.iter().filter(|n| n.get_name().unwrap() == name_arg.unwrap()).collect();
          } else {
            matched = nodes.iter().filter(|n| n.get_serial().unwrap() == serial).collect();
          }
          // First attempt to match serial number
          if !serial.is_empty() && matched.len() == 1 {
            let n = matched[0];
            println!("Re-registering to {}", hid_io_client::format_node(n));
            ser = matched[0].get_serial().unwrap().to_owned();
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
              ser = n.get_serial().unwrap().to_owned();
              // Otherwise display a list of keyboard nodes
            }
          }
          ser
        }
      }
    };

    // Handle Args
    match matches.subcommand() {
      Some(("list", _)) => {
        let keyboards: Vec<_> = nodes
          .iter()
          .filter(|n| {
            n.get_type().unwrap() == NodeType::UsbKeyboard
              || n.get_type().unwrap() == NodeType::BleKeyboard
          })
          .collect();
        for n in keyboards {
          // TODO: Fix formatting
          println!("{}", hid_client_stdout::stdout_from_node(n));
        }
        std::process::exit(0);
      }
      Some(("exec", sub_matches)) => {
        use crate::commands::Commands;
        let command: Option<Commands>;
        let serial_arg = sub_matches.try_get_one::<String>("serial").unwrap();
        let name_arg = sub_matches.try_get_one::<String>("name").unwrap();
        serial = find_kb(serial_arg.clone(), name_arg.clone());
        match sub_matches.subcommand() {
          Some(("LayerSet", sub_matches1)) => {
            let layer_arg = sub_matches1.get_one::<String>("LAYER").unwrap();
            println!("exec LayerSet: {}", layer_arg);

            let layer = layer_arg.parse::<u8>().unwrap();
            command = Some(Commands::LayerSet(layer));
          }
          _ => unreachable!(),
        }
        if command.is_some() {
          let device = nodes.iter().find(|n| {
            println!("Found: {}", n.get_serial().unwrap());
            n.get_serial().unwrap() == serial
          });
          if device.is_none() {
            eprintln!("Could not find node: {}", serial);
            std::process::exit(1);
          }
          let device = device.unwrap();
          // serial = device.get_serial().unwrap().to_string();
          println!("READY");

          let node = match device.get_node().which().unwrap() {
            hid_io_client::common_capnp::destination::node::Which::Keyboard(n) => n.unwrap(),
            hid_io_client::common_capnp::destination::node::Which::Daemon(_) => {
              std::process::exit(1);
            }
          };

          let node = hid_io_core::hidio_capnp::node::Client {
            client: node.client,
          };
          let mut request;
          match command.unwrap() {
            Commands::LayerSet(layer) => {
              request = node.layer_set_command_request();
              request.get().set_layer(layer.into());
              let _callback = request.send().promise.await.unwrap();
              println!("LayerSet: {} request sent", layer);
              std::process::exit(0);
            }
          }
        } else {
          println!("Command does not exist");
          std::process::exit(1);
        }
      }
      Some(("subscribe", sub_matches)) => {
        let serial_arg = sub_matches.try_get_one::<String>("serial").unwrap();
        let name_arg = sub_matches.try_get_one::<String>("name").unwrap();
        println!("Calling out to subscribe with {:?}, {:?}", serial_arg, name_arg);

        serial = find_kb(serial_arg.clone(), name_arg.clone());

        let device = nodes.iter().find(|n| {
          println!("Found: {}", n.get_serial().unwrap());
          n.get_serial().unwrap() == serial
        });
        if device.is_none() {
          eprintln!("Could not find node: {}", serial);
          std::process::exit(1);
        }
        let device = device.unwrap();
        // serial = device.get_serial().unwrap().to_string();

        // Build subscription callback
        let subscription = capnp_rpc::new_client(keysub::KeyboardSubscriberImpl::default());

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
          let options = params.init_options(1);
          options.get(0).set_type(keyboard_capnp::keyboard::SubscriptionOptionType::Volume);
          request
        };
        let _callback = subscribe_req.send().promise.await.unwrap();

        println!("READY");
        loop {
          tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

          // Check if the server is still alive
          let request = hidio_server.alive_request();
          if let Err(e) = request.send().promise.await {
            println!("Dead: {}", e);
            // Break the subscription loop and attempt to reconnect
            break;
          }
        }
      }
      _ => {
        eprintln!("Unknown command\nTry --help for a list of commands");
        std::process::exit(1);
      }
    }
  }
}
