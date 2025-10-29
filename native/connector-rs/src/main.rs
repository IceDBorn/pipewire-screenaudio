use std::{
  env,
  error::Error,
  fs::File,
  io::{stdin, stdout},
  os,
  path::{Path, PathBuf},
};

use env_logger::{Builder, Target};

mod command;
mod daemon;
mod helpers;
mod ipc;
mod ipc_request;
mod monitor;

use helpers::io;

use crate::daemon::monitor_and_connect_nodes;

fn main() -> Result<(), Box<dyn Error>> {
  let mut builder = Builder::from_default_env();
  builder.target(Target::Stderr);
  builder.init();
  let mut args = std::env::args();
  let process = args.next().expect("binary path should be the first arg");
  let subcommand = args.next();

  match subcommand.as_ref().map(|s| s.as_str()) {
    Some("daemon") => {
      if let Err(err) = monitor_and_connect_nodes() {
        log::error!("error: {}", err);
        Err(Box::new(err))?;
      }
    }
    Some(subcommand) => {
      log::error!("invalid subcommand {subcommand}");
      Err("invalid subcommand")?;
    }
    None => {
      let payload = io::read(stdin()).unwrap();
      let result = command::run(payload).unwrap();
      let _ = io::write(result, stdout());
    }
  }

  Ok(())
}
