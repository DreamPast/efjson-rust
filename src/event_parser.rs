use std::hint::unreachable_unchecked;

use crate::{
  JsonArray, JsonArrayIndex, JsonLocation, JsonObject, JsonOption, JsonParserError,
  JsonParserPosition, JsonStreamParser, JsonToken, JsonTokenInfo, JsonTokenRootInfo, JsonValue,
};

#[derive(Default)]
pub struct JsonEventObjectReceiver {
  pub set: Option<Box<dyn FnMut(&str, &JsonValue)>>,
  pub next: Option<Box<dyn FnMut()>>,
  pub key_receiver: Option<Box<dyn FnMut(char)>>,
  pub key_save: Option<Box<dyn FnMut(&String)>>,
  pub subreceiver: Option<Box<dyn FnMut(&str) -> Option<JsonEventReceiver>>>,
}
#[derive(Default)]
pub struct JsonEventArrayReceiver {
  pub set: Option<Box<dyn FnMut(JsonArrayIndex, &JsonValue)>>,
  pub next: Option<Box<dyn FnMut(JsonArrayIndex)>>,
  pub subreceiver: Option<Box<dyn FnMut(JsonArrayIndex) -> Option<JsonEventReceiver>>>,
}

pub struct JsonEventReceiver {
  pub start: Option<Box<dyn FnOnce()>>,
  pub end: Option<Box<dyn FnOnce()>>,
  pub feed: Option<Box<dyn FnMut(&JsonToken)>>,
  pub save: Option<Box<dyn FnOnce(&JsonValue)>>,
  pub integer_save: Option<Box<dyn FnOnce(i64)>>,

  pub accept_null: bool,
  pub accept_boolean: bool,
  pub accept_integer: bool,
  pub accept_number: bool,
  pub accept_string: bool,
  pub accept_object: bool,
  pub accept_array: bool,

  pub string_append: Option<Box<dyn FnMut(char)>>,
  pub object: JsonEventObjectReceiver,
  pub array: JsonEventArrayReceiver,
}
impl JsonEventReceiver {
  pub fn new_empty() -> Self {
    JsonEventReceiver {
      accept_null: false,
      accept_boolean: false,
      accept_integer: false,
      accept_number: false,
      accept_string: false,
      accept_object: false,
      accept_array: false,
      start: None,
      end: None,
      feed: None,
      save: None,
      integer_save: None,
      string_append: None,
      object: JsonEventObjectReceiver::default(),
      array: JsonEventArrayReceiver::default(),
    }
  }
  pub fn new_all() -> Self {
    JsonEventReceiver {
      accept_null: true,
      accept_boolean: true,
      accept_integer: true,
      accept_number: true,
      accept_string: true,
      accept_object: true,
      accept_array: true,
      start: None,
      end: None,
      feed: None,
      save: None,
      integer_save: None,
      string_append: None,
      object: JsonEventObjectReceiver::default(),
      array: JsonEventArrayReceiver::default(),
    }
  }
}

#[derive(Clone, Debug)]
struct _ObjectState {
  save_child: bool,
  child: JsonValue,
  key: Option<String>,
  save_key: bool,
  save_value: bool,
  object: Option<JsonObject>,
}
#[derive(Clone, Debug)]
struct _ArrayState {
  save_child: bool,
  child: Option<JsonValue>,
  index: JsonArrayIndex,
  array: Option<JsonArray>,
}

#[derive(Clone, Debug)]
enum _SubState {
  None,
  Null,
  Boolean,
  Number(Option<Vec<char>>),
  String(Option<String>, bool),
  Object(_ObjectState),
  Array(_ArrayState),
}
struct _State {
  receiver: JsonEventReceiver,
  substate: _SubState,
}

