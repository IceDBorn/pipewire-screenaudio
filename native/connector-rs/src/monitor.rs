use std::{
  error, io,
  thread::{self, JoinHandle},
};

use pipewire::{channel::Sender, context::ContextRc, main_loop::MainLoopRc};
use pipewire_utils::Ports;
use thiserror::Error;

use crate::helpers::pipewire::{self as pipewire_helpers, TerminateSignal};
use pipewire::Error as PWError;

#[derive(Error, Debug)]
pub enum Error {
  #[error("pipewire error")]
  PipewireError(#[from] pipewire::Error),
  #[error("thread spawning error")]
  ThreadSpawnError(io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct MonitorThreadHandle {
  join_handle: JoinHandle<Result<(), pipewire::Error>>,
  stop_signal_receiver: Sender<TerminateSignal>,
}

impl MonitorThreadHandle {
  pub fn launch_monitor_thread(mic_ports: Ports) -> Result<Self> {
    let (sender, receiver) = pipewire::channel::channel();

    log::info!("launching monitor thread");
    let join_handle = thread::Builder::new()
      .name("monitor and connector thread".to_owned())
      .spawn({ move || pipewire_helpers::monitor_and_connect_nodes(mic_ports, receiver) })
      .map_err(Error::ThreadSpawnError)?;

    Ok(MonitorThreadHandle {
      join_handle,
      stop_signal_receiver: sender,
    })
  }

  pub fn stop(self) {
    log::info!("stopping monitor thread");
    self.stop_signal_receiver.send(TerminateSignal).unwrap();
    self.join_handle.join().unwrap();
  }
}
