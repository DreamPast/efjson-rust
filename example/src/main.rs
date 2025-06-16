use std::{collections::HashMap, vec};

use efjson::{
  JsonValue, ParserOption,
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
  println!("Time: {:?}", duration);
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
    "{:?}",
    vec![
      deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4]"),
      deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4,]"),
      deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,]"),
      deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4,5]")
    ]
  );
  println!(
    "{:?}",
    deserialize::<HashMap<String, Option<i32>>>(ParserOption::all(), "{'a':1,'b':null,}")
  );

  {
    println!("[");
    for token in
      deserialize::<JsonRawToken>(ParserOption::all(), r#"{'a':12,b:[13,14]}"#).unwrap().tokens
    {
      println!("    {:?}", token);
    }
    println!("]");
  }

  println!(
    "{:?}\t",
    vec![
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"[]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,13]"#)
    ]
  );

  println!(
    "{:?}",
    deserialize::<HashMap<String, JsonRawString>>(
      ParserOption::all(),
      r#"{'a':1.2e3,'b':null,'c':"str",'d':[13,14],'e':{a:12,},}"#
    )
    .unwrap()
    .iter()
    .map(|(k, v)| (k, &v.json))
    .collect::<HashMap<_, _>>()
  );
  println!(
    "{:?}",
    deserialize::<Vec<JsonRawString>>(
      ParserOption::all(),
      r#"[1.2e3,null,"str",[13,14,],{a:12,},]"#
    )
    .unwrap()
    .iter_mut()
    .map(|x| &x.json)
    .collect::<Vec<_>>()
  );
  println!(
    "{:?}",
    deserialize::<JsonValue>(ParserOption::all(), r#"[1.2e3,null,"str",[13,14,],{"a":12,},]"#)
      .unwrap()
  );
}

fn main() {
  println!("======Stream Parser======");
  test_stream();
  perf();
  println!("======Event Parser======");
  test_event();
  println!("======Deserializer======");
  test_deserializer();
}
