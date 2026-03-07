use std::{env, path::PathBuf};

pub fn get_runtime_path() -> PathBuf {
  let mut path: PathBuf = env::var("XDG_RUNTIME_DIR")
    .unwrap_or("/tmp".to_owned())
    .into();

  path.push("pipewire-screenaudio");

  path
}
