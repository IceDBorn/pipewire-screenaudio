use std::{
  env,
  error::Error,
  io::{stdin, stdout},
  panic,
  path::PathBuf,
  process,
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
use tracing_appender::rolling::RollingFileAppender;
use tracing_panic::panic_hook;
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
            .with_default_directive(LevelFilter::ERROR.into())
            .from_env_lossy(),
        ),
    );

  tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");
  let span = tracing::info_span!("main", pid = process::id());
  let _span_handle = span.enter();

  panic::set_hook(Box::new(panic_hook));

  match subcommand.as_deref() {
    Some("daemon") => {
      if let Err(err) = monitor_and_connect_nodes() {
        tracing::error!("error: {}", err);
        Err(Box::new(err))?;
      }
    }
    Some(_) | None => {
      let payload = io::read(stdin()).unwrap();
      tracing::info!(payload = format!("{payload:?}"), "running connector");
      match command::run(payload) {
        Ok(result) => {
          let _ = io::write(result, stdout());
        }
        Err(err) => {
          tracing::error!("command error: {}", err);
          let _ = io::write(err, stdout());
        }
      }
    }
  }

  Ok(())
}
