use std::collections::HashMap;

use efjson::{
  ParserOption,
  deserialize::{deserialize},
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
  let x = StreamParser::parse(ParserOption::default(), &s);
  println!("{} {}", s.len(), x.unwrap().len());
  let duration = start.elapsed();
  println!("运行时间: {:?}", duration);
}

const SRC: &'static str = r#"{
"N":null,"T":true,"F":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;

#[allow(dead_code)]
fn test_stream() {
  let tokens = StreamParser::parse(ParserOption::default(), SRC).unwrap();
  for item in tokens {
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
  EventParser::parse(receiver, ParserOption::make_json5(), SRC).unwrap();
}

fn test_deserializer() {
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "1"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), ".1"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "1."));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "1.234e3"));
  print!("\n");

  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "0x1234"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "0o1234"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "0b1011"));
  print!("\n");

  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "Infinity"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "-Infinity"));
  print!("{:?}\t", deserialize::<f64>(ParserOption::all(), "NaN"));
  print!("\n");

  print!("{:?}\t", deserialize::<i64>(ParserOption::all(), "-0x1234"));
  print!("{:?}\t", deserialize::<i64>(ParserOption::all(), "0o1234"));
  print!("{:?}\t", deserialize::<i64>(ParserOption::all(), "+0b1011"));
  print!("\n");

  print!("{:?}\t", deserialize::<bool>(ParserOption::all(), "true"));
  print!("{:?}\t", deserialize::<bool>(ParserOption::all(), "false"));
  print!("{:?}\t", deserialize::<()>(ParserOption::all(), "null"));
  print!("\n");

  println!("{:?}\t", deserialize::<Vec<i32>>(ParserOption::all(), "[1,2,3,4]"));
  println!(
    "{:?}\t",
    deserialize::<HashMap<String, Option<i32>>>(ParserOption::all(), "{'a':1,'b':null}")
  );
  print!("\n");
}

fn main() {
  test_stream();
  test_event();
  test_deserializer();
  perf();
}
