use std::process::Output;

use capnp::traits::IntoInternalStructReader;
use hid_io_client::capnp;
use hid_io_client::capnp::capability::Promise;
use hid_io_client::capnp_rpc;
use hid_io_client::common_capnp::NodeType;
use hid_io_client::keyboard_capnp;

use crate::gui::HidIoGui;
use crate::modules::layer::handle_layer_event;
use crate::modules::volume::handle_volume;

#[derive(Clone, Copy, Debug)]
pub enum MSG {
  Layer(u16),
  Volume(u16),
}

pub fn log<T, E>(r: Result<T, E>)
where
  E: std::fmt::Display,
{
  match r {
    Err(e) => {
      println!("ERROR: {}", e);
    }
    _ => {}
  }
}

pub fn log_cmd(cmd: &Output) {
  if !cmd.status.success() {
    panic!(
      "ERROR: pamixer - {} -- {}",
      String::from_utf8(cmd.stderr.clone()).unwrap(),
      String::from_utf8(cmd.stdout.clone()).unwrap()
    );
  } else {
    if !cmd.stderr.is_empty() {
      println!("ERROR: pamixer - {}", String::from_utf8(cmd.stderr.clone()).unwrap(),);
    }
    if !cmd.stdout.is_empty() {
      println!("pamixer - {}", String::from_utf8(cmd.stdout.clone()).unwrap(),);
    }
  }
}

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
        let mut queue = unsafe { crate::HIDIO_QUEUE.lock().unwrap() };
        queue.push(MSG::Layer(l.get_layer()));
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
