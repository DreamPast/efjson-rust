use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::TokenInfo,
};

#[derive(Debug)]
pub struct StatelessDeserializer<T> {
  _phantom: std::marker::PhantomData<T>,
}

impl Deserializer<bool> for StatelessDeserializer<bool> {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<bool>, DeserError> {
    match token.info {
      TokenInfo::True(_, done) => {
        if done {
          Ok(DeserResult::Complete(true))
        } else {
          Ok(DeserResult::Continue)
        }
      }
      TokenInfo::False(_, done) => {
        if done {
          Ok(DeserResult::Complete(false))
        } else {
          Ok(DeserResult::Continue)
        }
      }
      _ => {
        if token_is_space(&token) {
          Ok(DeserResult::Continue)
        } else {
          Err("expect boolean".into())
        }
      }
    }
  }
}
impl DefaultDeserializable<bool> for bool {
  type DefaultDeserializer = StatelessDeserializer<bool>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    StatelessDeserializer { _phantom: std::marker::PhantomData }
  }
}

impl Deserializer<()> for StatelessDeserializer<()> {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<()>, DeserError> {
    match token.info {
      TokenInfo::Null(_, done) => {
        if done {
          Ok(DeserResult::Complete(()))
        } else {
          Ok(DeserResult::Continue)
        }
      }
      _ => {
        if token_is_space(&token) {
          Ok(DeserResult::Continue)
        } else {
          Err("expect null".into())
        }
      }
    }
  }
}
impl DefaultDeserializable<()> for () {
  type DefaultDeserializer = StatelessDeserializer<()>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    StatelessDeserializer { _phantom: std::marker::PhantomData }
  }
}
