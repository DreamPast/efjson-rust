use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::{Category, Token, TokenInfo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StageEnum {
  NotStarted,
  Stateless, // null, true, false
  Number,
  String,
  Structure,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonRawString {
  pub json: String,
}
#[derive(Debug)]
pub struct JsonRawStringDeserializer {
  stage: StageEnum,
  cnt: usize,
  json: String,
}
impl Deserializer<JsonRawString> for JsonRawStringDeserializer {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<JsonRawString>, DeserError> {
    match self.stage {
      StageEnum::NotStarted => {
        self.json.push(token.c);
        match token.info.get_category() {
          Category::String => {
            self.stage = StageEnum::String;
            Ok(DeserResult::Continue)
          }
          Category::Array => {
            self.stage = StageEnum::Structure;
            self.cnt = 1;
            Ok(DeserResult::Continue)
          }
          Category::Object => {
            self.stage = StageEnum::Structure;
            self.cnt = 1;
            Ok(DeserResult::Continue)
          }
          Category::Null | Category::Boolean => {
            self.stage = StageEnum::Stateless;
            Ok(DeserResult::Continue)
          }
          Category::Number => {
            self.stage = StageEnum::Number;
            Ok(DeserResult::Continue)
          }
          Category::Identifier => unreachable!(),
          Category::Comment | Category::Whitespace | Category::Eof => Ok(DeserResult::Continue),
        }
      }
      StageEnum::Stateless => {
        self.json.push(token.c);
        match token.info {
          TokenInfo::Null(_, done) | TokenInfo::True(_, done) | TokenInfo::False(_, done) => {
            if done {
              Ok(DeserResult::Complete(JsonRawString { json: std::mem::take(&mut self.json) }))
            } else {
              Ok(DeserResult::Continue)
            }
          }
          _ => unreachable!(),
        }
      }
      StageEnum::Number => {
        if matches!(token.info.get_category(), Category::Number) {
          self.json.push(token.c);
          Ok(DeserResult::Continue)
        } else {
          Ok(DeserResult::CompleteWithRollback(JsonRawString {
            json: std::mem::take(&mut self.json),
          }))
        }
      }
      StageEnum::String => {
        self.json.push(token.c);
        if matches!(token.info, TokenInfo::StringEnd) {
          Ok(DeserResult::Complete(JsonRawString { json: std::mem::take(&mut self.json) }))
        } else {
          Ok(DeserResult::Continue)
        }
      }
      StageEnum::Structure => {
        self.json.push(token.c);
        match token.info {
          TokenInfo::ArrayStart | TokenInfo::ObjectStart => {
            self.cnt += 1;
            Ok(DeserResult::Continue)
          }
          TokenInfo::ArrayEnd | TokenInfo::ObjectEnd => {
            self.cnt -= 1;
            if self.cnt == 0 {
              Ok(DeserResult::Complete(JsonRawString { json: std::mem::take(&mut self.json) }))
            } else {
              Ok(DeserResult::Continue)
            }
          }
          _ => Ok(DeserResult::Continue),
        }
      }
    }
  }
}
impl DefaultDeserializable<JsonRawString> for JsonRawString {
  type DefaultDeserializer = JsonRawStringDeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    JsonRawStringDeserializer { stage: StageEnum::NotStarted, cnt: 0, json: String::new() }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRawToken {
  pub tokens: Vec<Token>,
}
#[derive(Debug)]
pub struct JsonRawTokenDeserializer {
  stage: StageEnum,
  cnt: usize,
  tokens: Vec<Token>,
}
impl Deserializer<JsonRawToken> for JsonRawTokenDeserializer {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<JsonRawToken>, DeserError> {
    match self.stage {
      StageEnum::NotStarted => {
        self.tokens.push(token.clone());
        match token.info.get_category() {
          Category::String => {
            self.stage = StageEnum::String;
            Ok(DeserResult::Continue)
          }
          Category::Array => {
            self.stage = StageEnum::Structure;
            self.cnt = 1;
            Ok(DeserResult::Continue)
          }
          Category::Object => {
            self.stage = StageEnum::Structure;
            self.cnt = 1;
            Ok(DeserResult::Continue)
          }
          Category::Null | Category::Boolean => {
            self.stage = StageEnum::Stateless;
            Ok(DeserResult::Continue)
          }
          Category::Number => {
            self.stage = StageEnum::Number;
            Ok(DeserResult::Continue)
          }
          Category::Identifier => unreachable!(),
          Category::Comment | Category::Whitespace | Category::Eof => Ok(DeserResult::Continue),
        }
      }
      StageEnum::Stateless => {
        self.tokens.push(token.clone());
        match token.info {
          TokenInfo::Null(_, done) | TokenInfo::True(_, done) | TokenInfo::False(_, done) => {
            if done {
              Ok(DeserResult::Complete(JsonRawToken { tokens: std::mem::take(&mut self.tokens) }))
            } else {
              Ok(DeserResult::Continue)
            }
          }
          _ => unreachable!(),
        }
      }
      StageEnum::Number => {
        if matches!(token.info.get_category(), Category::Number) {
          self.tokens.push(token.clone());
          Ok(DeserResult::Continue)
        } else {
          Ok(DeserResult::CompleteWithRollback(JsonRawToken {
            tokens: std::mem::take(&mut self.tokens),
          }))
        }
      }
      StageEnum::String => {
        self.tokens.push(token.clone());
        if matches!(token.info, TokenInfo::StringEnd) {
          Ok(DeserResult::Complete(JsonRawToken { tokens: std::mem::take(&mut self.tokens) }))
        } else {
          Ok(DeserResult::Continue)
        }
      }
      StageEnum::Structure => {
        self.tokens.push(token.clone());
        match token.info {
          TokenInfo::ArrayStart | TokenInfo::ObjectStart => {
            self.cnt += 1;
            Ok(DeserResult::Continue)
          }
          TokenInfo::ArrayEnd | TokenInfo::ObjectEnd => {
            self.cnt -= 1;
            if self.cnt == 0 {
              Ok(DeserResult::Complete(JsonRawToken { tokens: std::mem::take(&mut self.tokens) }))
            } else {
              Ok(DeserResult::Continue)
            }
          }
          _ => Ok(DeserResult::Continue),
        }
      }
    }
  }
}
impl DefaultDeserializable<JsonRawToken> for JsonRawToken {
  type DefaultDeserializer = JsonRawTokenDeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    JsonRawTokenDeserializer { stage: StageEnum::NotStarted, cnt: 0, tokens: Vec::new() }
  }
}
