use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::TokenInfo,
};

#[derive(Debug)]
struct OptionReceiver<T, Deser>
where
  Deser: Deserializer<T>,
{
  deser: Deser,
  started: bool,
  _phantom: std::marker::PhantomData<T>,
}
impl<T, Deser> Deserializer<Option<T>> for OptionReceiver<T, Deser>
where
  Deser: Deserializer<T>,
{
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<Option<T>>, DeserError> {
    if !self.started {
      if token_is_space(&token) {
        return Ok(DeserResult::Continue);
      }
      match token.info {
        TokenInfo::Null(_, done) => {
          return Ok(if done { DeserResult::Complete(None) } else { DeserResult::Continue });
        }
        _ => {
          self.started = true;
        }
      }
    }
    Ok(self.deser.feed_token(token)?.map(|v| Some(v)))
  }
}
impl<T> DefaultDeserializable<Option<T>> for Option<T>
where
  T: DefaultDeserializable<T>,
{
  fn default_deserializer() -> impl Deserializer<Option<T>> {
    OptionReceiver {
      deser: T::default_deserializer(),
      started: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
