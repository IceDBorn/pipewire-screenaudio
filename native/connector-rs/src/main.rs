extern crate env_logger;

extern crate json;
use json::{object,JsonValue};

mod command;
mod io;

fn main() {
  env_logger::init();

  let payload = io::read().unwrap();
  let result = command::run(payload).unwrap();
  let _ = io::write(result);
}
