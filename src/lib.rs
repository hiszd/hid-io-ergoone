use std::process::Output;

pub mod json;

pub fn log_cmd(cmd: &Output) {
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
