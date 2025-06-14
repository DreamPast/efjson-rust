use crate::{
  ParserOption,
  stream_parser::{StreamParser, Token, TokenInfo},
};

pub fn token_is_space(token: &Token) -> bool {
  return matches!(
    token.info,
    TokenInfo::CommentMayStart
      | TokenInfo::CommentMultiLine
      | TokenInfo::CommentSingleLine
      | TokenInfo::CommentMultiLineEnd
      | TokenInfo::Eof
      | TokenInfo::Whitespace
  );
}

#[derive(Debug)]
pub enum DeserResult<Result> {
  Complete(Result),
  CompleteWithRollback(Result),
  Continue,
}
impl<Result> DeserResult<Result> {
  fn map<Dest, Op>(self, op: Op) -> DeserResult<Dest>
  where
    Op: FnOnce(Result) -> Dest,
  {
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
}

pub fn feed_tokens_to<T>(
  deserializer: &mut impl Deserializer<T>,
  token_iter: impl Iterator<Item = Token>,
) -> Result<DeserResult<T>, DeserError> {
  for token in token_iter {
    match deserializer.feed_token(token)? {
      DeserResult::Continue => {}
      rest => return Ok(rest),
    }
  }
  Ok(DeserResult::Continue)
}
pub fn unwrap_deser_result<T>(result: Result<DeserResult<T>, DeserError>) -> Result<T, DeserError> {
  match result? {
    DeserResult::Complete(v) | DeserResult::CompleteWithRollback(v) => Ok(v),
    DeserResult::Continue => Err("incomplete deserialization".into()),
  }
}

pub trait DefaultDeserializable<T> {
  fn default_deserializer() -> impl Deserializer<T>;
}
pub fn create_default_deserializer<T: DefaultDeserializable<T>>() -> impl Deserializer<T> {
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
  unwrap_deser_result(feed_tokens_to(&mut deserializer, tokens))
}

mod receiver;
pub use receiver::*;
mod deserlizer;
pub use deserlizer::*;
