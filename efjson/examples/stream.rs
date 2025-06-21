use efjson::{stream_parser::StreamParser, ParserOption};

const SRC: &'static str = r#"{
"N":null,"T":true,"F":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;
fn test_stream() {
  let tokens = StreamParser::parse(ParserOption::default(), SRC).unwrap();
  for item in tokens {
    println!("{:?}", item);
  }
}

fn main() {
  test_stream();
}
