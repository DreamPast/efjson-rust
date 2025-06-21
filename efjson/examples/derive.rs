use std::collections::HashMap;

use efjson::{deserialize::deserialize, Deserializable, ParserOption};

const SRC: &'static str = r#"{
"n":null,"t":true,"f":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;

#[derive(Debug, Deserializable)]
#[allow(dead_code)]
struct Struct {
  n: (),
  t: bool,
  f: bool,
  str: String,
  num: f64,
  arr: (String, HashMap<String, String>),
}

fn main() {
  let r: Result<Struct, _> = deserialize(ParserOption::all(), SRC);
  println!("{:#?}", r);
}
