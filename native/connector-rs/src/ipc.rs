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

fn get_runtime_path() -> PathBuf {
  env::var("XDG_RUNTIME_DIR")
    .unwrap_or("/tmp".to_owned())
    .into()
}

fn get_ipc_socket_path() -> PathBuf {
  let mut path = get_runtime_path();
  path.push("pipewire-screenaudio/ipc.sock");
  path
}

pub fn listen() -> io::Result<UnixListener> {
  let path = get_ipc_socket_path();
  let _ = fs::remove_file(&path);
  UnixListener::bind(path)
}

pub fn connect() -> io::Result<UnixStream> {
  let mut retries = 50;
  let path = get_ipc_socket_path();
  let mut last_error = None;
  while retries > 0 {
    match UnixStream::connect(path.clone()) {
      Ok(socket) => return Ok(socket),
      Err(err) => last_error = Some(err),
    }
    thread::sleep(Duration::from_millis(100));
  }
  Err(last_error.unwrap())
}