fn parse_number(s: &Vec<char>) -> Result<f64, JsonParserError> {
  let s: String = s.iter().collect();
  s.parse::<f64>().map_err(|_| "failed to parse number")
}
fn parse_integer(s: &Vec<char>) -> Option<i64> {
  let c0 = unsafe { *s.get_unchecked(0) };
  let start = (c0 == '+' || c0 == '-') as usize;
  let radix: u64 = if unsafe { *s.get_unchecked(start) } == '0' {
    match s.get(start + 1).unwrap_or(&'0') {
      'x' | 'X' => 16,
      'o' | 'O' => 8,
      'b' | 'B' => 2,
      _ => 10,
    }
  } else {
    10
  };
  let mut start = start + if radix == 10 { 0 } else { 2 };
  let mut value: u64 = 0;
  while start < s.len() {
    let c = unsafe { *s.get_unchecked(start) };
    value = value.checked_mul(radix)?;
    value = value.checked_add(match c {
      '0'..='9' => c as u64 - '0' as u64,
      'a'..='z' => (c as u64 - 'a' as u64) + 10,
      'A'..='Z' => (c as u64 - 'A' as u64) + 10,
      _ => {
        return None;
      }
    })?;
    start += 1;
  }

  if c0 == '-' {
    if value == 0 {
      None
    } else if value <= (i64::MAX as u64 + 1) {
      Some((value as i64).overflowing_neg().0)
    } else {
      None
    }
  } else {
    (value < i64::MAX as u64).then_some(value as i64)
  }
}

macro_rules! call_opt {
  ($opt:expr $(, $args:expr )* ) => {
    if let Some(f) = $opt.as_mut() {
      f($($args),*);
    }
  };
}
macro_rules! call_opt_once {
  ($opt:expr $(, $args:expr )* ) => {
    if let Some(f) = $opt.take() {
      f($($args),*);
    }
  };
}

pub struct JsonEventEmitter {
  stack: Vec<_State>,
}
impl JsonEventEmitter {
  pub fn new(receiver: JsonEventReceiver) -> Self {
    Self { stack: vec![_State { receiver, substate: _SubState::None }] }
  }
  fn _end_value<T>(&mut self, val: T)
  where
    T: Into<JsonValue>,
  {
    let mut state = unsafe { self.stack.pop().unwrap_unchecked() };
    call_opt_once!(state.receiver.end);
    let json_value: JsonValue = val.into();
    call_opt_once!(state.receiver.save, &json_value);
    if let Some(back) = self.stack.last_mut() {
      match back.substate {
        _SubState::Array(ref mut state) => state.child = Some(json_value),
        _SubState::Object(ref mut state) => state.child = json_value,
        _ => unsafe { unreachable_unchecked() },
      }
    }
  }
  fn _end_value_nosave(&mut self) {
    call_opt_once!(unsafe { self.stack.pop().unwrap_unchecked() }.receiver.end)
  }

