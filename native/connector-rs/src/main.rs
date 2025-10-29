use std::{
  env,
  error::Error,
  fs::File,
  io::{stdin, stdout},
  os,
  path::{Path, PathBuf},
};

mod command;
mod daemon;
mod dirs;
mod helpers;
mod ipc;
mod ipc_request;
mod monitor;

use helpers::io;
use tracing::Level;
use tracing_appender::{
  non_blocking,
  rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

use crate::{daemon::monitor_and_connect_nodes, dirs::get_runtime_path};

fn get_logs_path() -> PathBuf {
  let mut path = get_runtime_path();
  path.push("logs");
  path
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut args = std::env::args();
  let _ = args.next().expect("binary path should be the first arg");
  let subcommand = args.next();

  let file_appender = RollingFileAppender::builder()
    .rotation(Rotation::HOURLY)
    .filename_prefix(subcommand.clone().unwrap_or_else(|| "connector".to_owned()))
    .build(get_logs_path())
    .unwrap();
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let subscriber = Registry::default()
    .with(
      fmt::Layer::default()
        .with_writer(non_blocking)
        .with_ansi(false),
    )
    .with(fmt::Layer::default().with_writer(std::io::stderr));

  tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

  match subcommand.as_ref().map(|s| s.as_str()) {
    Some("daemon") => {
      if let Err(err) = monitor_and_connect_nodes() {
        tracing::error!("error: {}", err);
        Err(Box::new(err))?;
      }
    }
    Some(subcommand) => {
      tracing::error!("invalid subcommand {subcommand}");
      Err("invalid subcommand")?;
    }
    None => {
      tracing::info!("started connector");

      let payload = io::read(stdin()).unwrap();
      let result = command::run(payload).unwrap();
      let _ = io::write(result, stdout());
    }
  }

  Ok(())
}
