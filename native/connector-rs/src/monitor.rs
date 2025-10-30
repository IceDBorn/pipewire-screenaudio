use std::{
  io,
  thread::{self, JoinHandle},
};

use pipewire_utils::{
  cancellation_signal::{CancellationController, CancellationSignal},
  PipewireClient, PipewireError, Ports,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("pipewire error")]
  PipewireError(#[from] PipewireError),
  #[error("thread spawning error")]
  ThreadSpawnError(io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct MonitorThreadHandle {
  join_handle: Option<JoinHandle<Result<(), PipewireError>>>,
  cancellation_controller: CancellationController,
}

impl MonitorThreadHandle {
  pub fn launch_monitor_thread(mic_ports: Ports) -> Result<Self> {
    let (controller, signal) = CancellationSignal::pair();

    tracing::info!("launching monitor thread");
    let join_handle = thread::Builder::new()
      .name("monitor and connector thread".to_owned())
      .spawn(move || {
        let client = PipewireClient::new()?;
        client.monitor_and_connect_nodes(mic_ports, signal)
      })
      .map_err(Error::ThreadSpawnError)?;

    Ok(MonitorThreadHandle {
      join_handle: Some(join_handle),
      cancellation_controller: controller,
    })
  }

  pub fn stop(&mut self) {
    tracing::info!("stopping monitor thread");
    self.cancellation_controller.cancel();
    if let Some(handle) = self.join_handle.take() {
      if let Err(err) = handle.join().unwrap() {
        tracing::error!("monitor thread returned error: {err}");
      }
    }
  }
}

impl Drop for MonitorThreadHandle {
  fn drop(&mut self) {
    self.stop();
  }
}
