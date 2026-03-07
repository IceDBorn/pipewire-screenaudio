use std::{
  error::Error,
  fs, io,
  os::unix::net::{UnixListener, UnixStream},
  path::{Path, PathBuf},
  thread,
  time::Duration,
};

use crate::io as ipc_io;
use crate::{daemon, dirs::get_runtime_path};

fn get_ipc_socket_path() -> PathBuf {
  let mut path = get_runtime_path();
  path.push("ipc.sock");
  path
}

fn ensure_stopped(path: &Path) -> Result<(), Box<dyn Error>> {
  if path.exists() {
    let stream = connect_inner(1)?;
    ipc_io::write(daemon::Command::Stop, &stream)?;
  }
  Ok(())
}

pub fn listen() -> io::Result<UnixListener> {
  let path = get_ipc_socket_path();
  let _ = ensure_stopped(&path);
  let _ = fs::remove_file(&path);
  UnixListener::bind(path)
}

pub fn fake_connect() {
  let _ = UnixStream::connect(get_ipc_socket_path());
}

pub fn connect_inner(tries: usize) -> io::Result<UnixStream> {
  let mut retries = tries;
  let path = get_ipc_socket_path();
  let mut last_error = None;
  while retries > 0 {
    tracing::debug!("connecting");
    match UnixStream::connect(path.clone()) {
      Ok(socket) => return Ok(socket),
      Err(err) => last_error = Some(err),
    }
    retries -= 1;
    if retries > 0 {
      thread::sleep(Duration::from_millis(100));
    }
  }
  Err(last_error.unwrap())
}

pub fn connect() -> io::Result<UnixStream> {
  connect_inner(5)
}
