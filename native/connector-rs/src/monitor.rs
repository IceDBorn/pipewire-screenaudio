use std::{
  io,
  rc::Rc,
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
  pub fn launch_monitor_thread(
    mic_ports: Ports,
    instance_identifier: Option<String>,
  ) -> Result<Self> {
    tracing::debug!("starting monitor thread");
    let (controller, signal) = CancellationSignal::pair();

    let join_handle = thread::Builder::new()
      .name("monitor and connector thread".to_owned())
      .spawn(move || {
        let client = PipewireClient::new()?;
        let instance_identifier = instance_identifier.map(Rc::new);
        client.monitor_and_connect_nodes(mic_ports, signal, move |node_app_name| {
          tracing::trace!(
            node_app_name,
            instance_identifier = instance_identifier.as_ref().map(|ii| ii.as_str()),
            "filtering"
          );
          instance_identifier
            .as_ref()
            .is_none_or(move |excluded_node_name| {
              node_app_name
                .is_none_or(|node_app_name| !node_app_name.ends_with(excluded_node_name.as_ref()))
            })
        })
      })
      .map_err(Error::ThreadSpawnError)?;

    Ok(MonitorThreadHandle {
      join_handle: Some(join_handle),
      cancellation_controller: controller,
    })
  }

  pub fn stop(&mut self) {
    tracing::debug!("stopping monitor thread");
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
