extern crate env_logger;
use env_logger::{Builder, Target};

mod command;
mod helpers;

use helpers::io;

fn main() {
  let mut builder = Builder::from_default_env();
  builder.target(Target::Stderr);
  builder.init();

  let payload = io::read().unwrap();
  let result = command::run(payload).unwrap();
  let _ = io::write(result);
}
