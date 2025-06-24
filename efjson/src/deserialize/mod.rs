use crate::{
  stream_parser::{StreamParser, Token},
  ParserOption,
};

#[derive(Debug)]
pub enum DeserResult<Result> {
  Complete(Result),
  CompleteWithRollback(Result),
  Continue,
}
impl<Result> DeserResult<Result> {
  pub fn map<Dest, Op: FnOnce(Result) -> Dest>(self, op: Op) -> DeserResult<Dest> {
    match self {
      DeserResult::Complete(result) => DeserResult::Complete(op(result)),
      DeserResult::CompleteWithRollback(result) => DeserResult::CompleteWithRollback(op(result)),
      DeserResult::Continue => DeserResult::Continue,
    }
  }
}

pub type DeserError = Box<dyn std::error::Error + Send + Sync>;

pub trait Deserializer<T> {
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<T>, DeserError>;

  fn feed_token_iter(
    &mut self,
    token_iter: impl Iterator<Item = Token>,
  ) -> Result<DeserResult<T>, DeserError> {
    for token in token_iter {
      match self.feed_token(token)? {
        DeserResult::Continue => {}
        rest => return Ok(rest),
      }
    }
    Ok(DeserResult::Continue)
  }
}

pub fn unwrap_deser_result<T>(result: Result<DeserResult<T>, DeserError>) -> Result<T, DeserError> {
  match result? {
    DeserResult::Complete(v) | DeserResult::CompleteWithRollback(v) => Ok(v),
    DeserResult::Continue => Err("incomplete deserialization".into()),
  }
}

pub trait DefaultDeserializable<T> {
  type DefaultDeserializer: Deserializer<T>;
  fn default_deserializer() -> Self::DefaultDeserializer;
}
pub fn create_default_deserializer<T: DefaultDeserializable<T>>(
) -> <T as DefaultDeserializable<T>>::DefaultDeserializer {
  T::default_deserializer()
}

pub fn deserialize<T: DefaultDeserializable<T>>(
  option: ParserOption,
  s: &str,
) -> Result<T, DeserError> {
  let mut deserializer = create_default_deserializer::<T>();
  for token in StreamParser::create_iter(option, s.chars()) {
    match deserializer.feed_token(token.map_err(|e| -> DeserError { e.into() })?) {
      Ok(res) => match res {
        DeserResult::Complete(v) | DeserResult::CompleteWithRollback(v) => return Ok(v),
        DeserResult::Continue => {}
      },
      Err(e) => return Err(e),
    }
  }
  Err("incomplete deserialization".into())
}
pub fn deserialize_tokens<T: DefaultDeserializable<T>>(
  tokens: impl Iterator<Item = Token>,
) -> Result<T, DeserError> {
  let mut deserializer = create_default_deserializer::<T>();
  unwrap_deser_result(deserializer.feed_token_iter(tokens))
}

mod receiver;
pub use receiver::*;
mod deserlizer;
pub use deserlizer::*;
