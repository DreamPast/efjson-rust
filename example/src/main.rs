use std::collections::HashMap;

use efjson::{
  ParserOption,
  deserialize::{JsonRawString, JsonRawToken, deserialize},
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
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[1,.1,1.,1.234e3]"));
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[0x1234,0o1234,0b1011]"));
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[Infinity,-Infinity,NaN]"));
  println!("{:?}", deserialize::<Vec<i64>>(ParserOption::all(), "[-0x1234,0o1234,+0b1011]"));
  println!(
    "{:?}\t{:?}",
    deserialize::<Vec<bool>>(ParserOption::all(), "[true,false]"),
    deserialize::<()>(ParserOption::all(), "null")
  );
  println!(
    "{:?}\t{:?}\t{:?}\t{:?}",
    deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4]"),
    deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4,]"),
    deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,]"),
    deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4,5]")
  );
  println!(
    "{:?}",
    deserialize::<HashMap<String, Option<i32>>>(ParserOption::all(), "{'a':1,'b':null,}")
  );

  println!("{:?}", deserialize::<JsonRawToken>(ParserOption::all(), r#"{'a':12,b:[13,14]}"#));

  print!("{:?}\t", deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12]"#));
  print!("{:?}\t", deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,]"#));
  print!("{:?}\t", deserialize::<(String, i32)>(ParserOption::all(), r#"[]"#));
  print!("{:?}\t", deserialize::<(String, i32)>(ParserOption::all(), r#"["a",]"#));
  print!("{:?}\t", deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,13]"#));
  print!("\n");

  println!(
    "{:?}",
    deserialize::<HashMap<String, JsonRawString>>(
      ParserOption::all(),
      r#"{'a':1.2e3,'b':null,'c':[13,14],'d':{a:12,},}"#
    )
  );
  println!(
    "{:?}",
    deserialize::<Vec<JsonRawString>>(ParserOption::all(), r#"[1.2e3,null,[13,14,],{a:12,},]"#)
  );
}

fn main() {
  test_stream();
  test_event();
  test_deserializer();
  perf();
}
