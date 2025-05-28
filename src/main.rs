use efjson_rust::{JsonOption, json_stream_parse};
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
  let x = json_stream_parse(&s, JsonOption::default());
  println!("{} {}", s.len(), x.unwrap().len());
  let duration = start.elapsed();
  println!("运行时间: {:?}", duration);
}

fn main() {
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
  println!("{}", SRC);
  for item in json_stream_parse(SRC, JsonOption::default()).unwrap() {
    println!("{:?}", item);
  }
  perf();
}
