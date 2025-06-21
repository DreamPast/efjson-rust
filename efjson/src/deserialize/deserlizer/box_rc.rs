use std::{rc::Rc, sync::Arc};

use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::Token,
};

pub struct BoxDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  deserializer: Box<T::DefaultDeserializer>,
}
impl<T> Deserializer<Box<T>> for BoxDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Box<T>>, DeserError> {
    Ok(self.deserializer.feed_token(token)?.map(|v| Box::new(v)))
  }
}
impl<T> DefaultDeserializable<Box<T>> for Box<T>
where
  T: DefaultDeserializable<T>,
{
  type DefaultDeserializer = BoxDeserializer<T>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    BoxDeserializer { deserializer: Box::new(T::default_deserializer()) }
  }
}

pub struct RcDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  deserializer: Box<T::DefaultDeserializer>,
}
impl<T> Deserializer<Rc<T>> for RcDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Rc<T>>, DeserError> {
    Ok(self.deserializer.feed_token(token)?.map(|v| Rc::new(v)))
  }
}
impl<T> DefaultDeserializable<Rc<T>> for Rc<T>
where
  T: DefaultDeserializable<T>,
{
  type DefaultDeserializer = RcDeserializer<T>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    RcDeserializer { deserializer: Box::new(T::default_deserializer()) }
  }
}

pub struct ArcDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  deserializer: Box<T::DefaultDeserializer>,
}
impl<T> Deserializer<Arc<T>> for ArcDeserializer<T>
where
  T: DefaultDeserializable<T>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Arc<T>>, DeserError> {
    Ok(self.deserializer.feed_token(token)?.map(|v| Arc::new(v)))
  }
}
impl<T> DefaultDeserializable<Arc<T>> for Arc<T>
where
  T: DefaultDeserializable<T>,
{
  type DefaultDeserializer = ArcDeserializer<T>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    ArcDeserializer { deserializer: Box::new(T::default_deserializer()) }
  }
}