  pub fn _need_save(&self) -> bool {
    let len = self.stack.len();
    return unsafe { self.stack.get_unchecked(len - 1) }.receiver.save.is_some()
      || (len >= 2
        && match &unsafe { self.stack.get_unchecked(len - 2) }.substate {
          _SubState::Array(state) => state.save_child,
          _SubState::Object(state) => state.save_child,
          _ => false,
        });
  }
  pub fn _feed_stateless(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    match token.info {
      JsonTokenInfo::Null(done, _) => {
        if let _SubState::None = &state.substate {
          call_opt_once!(state.receiver.start);
          state.substate = _SubState::Null;
          call_opt!(state.receiver.feed, &token);
        } else {
          call_opt!(state.receiver.feed, &token);
          if done {
            self._end_value(());
          }
        }
        Ok(())
      }
      JsonTokenInfo::True(done, _) => {
        if let _SubState::None = &state.substate {
          call_opt_once!(state.receiver.start);
          state.substate = _SubState::Boolean;
          call_opt!(state.receiver.feed, &token);
        } else {
          call_opt!(state.receiver.feed, &token);
          if done {
            self._end_value(true);
          }
        }
        Ok(())
      }
      JsonTokenInfo::False(done, _) => {
        if let _SubState::None = &state.substate {
          call_opt_once!(state.receiver.start);
          state.substate = _SubState::Boolean;
          call_opt!(state.receiver.feed, &token);
        } else {
          call_opt!(state.receiver.feed, &token);
          if done {
            self._end_value(false);
          }
        }
        Ok(())
      }
      _ => unreachable!(),
    }
  }
  pub fn _feed_number(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let need_save = self._need_save();
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    if let _SubState::None = &state.substate {
      if !state.receiver.accept_number && !state.receiver.accept_integer {
        return Err("number is rejected");
      }
      state.substate = _SubState::Number(if need_save || state.receiver.save.is_some() {
        Some(vec![token.c])
      } else {
        None
      });
      call_opt_once!(state.receiver.start);
      call_opt!(state.receiver.feed, &token);
      return Ok(());
    }
    call_opt!(state.receiver.feed, &token);
    let _SubState::Number(list) = &mut state.substate else { unreachable!() };
    if let Some(list) = list {
      list.push(token.c);
    }
    Ok(())
  }
  pub fn _feed_string(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let need_save = self._need_save();
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    if let _SubState::None = &state.substate {
      if !state.receiver.accept_string {
        return Err("string is rejected");
      }
      state.substate = _SubState::String(
        if need_save || state.receiver.save.is_some() { Some(String::from(token.c)) } else { None },
        false,
      );
      call_opt_once!(state.receiver.start);
      call_opt!(state.receiver.feed, &token);
      return Ok(());
    }
    call_opt!(state.receiver.feed, &token);
    let _SubState::String(list, _) = &mut state.substate else { unreachable!() };
    match token.info {
      JsonTokenInfo::StringStart => unreachable!(),
      JsonTokenInfo::StringEnd => {
        if let Some(s) = list.take() {
          self._end_value(s);
        } else {
          self._end_value_nosave();
        }
      }
      JsonTokenInfo::StringNormal => {
        call_opt!(state.receiver.string_append, token.c);
        list.as_mut().map(|l| l.push(token.c));
      }
      JsonTokenInfo::StringEscapeStart
      | JsonTokenInfo::StringEscapeUnicodeStart
      | JsonTokenInfo::StringEscapeHexStart
      | JsonTokenInfo::StringNextLine => {}
      JsonTokenInfo::StringEscape(c) => {
        call_opt!(state.receiver.string_append, c);
        list.as_mut().map(|l| l.push(token.c));
      }
      JsonTokenInfo::StringEscapeUnicode(c, _) | JsonTokenInfo::StringEscapeHex(c, _) => {
        if let Some(c) = c {
          call_opt!(state.receiver.string_append, c);
          list.as_mut().map(|l| l.push(token.c));
        }
      }
      _ => unreachable!(),
    }
    Ok(())
  }
  pub fn _feed_identifier(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let need_save = self._need_save();
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    if let _SubState::None = &state.substate {
      if !state.receiver.accept_string {
        return Err("string is rejected");
      }
      state.substate = _SubState::String(
        if need_save || state.receiver.save.is_some() { Some(String::from(token.c)) } else { None },
        false,
      );

      call_opt_once!(state.receiver.start);
      call_opt!(
        state.receiver.feed,
        &JsonToken { c: '"', info: JsonTokenInfo::StringStart, location: JsonLocation::Key }
      );
    }
    let _SubState::String(list, _) = &mut state.substate else { unreachable!() };
    match token.info {
      JsonTokenInfo::IdentifierNormal => {
        call_opt!(
          state.receiver.feed,
          &JsonToken { c: token.c, info: JsonTokenInfo::StringNormal, location: JsonLocation::Key }
        );
        call_opt!(state.receiver.string_append, token.c);
        if let Some(list) = list {
          list.push(token.c);
        }
      }
      JsonTokenInfo::IdentifierEscapeStart(done, _) => {
        call_opt!(
          state.receiver.feed,
          &JsonToken {
            c: token.c,
            info: if done {
              JsonTokenInfo::StringEscapeStart
            } else {
              JsonTokenInfo::StringEscapeUnicodeStart
            },
            location: JsonLocation::Key,
          }
        );
      }
      JsonTokenInfo::IdentifierEscape(c, idx) => {
        call_opt!(
          state.receiver.feed,
          &JsonToken {
            c: token.c,
            info: JsonTokenInfo::StringEscapeUnicode(c, idx),
            location: JsonLocation::Key,
          }
        );
        if let Some(c) = c {
          call_opt!(state.receiver.string_append, c);
          if let Some(list) = list {
            list.push(c);
          }
        }
      }

      _ => unreachable!(),
    }
    Ok(())
  }
  pub fn _feed_object(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let need_save = self._need_save();
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    if let _SubState::None = &state.substate {
      assert!(matches!(token.info, JsonTokenInfo::ObjectStart));
      if !state.receiver.accept_object {
        return Err("object is rejected");
      }
      let subreceiver = &state.receiver.object;
      let save = need_save || state.receiver.save.is_some();
      let save_value = save || subreceiver.set.is_some();
      let save_key = save_value || subreceiver.key_save.is_some();
      state.substate = _SubState::Object(_ObjectState {
        child: JsonValue::NULL,
        key: None,
        save_key,
        save_value,
        save_child: save_key,
        object: if save { Some(JsonObject::new()) } else { None },
      });
      call_opt_once!(state.receiver.start);
      call_opt!(state.receiver.feed, &token);
      self.stack.push(_State { receiver: JsonEventReceiver::new_all(), substate: _SubState::None });
      return Ok(());
    }
    call_opt!(state.receiver.feed, &token);
    let _SubState::Object(obj) = &mut state.substate else { unreachable!() };
    match token.info {
      JsonTokenInfo::ObjectStart => unreachable!(),
      JsonTokenInfo::ObjectEnd => {
        if let Some(key) = obj.key.take() {
          let value = std::mem::replace(&mut obj.child, JsonValue::NULL);
          call_opt!(state.receiver.object.set, &key, &value);
          if let Some(target) = obj.object.as_mut() {
            target.insert(key, value);
          }
        }
        if let Some(target) = obj.object.take() {
          self._end_value(target);
        } else {
          self._end_value_nosave();
        }
      }
      JsonTokenInfo::ObjectNext => {
        if let Some(key) = obj.key.take() {
          let value = std::mem::replace(&mut obj.child, JsonValue::NULL);
          call_opt!(state.receiver.object.set, &key, &value);
          if let Some(target) = obj.object.as_mut() {
            target.insert(key, value);
          }
        }
        call_opt!(state.receiver.object.next);
        obj.key = None;
        obj.child = JsonValue::NULL;
        obj.save_child = obj.save_key;
        self
          .stack
          .push(_State { receiver: JsonEventReceiver::new_all(), substate: _SubState::None });
      }
      JsonTokenInfo::ObjectValueStart => {
        if let JsonValue::STRING(s) = std::mem::replace(&mut obj.child, JsonValue::NULL) {
          obj.key = Some(s);
        }
        obj.save_child = obj.save_value;
        let next_receiver = state
          .receiver
          .object
          .subreceiver
          .as_mut()
          .and_then(|f| f(&obj.key.as_ref().unwrap()))
          .unwrap_or_else(JsonEventReceiver::new_all);
        self.stack.push(_State { receiver: next_receiver, substate: _SubState::None })
      }
      _ => unreachable!(),
    }
    Ok(())
  }
  pub fn _feed_array(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let need_save = self._need_save();
    let state = unsafe { self.stack.last_mut().unwrap_unchecked() };
    if let _SubState::None = &state.substate {
      assert!(matches!(token.info, JsonTokenInfo::ArrayStart));
      if !state.receiver.accept_array {
        return Err("array is rejected");
      }
      let save = need_save || state.receiver.save.is_some();
      let save_child = save || state.receiver.array.set.is_some();
      state.substate = _SubState::Array(_ArrayState {
        child: None,
        index: 0,
        save_child,
        array: if save { Some(JsonArray::new()) } else { None },
      });
      call_opt_once!(state.receiver.start);
      call_opt!(state.receiver.feed, &token);
      let subreceiver = state
        .receiver
        .array
        .subreceiver
        .as_mut()
        .and_then(|f| f(0))
        .unwrap_or_else(JsonEventReceiver::new_all);
      self.stack.push(_State { receiver: subreceiver, substate: _SubState::None });
      return Ok(());
    }
    call_opt!(state.receiver.feed, &token);
    let _SubState::Array(arr) = &mut state.substate else { unreachable!() };
    match token.info {
      JsonTokenInfo::ArrayStart => unreachable!(),
      JsonTokenInfo::ArrayNext => {
        if let Some(child) = arr.child.take() {
          call_opt!(state.receiver.array.set, arr.index, &child);
          arr.array.as_mut().unwrap().push(child);
        }
        arr.index += 1;
        call_opt!(state.receiver.array.next, arr.index);
        let next_receiver = state
          .receiver
          .array
          .subreceiver
          .as_mut()
          .and_then(|f| f(arr.index))
          .unwrap_or_else(JsonEventReceiver::new_all);
        self.stack.push(_State { receiver: next_receiver, substate: _SubState::None });
      }
      JsonTokenInfo::ArrayEnd => {
        if let Some(child) = arr.child.take() {
          call_opt!(state.receiver.array.set, arr.index, &child);
          arr.array.as_mut().unwrap().push(child);
          arr.index += 1;
        }
        if let Some(target) = arr.array.take() {
          assert_eq!(arr.index, target.len());
          self._end_value(target);
        } else {
          self._end_value_nosave();
        }
      }
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn feed_one(&mut self, token: JsonToken) -> Result<(), JsonParserError> {
    let Some(state) = self.stack.last_mut() else {
      return Ok(());
    };

    if let _SubState::Number(list) = &state.substate {
      // let (number_rec, int_rec) = (&state.receiver.number, &state.receiver.integer);
      if !matches!(token.info.get_root_info(), JsonTokenRootInfo::Number) {
        if let Some(list) = list {
          // saved
          if state.receiver.accept_integer {
            if let Some(int_val) = parse_integer(&list) {
              call_opt_once!(state.receiver.end);
              call_opt_once!(state.receiver.integer_save, int_val);
            } else if !state.receiver.accept_number {
              return Err("invalid integer");
            }
          }

          let val = parse_number(&list)?;
          self._end_value(val);
        } else {
          self._end_value_nosave();
        }
      }
    } else if let _SubState::String(list, is_identifier) = &mut state.substate {
      if *is_identifier && !matches!(token.info.get_root_info(), JsonTokenRootInfo::Identifier) {
        call_opt!(
          state.receiver.feed,
          &JsonToken { c: '"', info: JsonTokenInfo::StringEnd, location: JsonLocation::Key }
        );
        if let Some(s) = list.take() {
          self._end_value(s);
        }
      }
    } else if let _SubState::None = &state.substate {
      if matches!(token.info, JsonTokenInfo::ArrayEnd | JsonTokenInfo::ObjectEnd) {
        self.stack.pop();
      }
    }

    match token.info.get_root_info() {
      JsonTokenRootInfo::Number => self._feed_number(token),
      JsonTokenRootInfo::String => self._feed_string(token),
      JsonTokenRootInfo::Identifier => self._feed_identifier(token),
      JsonTokenRootInfo::Object => self._feed_object(token),
      JsonTokenRootInfo::Array => self._feed_array(token),
      JsonTokenRootInfo::Null => self._feed_stateless(token),
      JsonTokenRootInfo::Boolean => self._feed_stateless(token),
      JsonTokenRootInfo::Whitespace | JsonTokenRootInfo::Eof | JsonTokenRootInfo::Comment => Ok(()),
    }
  }
  pub fn feed<Container>(&mut self, tokens: Container) -> Result<(), JsonParserError>
  where
    Container: IntoIterator<Item = JsonToken>,
  {
    for token in tokens {
      self.feed_one(token)?;
    }
    Ok(())
  }

  pub fn parse<Container>(
    receiver: JsonEventReceiver,
    tokens: Container,
  ) -> Result<(), (usize, JsonParserError)>
  where
    Container: IntoIterator<Item = JsonToken>,
  {
    let mut parser = JsonEventEmitter::new(receiver);
    let mut cnt = 0;
    for token in tokens {
      if let Err(e) = parser.feed_one(token) {
        return Err((cnt, e));
      }
      cnt += 1;
    }
    Ok(())
  }
}

pub struct JsonEventParser {
  emitter: JsonEventEmitter,
  parser: JsonStreamParser,
}
impl JsonEventParser {
  pub fn new(receiver: JsonEventReceiver, option: JsonOption) -> Self {
    Self { emitter: JsonEventEmitter::new(receiver), parser: JsonStreamParser::new(option) }
  }
  pub fn feed_one(&mut self, c: char) -> Result<(), JsonParserError> {
    self.emitter.feed_one(self.parser.feed_one(c)?)
  }
  pub fn feed(&mut self, s: &str) -> Result<(), (usize, JsonParserError)> {
    let mut cnt = 0;
    for c in s.chars() {
      if let Err(e) = self.parser.feed_one(c).and_then(|token| self.emitter.feed_one(token)) {
        return Err((cnt, e));
      }
      cnt += 1;
    }
    Ok(())
  }
  pub fn end(&mut self) -> Result<(), JsonParserError> {
    self.feed_one('\0')
  }

  pub fn get_position(&self) -> JsonParserPosition {
    self.parser.get_position()
  }
  pub fn get_line(&self) -> JsonParserPosition {
    self.parser.get_line()
  }
  pub fn get_column(&self) -> JsonParserPosition {
    self.parser.get_column()
  }

  pub fn parse(
    receiver: JsonEventReceiver,
    option: JsonOption,
    str: &str,
  ) -> Result<(), (usize, JsonParserError)> {
    let mut parser = JsonEventParser::new(receiver, option);
    let mut cnt = 0;
    for c in str.chars() {
      if let Err(e) = parser.feed_one(c) {
        return Err((cnt, e));
      }
      cnt += 1;
    }
    if let Err(e) = parser.end() { Err((cnt, e)) } else { Ok(()) }
  }
}
