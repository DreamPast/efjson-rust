use efjson::{
  deserialize::{DefaultDeserializable, Deserializer},
  stream_parser::StreamParser,
  Deserializable, ParserOption,
};

type StringDeserializer = <String as DefaultDeserializable<String>>::DefaultDeserializer;

#[allow(dead_code)]
struct A(String);
impl Drop for A {
  fn drop(&mut self) {
    println!("[A] drop");
  }
}
struct ADeserializer(StringDeserializer);
impl Deserializer<A> for ADeserializer {
  fn feed_token(
    &mut self,
    token: efjson::stream_parser::Token,
  ) -> Result<efjson::deserialize::DeserResult<A>, efjson::deserialize::DeserError> {
    Ok(self.0.feed_token(token)?.map(|v| {
      println!("[A] new");
      A(v)
    }))
  }
}
impl Drop for ADeserializer {
  fn drop(&mut self) {
    println!("[ADeserializer] drop");
  }
}
impl DefaultDeserializable<A> for A {
  type DefaultDeserializer = ADeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    println!("[ADeserializer] new");
    ADeserializer(String::default_deserializer())
  }
}

#[allow(dead_code)]
struct B(String);
impl Drop for B {
  fn drop(&mut self) {
    println!("[B] drop");
  }
}
struct BDeserializer(StringDeserializer);
impl Deserializer<B> for BDeserializer {
  fn feed_token(
    &mut self,
    token: efjson::stream_parser::Token,
  ) -> Result<efjson::deserialize::DeserResult<B>, efjson::deserialize::DeserError> {
    Ok(self.0.feed_token(token)?.map(|v| {
      println!("[B] new");
      B(v)
    }))
  }
}
impl Drop for BDeserializer {
  fn drop(&mut self) {
    println!("[BDeserializer] drop");
  }
}
impl DefaultDeserializable<B> for B {
  type DefaultDeserializer = BDeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    println!("[BDeserializer] new");
    BDeserializer(String::default_deserializer())
  }
}

#[derive(Deserializable)]
struct Structure {
  a: A,
  b: B,
}

fn deserialize_part<T: DefaultDeserializable<T>>(input: &str) {
  let mut deserializer = <T as DefaultDeserializable<T>>::default_deserializer();
  let mut parser = StreamParser::new(ParserOption::all());
  for c in input.chars() {
    let _ = deserializer.feed_token(parser.feed_one(c).unwrap());
  }
}
macro_rules! test {
  ($t:ty, $input:expr) => {{
    println!("======{}", $input);
    deserialize_part::<$t>($input)
  }};
}

fn main() {
  test!(Structure, r#"{'a':"#);
  test!(Structure, r#"{"a":'1"#);
  test!(Structure, r#"{"a":'1',"#);
  test!(Structure, r#"{"a":'1',"b":'1"#);
  test!(Structure, r#"{"a":'1',"b":'1'"#);
  test!(Structure, r#"{"a":'1',"b":'1'}"#);

  test!((A, B), r#"['1"#);
  test!((A, B), r#"['1',"#);
  test!((A, B), r#"['1','2'"#);
  test!((A, B), r#"['1','2',]"#);
  test!((A, B), r#"['1','2','"#);
}
