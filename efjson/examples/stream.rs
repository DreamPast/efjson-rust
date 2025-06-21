use efjson::{ParserOption, stream_parser::StreamParser};

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

fn main() {
  test_stream();
  perf();
}
