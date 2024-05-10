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

use capnp::traits::IntoInternalStructReader;
use hid_io_client::capnp;
use hid_io_client::capnp::capability::Promise;
use hid_io_client::capnp_rpc;
use hid_io_client::common_capnp::NodeType;
use hid_io_client::keyboard_capnp;
use hid_io_client::setup_logging_lite;
use hid_io_ergoone::gui::HidIoGui;
use hid_io_ergoone::modules::layer::handle_layer_event;
use hid_io_ergoone::modules::volume::handle_volume;
use iced::Application;
use iced::Font;
use iced::Settings;
use rand::Rng;
use tokio::task::JoinError;

static mut HIDIO_MSG_TX: Option<std::sync::mpsc::Sender<String>> = None;
static mut HIDIO_MSG_RX: Option<std::sync::mpsc::Receiver<String>> = None;

#[derive(Default)]
pub struct KeyboardSubscriberImpl {}

impl keyboard_capnp::keyboard::subscriber::Server for KeyboardSubscriberImpl {
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
    // println!("{:?}", data);
    match params.which().unwrap() {
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::Volume(v) => {
        let v = v.unwrap();
        let cmd = v.get_cmd().unwrap();
        let vol = v.get_vol();
        let app_raw = v.get_app().unwrap();
        let app = match app_raw.len() {
          0 => None,
          _ => Some(app_raw),
        };
        println!("Volume: cmd: {:?}, vol: {}, app: {}", cmd, vol, app_raw);
        handle_volume(cmd, vol, app);
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::LayerChanged(l) => {
        let l = l.unwrap();
        println!("LayerChanged: {}", l.get_layer());
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
pub async fn main() -> Result<(), iced::Error> {
  let (tx, mut rx) = tokio::sync::broadcast::channel(1);
  unsafe {
    HIDIO_MSG_TX = Some(tx);
    HIDIO_MSG_RX = Some(rx);
  }
  setup_logging_lite().ok();
  let gui = tokio::spawn(async move {
    HidIoGui::run(Settings {
      default_font: Font::MONOSPACE,
      ..Settings::default()
    })
  });
  let hid = tokio::spawn(try_main());
  gui.await.unwrap()
}

async fn try_main() -> Result<(), capnp::Error> {
  // Prepare hid-io-core connection
  let mut hidio_conn = hid_io_client::HidioConnection::new().unwrap();
  let mut rng = rand::thread_rng();

  // Serial is used for automatic reconnection if hid-io goes away and comes back
  let serial = "".to_string();

  loop {
    // Connect and authenticate with hid-io-core
    let (hidio_auth, hidio_server) = hidio_conn
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
    let serial: String = match args.get(1) {
      Some(n) => {
        println!("Serial specified: {}", n);
        n.to_owned()
      }
      None => {
        let ser: String;

        let serial_matched: Vec<_> =
          nodes.iter().filter(|n| n.get_serial().unwrap() == serial).collect();
        // First attempt to match serial number
        if !serial.is_empty() && serial_matched.len() == 1 {
          let n = serial_matched[0];
          println!("Re-registering to {}", hid_io_client::format_node(n));
          ser = serial_matched[0].get_serial().unwrap().to_owned();
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
          } else {
            println!();
            for n in keyboards {
              println!(" * {} - {}", n.get_serial().unwrap(), hid_io_client::format_node(n));
            }

            print!("Please choose a device: ");
            std::io::stdout().flush()?;

            let mut n = String::new();
            std::io::stdin().read_line(&mut n)?;
            ser = n.trim().to_owned();
          }
        }
        ser
      }
    };

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
    let subscription = capnp_rpc::new_client(KeyboardSubscriberImpl::default());

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
}
