use capnp::capability::Promise;
use capnp::traits::IntoInternalStructReader;
use hid_io_client::capnp_rpc;
use hid_io_core::keyboard_capnp;

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
        let app_msg = match app_raw.len() {
          0 => None,
          _ => Some(app_raw.to_string()),
        };
        let msg = hid_client_stdout::Messages::Volume(cmd, vol, app_msg);
        let str = String::try_from(msg).unwrap();
        println!("{}", str);
        // handle_volume(cmd, vol, app);
      }
      hid_io_client::keyboard_capnp::keyboard::signal::data::Which::LayerChanged(l) => {
        let l = l.unwrap();
        let msg = hid_client_stdout::Messages::LayerChanged(l.get_layer());
        let str = String::try_from(msg).unwrap();
        println!("{}", str);
      }
      #[allow(unreachable_patterns)]
      _ => {
        println!("Unknown signal");
      }
    }
    Promise::ok(())
  }
}
