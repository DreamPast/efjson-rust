use efjson::{
  JsonParserOption,
  event_parser::{EventObjectReceiver, EventParser, EventReceiver},
  stream_parser::StreamParser,
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
  let x = StreamParser::parse(JsonParserOption::default(), &s);
  println!("{} {}", s.len(), x.unwrap().len());
  let duration = start.elapsed();
  println!("运行时间: {:?}", duration);
}

const SRC: &'static str = r#"{
"null":null,"true":true,"false":false,
"string":"string,\"escape\",\uD83D\uDE00,😊",
"integer":12,"negative":-12,"fraction":12.34,"exponent":1.234e2,
"array":["1st element",{"object":"nesting"}],
"object":{"1st":[],"2st":{}}
}"#;

#[allow(dead_code)]
fn test_stream() {
  for item in StreamParser::parse(JsonParserOption::default(), SRC).unwrap() {
    println!("{:?}", item);
  }
}

#[allow(dead_code)]
fn test_event() {
  let receiver = EventReceiver {
    object: EventObjectReceiver {
      set: Some(Box::new(|k, v| {
        println!("{}: {:?}", k, v);
      })),
      ..Default::default()
    },
    ..EventReceiver::new_all()
  };
  EventParser::parse(receiver, JsonParserOption::new_json5(), SRC).unwrap();
}

fn main() {
  test_stream();
  test_event();
  perf();
}
