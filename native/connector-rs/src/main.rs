extern crate env_logger;

mod command;
mod helpers;

use helpers::io;

fn main() {
  env_logger::init();

  let payload = io::read().unwrap();
  let result = command::run(payload).unwrap();
  let _ = io::write(result);
}
