use std::{
  env,
  error::Error,
  io::{stdin, stdout},
  path::PathBuf,
};

mod command;
mod daemon;
mod dirs;
mod helpers;
mod ipc;
mod ipc_request;
mod monitor;

use helpers::io;
use tracing::{level_filters::LevelFilter, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

use crate::{daemon::monitor_and_connect_nodes, dirs::get_runtime_path};

fn get_logs_path() -> PathBuf {
  let mut path = get_runtime_path();
  path.push("logs");
  path
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut args = env::args();
  let _ = args.next().expect("binary path should be the first arg");
  let subcommand = args.next();

  let file_appender = RollingFileAppender::builder()
    .rotation(Rotation::HOURLY)
    .filename_prefix(
      match subcommand.as_deref() {
        Some("daemon") => "daemon",
        _ => "connector",
      }
      .to_owned(),
    )
    .build(get_logs_path())
    .unwrap();
  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
  let subscriber = Registry::default()
    .with(
      fmt::Layer::default()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(LevelFilter::from_level(Level::DEBUG)),
    )
    .with(
      fmt::Layer::default()
        .with_writer(std::io::stderr)
        .with_filter(
          EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
        ),
    );

  tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

  match subcommand.as_deref() {
    Some("daemon") => {
      if let Err(err) = monitor_and_connect_nodes() {
        tracing::error!("error: {}", err);
        Err(Box::new(err))?;
      }
    }
    Some(_) | None => {
      tracing::info!("started connector");

      let payload = io::read(stdin()).unwrap();
      let result = command::run(payload).unwrap();
      let _ = io::write(result, stdout());
    }
  }

  Ok(())
}
