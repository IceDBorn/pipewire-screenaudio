use std::{
  env,
  fs::{self, File},
  io,
  os::unix::net::{UnixListener, UnixStream},
  path::PathBuf,
  thread,
  time::Duration,
};

use thiserror::Error;

use crate::dirs::get_runtime_path;

fn get_ipc_socket_path() -> PathBuf {
  let mut path = get_runtime_path();
  path.push("ipc.sock");
  path
}

pub fn listen() -> io::Result<UnixListener> {
  let path = get_ipc_socket_path();
  let _ = fs::remove_file(&path);
  UnixListener::bind(path)
}

pub fn fake_connect() {
  let _ = UnixStream::connect(get_ipc_socket_path());
}

pub fn connect() -> io::Result<UnixStream> {
  let mut retries = 5;
  let path = get_ipc_socket_path();
  let mut last_error = None;
  while retries > 0 {
    tracing::debug!("connecting");
    match UnixStream::connect(path.clone()) {
      Ok(socket) => return Ok(socket),
      Err(err) => last_error = Some(err),
    }
    thread::sleep(Duration::from_millis(100));
  }
  Err(last_error.unwrap())
}
