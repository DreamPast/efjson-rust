use efjson::{
  JsonEventObjectReceiver, JsonEventParser, JsonEventReceiver, JsonOption, JsonStreamParser,
};

#[allow(dead_code)]
fn perf() {
  use std::time::Instant;
  let mut s1 = String::new();
  s1 += "[";
  s1 += &"100,".repeat(1000);
  s1.pop();
  s1 += "],";
  let mut s = String::new();
  s += "[";
  s += &s1.repeat(1000);
  s.pop();
  s += "]";

  let start = Instant::now();
  let x = JsonStreamParser::parse(JsonOption::default(), &s);
  println!("{} {}", s.len(), x.unwrap().len());
  let duration = start.elapsed();
  println!("运行时间: {:?}", duration);
}

const SRC: &'static str = r#"{
  "null": null,
  "true": true,
  "false": false,

  "string": "string",
  "string_with_escape": "string with \"escape\"",
  "string_with_unicode_escape": "string with \uD83D\uDE00",
  "string_with_unicode": "string with 😊",

  "integer": 1234,
  "negative": -1234,
  "number": 1234.5678,
  "number_with_exponent": 1.234e2,

  "array": [
    "this is the first element",
    {
      "object": "a nesting object"
    }
  ],
  "object": {
    "1st": [],
    "2st": {}
  }
}"#;

#[allow(dead_code)]
fn test_stream() {
  for item in JsonStreamParser::parse(JsonOption::default(), SRC).unwrap() {
    println!("{:?}", item);
  }
}

#[allow(dead_code)]
fn test_event() {
  let receiver = JsonEventReceiver {
    object: JsonEventObjectReceiver {
      set: Some(Box::new(|k, v| {
        println!("{}: {:?}", k, v);
      })),
      ..Default::default()
    },
    ..JsonEventReceiver::new_all()
  };
  JsonEventParser::parse(receiver, JsonOption::new_json5(), SRC).unwrap();
}

fn main() {
  test_stream();
  test_event();
  perf();
}
