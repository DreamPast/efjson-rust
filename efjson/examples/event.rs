use efjson::{
  event_parser::{EventObjectReceiver, EventParser, EventReceiver},
  ParserOption,
};

const SRC: &'static str = r#"{
"N":null,"T":true,"F":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;
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

fn main() {
  test_event();
}
