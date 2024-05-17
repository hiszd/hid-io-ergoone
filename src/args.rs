use clap::{arg, Command};

impl From<crate::commands::Commands> for Command {
  fn from(msgs: crate::commands::Commands) -> Self {
    use crate::commands::Commands;
    match msgs {
      Commands::LayerSet(_) => Command::new("LayerSet")
        .about("Sets the current layer on the keyboard")
        .arg(arg!([LAYER] "The layer to set").required(true)),
    }
  }
}

pub fn cli() -> Command {
  Command::new("hidiokb")
    .about("Hidio Keyboard CLI")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(
      Command::new("subscribe")
        .about("Subscribes to a keyboard")
        .alias("sub")
        .arg(
          arg!(-s --serial <SERIAL> "The serial number of the keyboard")
            .required_unless_present("name"),
        )
        .arg(arg!(-n --name <NAME> "The name of the keyboard").required_unless_present("serial"))
        .arg_required_else_help(true),
    )
    .subcommand(
      Command::new("exec")
        .about("Executes a command on the keyboard")
        .subcommand_required(true)
        .arg(
          arg!(-s --serial <SERIAL> "The serial number of the keyboard")
            .required_unless_present("name"),
        )
        .arg(arg!(-n --name <NAME> "The name of the keyboard").required_unless_present("serial"))
        .subcommand(Command::from(crate::commands::Commands::LayerSet(0)))
        .arg_required_else_help(true),
    )
    .subcommand(Command::new("list").about("List all keyboard nodes"))
}
