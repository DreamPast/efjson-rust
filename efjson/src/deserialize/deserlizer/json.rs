use crate::{
  deserialize::{
    create_default_deserializer, DefaultDeserializable, DeserError, DeserResult, Deserializer,
  },
  stream_parser::{Category, Token},
  JsonArray, JsonObject, JsonValue,
};

#[derive(Debug)]
enum JsonSubdeserializer {
  Null(<() as DefaultDeserializable<()>>::DefaultDeserializer),
  Boolean(<bool as DefaultDeserializable<bool>>::DefaultDeserializer),
  Number(<f64 as DefaultDeserializable<f64>>::DefaultDeserializer),
  String(<String as DefaultDeserializable<String>>::DefaultDeserializer),
  Array(<JsonArray as DefaultDeserializable<JsonArray>>::DefaultDeserializer),
  Object(<JsonObject as DefaultDeserializable<JsonObject>>::DefaultDeserializer),
}
#[derive(Debug)]
pub struct JsonDeserializer {
  subdeser: Option<Box<JsonSubdeserializer>>,
}
impl Deserializer<JsonValue> for JsonDeserializer {
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<JsonValue>, DeserError> {
    if self.subdeser.is_none() {
      match token.info.get_category() {
        Category::Whitespace | Category::Eof | Category::Comment => {
          return Ok(DeserResult::Continue);
        }
        Category::Null => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Null(create_default_deserializer::<()>())))
        }
        Category::Boolean => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Boolean(create_default_deserializer::<bool>())))
        }
        Category::Number => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Number(create_default_deserializer::<f64>())))
        }
        Category::String => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::String(create_default_deserializer::<String>())))
        }
        Category::Object => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Object(create_default_deserializer::<JsonObject>())))
        }
        Category::Array => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Array(create_default_deserializer::<JsonArray>())))
        }
        Category::Identifier => {
          return Err("unexpected identifier".into());
        }
      }
    }
    Ok(match self.subdeser.as_mut().unwrap().as_mut() {
      JsonSubdeserializer::Null(deser) => deser.feed_token(token)?.map(|_| JsonValue::Null),
      JsonSubdeserializer::Boolean(deser) => {
        deser.feed_token(token)?.map(|v| JsonValue::Boolean(v))
      }
      JsonSubdeserializer::Number(deser) => deser.feed_token(token)?.map(|v| JsonValue::Number(v)),
      JsonSubdeserializer::String(deser) => deser.feed_token(token)?.map(|v| JsonValue::String(v)),
      JsonSubdeserializer::Array(deser) => deser.feed_token(token)?.map(|v| JsonValue::Array(v)),
      JsonSubdeserializer::Object(deser) => deser.feed_token(token)?.map(|v| JsonValue::Object(v)),
    })
  }
}
impl DefaultDeserializable<JsonValue> for JsonValue {
  type DefaultDeserializer = JsonDeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    JsonDeserializer { subdeser: None }
  }
}
