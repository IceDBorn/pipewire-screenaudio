extern crate env_logger;

extern crate json;
use json::{object,JsonValue};

mod io;

fn main() {
  env_logger::init();

  let result = io::read().unwrap();

  let _ = io::write(JsonValue::from(object!{
    "test": "poutsa",
    "cmd": result.command,
    "args": result.arguments["size"].as_str().unwrap(),
  }));
}
