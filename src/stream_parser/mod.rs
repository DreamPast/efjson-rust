use crate::JsonParserOption;

mod char_check;
use char_check::*;

mod error {
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum ErrorKind {
    // << other >>
    CommentForbidden,
    Eof,
    NonwhitespaceAfterEnd,
    TrailingCommaForbidden,
    Unexpected,
    WrongBracket,
    WrongColon,
    // << array >>
    CommaInEmptyArray,
    // << object >>
    BadIdentifierEscape,
    BadPropertyNameInObject,
    CommaInEmptyObject,
    EmptyValueInObject,
    ExpectedColon,
    InvalidIdentifier,
    InvalidIdentifierEscape,
    RepeatedColon,
    // << string >>
    BadEscapeInString,
    BadHexEscapeInString,
    BadUnicodeEscapeInString,
    ControlCharacterForbiddenInString,
    SingleQuoteForbidden,
    // << number >>
    EmptyExponentPart,
    EmptyFractionPart,
    EmptyIntegerPart,
    ExponentNotAllowed,
    FractionNotAllowed,
    LeadingZeroForbidden,
    PositiveSignForbidden,
    UnexpectedInNumber,
  }
  impl ErrorKind {
    pub fn get_reason(&self) -> &'static str {
      match self {
        // << other >>
        Self::CommentForbidden => "comment not allowed",
        Self::Eof => "structure broken because of EOF",
        Self::NonwhitespaceAfterEnd => "unexpected non-whitespace character after end of JSON",
        Self::TrailingCommaForbidden => "trailing comma not allowed",
        Self::Unexpected => "unexpected character",
        Self::WrongBracket => "wrong bracket",
        Self::WrongColon => "colon only allowed between property name and value",
        // << array >>
        Self::CommaInEmptyArray => "empty array with trailing comma not allowed",
        // << object >>
        Self::BadIdentifierEscape => "the escape sequence for an identifier must start with \"/u\"",
        Self::BadPropertyNameInObject => "property name must be a string",
        Self::CommaInEmptyObject => "empty object with trailing comma not allowed",
        Self::EmptyValueInObject => "unexpected empty value in object",
        Self::ExpectedColon => "colon expected between property name and value",
        Self::InvalidIdentifier => "invalid identifier in JSON string",
        Self::InvalidIdentifierEscape => "invalid identifier escape sequence in JSON5 identifier",
        Self::RepeatedColon => "repeated colon not allowed",
        // << string >>
        Self::BadEscapeInString => "bad escape sequence in JSON string",
        Self::BadHexEscapeInString => "bad hex escape sequence in JSON string",
        Self::BadUnicodeEscapeInString => "bad Unicode escape sequence in JSON string",
        Self::ControlCharacterForbiddenInString => "control character not allowed in JSON string",
        Self::SingleQuoteForbidden => "single quote not allowed",
        // << number >>
        Self::EmptyExponentPart => "the exponent part of a number cannot be empty",
        Self::EmptyFractionPart => "the fraction part of a number cannot be empty",
        Self::EmptyIntegerPart => "the integer part of a number cannot be empty",
        Self::ExponentNotAllowed => "exponent part not allowed in non-decimal number",
        Self::FractionNotAllowed => "fraction part not allowed in non-decimal number",
        Self::LeadingZeroForbidden => "leading zero not allowed",
        Self::PositiveSignForbidden => "positive sign not allowed",
        Self::UnexpectedInNumber => "unexpected character in number",
      }
    }
  }
  impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}", self.get_reason())
    }
  }
}
use error::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParserError {
  pub position: usize,
  pub line: usize,
  pub column: usize,
  pub character: char,
  /** This may change frequently, so external modification should be avoided */
  kind: ErrorKind,
}
impl std::fmt::Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "At {}:{}({}), Character U+{:05X} - {}",
      self.line, self.column, self.position, self.character as u32, self.kind
    )
  }
}
impl ParserError {
  pub fn get_reason(&self) -> &'static str {
    self.kind.get_reason()
  }
}

mod internal {
  #[derive(Clone, Debug)]
  pub enum _ValueNumberState {
    Sign,
    Zero,
    Digit,
  }
  #[derive(Clone, Debug)]
  pub enum _ValueExponentState {
    Desire,
    Sign,
    Digit,
  }

  #[derive(Clone, Debug)]
  pub enum _ValueState {
    Empty,
    Null(u8),
    True(u8),
    False(u8),
    String(bool),
    StringEscape(bool),
    StringUnicode(bool, u8, u16),
    Number(_ValueNumberState),
    NumberFraction(bool),
    NumberExponent(_ValueExponentState),
    /* JSON5 */
    StringMultilineCr(bool),       // used to check \r\n
    StringEscapeHex(bool, u8, u8), // used to check \xNN
    NumberInfinity(u8),            // used to check "Infinity"
    NumberNan(u8),                 // used to check "NaN"
    NumberHex(bool),               // used to check hexadecimal number
    NumberOct(bool),               // used to check octal number
    NumberBin(bool),               // used to check binary number

    CommentMayStart,        // used to check comment
    CommentSingleLine,      // used to check single line comment
    CommentMultiLine,       // used to check multi-line comment
    CommentMultiLineMayEnd, // used to check multi-line comment

    Identifier,                      // used to check identifier key
    IdentifierEscape(bool, u8, u16), // used to check identifier key
  }

  #[derive(Clone, Copy, Debug)]
  pub enum _LocationState {
    RootStart,
    KeyFirstStart, // used to check trailing comma
    KeyStart,
    ValueStart,
    ElementFirstStart, // used to check trailing comma
    ElementStart,

    RootEnd,
    KeyEnd,
    ValueEnd,
    ElementEnd,
    Eof,
  }
}
use internal::*;

mod ret {
  use super::ParserError;

  #[derive(Clone, Copy, Debug)]
  pub enum Location {
    Root,
    Key,
    Value,
    Element,
    Object,
    Array,
  }

  #[derive(Clone, Copy, Debug)]
  pub enum TokenInfo {
    Whitespace,
    Eof,
    Null(bool, u8),
    True(bool, u8),
    False(bool, u8),

    StringStart,
    StringEnd,
    StringNormal,
    StringEscapeStart,
    StringEscapeUnicodeStart,
    StringEscape(char),
    StringEscapeUnicode(Option<char>, u8),
    StringNextLine,
    StringEscapeHexStart,
    StringEscapeHex(Option<char>, u8),

    NumberIntegerSign,
    NumberExponentSign,
    NumberIntegerDigit,
    NumberFractionDigit,
    NumberExponentDigit,
    NumberFractionStart,
    NumberExponentStart,
    NumberNan(bool, u8),
    NumberInfinity(bool, u8),
    NumberHexStart,
    NumberHex,
    NumberOctStart,
    NumberOct,
    NumberBinStart,
    NumberBin,

    ObjectStart,
    ObjectNext,
    ObjectValueStart,
    ObjectEnd,
    ArrayStart,
    ArrayNext,
    ArrayEnd,

    IdentifierNormal,
    IdentifierEscapeStart(bool, u8),
    IdentifierEscape(Option<char>, u8),

    CommentSingleLineMayStart,
    CommentSingleLine,
    CommentMultiLineMayStart,
    CommentMultiLine,
    CommentMultiLineEnd,
  }

  #[derive(Clone, Copy, Debug)]
  pub enum TokenRootInfo {
    String,
    Number,
    Object,
    Boolean,
    Null,
    Identifier,
    Array,
    Whitespace,
    Eof,
    Comment,
  }
  impl TokenInfo {
    pub fn get_root_info(&self) -> TokenRootInfo {
      match self {
        Self::Whitespace => TokenRootInfo::Whitespace,
        Self::Eof => TokenRootInfo::Eof,
        Self::Null(_, _) => TokenRootInfo::Null,
        Self::True(_, _) | Self::False(_, _) => TokenRootInfo::Boolean,
        Self::StringStart
        | Self::StringEnd
        | Self::StringNormal
        | Self::StringEscapeStart
        | Self::StringEscapeUnicodeStart
        | Self::StringEscape(_)
        | Self::StringEscapeUnicode(_, _)
        | Self::StringNextLine
        | Self::StringEscapeHexStart
        | Self::StringEscapeHex(_, _) => TokenRootInfo::String,
        Self::NumberIntegerSign
        | Self::NumberExponentSign
        | Self::NumberIntegerDigit
        | Self::NumberFractionDigit
        | Self::NumberExponentDigit
        | Self::NumberFractionStart
        | Self::NumberExponentStart
        | Self::NumberNan(_, _)
        | Self::NumberInfinity(_, _)
        | Self::NumberHexStart
        | Self::NumberHex
        | Self::NumberOctStart
        | Self::NumberOct
        | Self::NumberBinStart
        | Self::NumberBin => TokenRootInfo::Number,
        Self::ObjectStart | Self::ObjectNext | Self::ObjectValueStart | Self::ObjectEnd => {
          TokenRootInfo::Object
        }
        Self::ArrayStart | Self::ArrayNext | Self::ArrayEnd => TokenRootInfo::Array,
        Self::IdentifierNormal
        | Self::IdentifierEscapeStart(_, _)
        | Self::IdentifierEscape(_, _) => TokenRootInfo::Identifier,
        Self::CommentSingleLineMayStart
        | Self::CommentSingleLine
        | Self::CommentMultiLineMayStart
        | Self::CommentMultiLine
        | Self::CommentMultiLineEnd => TokenRootInfo::Comment,
      }
    }
  }

  #[derive(Clone, Copy, Debug)]
  pub struct Token {
    pub c: char,
    pub info: TokenInfo,
    pub location: Location,
  }

  pub type TokenResult = Result<Token, ParserError>;
}
pub use ret::*;

pub enum ParserStage {
  NotStarted,
  Parsing,
  Ended,
}

#[derive(Clone, Debug)]
pub struct StreamParser {
  position: usize,
  line: usize,
  column: usize,
  meet_cr: bool,
  location: _LocationState,
  state: _ValueState,
  stack: Vec<_LocationState>,
  option: JsonParserOption,
}
impl StreamParser {
  fn _next_location(state: _LocationState) -> _LocationState {
    match state {
      _LocationState::RootStart => _LocationState::RootEnd,
      _LocationState::KeyFirstStart | _LocationState::KeyStart => _LocationState::KeyEnd,
      _LocationState::ValueStart => _LocationState::ValueEnd,
      _LocationState::ElementFirstStart | _LocationState::ElementStart => {
        _LocationState::ElementEnd
      }
      _ => unreachable!(),
    }
  }
  fn _transform_location(location: _LocationState) -> Location {
    match location {
      _LocationState::RootStart | _LocationState::RootEnd => Location::Root,
      _LocationState::KeyFirstStart | _LocationState::KeyStart | _LocationState::KeyEnd => {
        Location::Key
      }
      _LocationState::ValueStart | _LocationState::ValueEnd => Location::Value,
      _LocationState::ElementFirstStart
      | _LocationState::ElementStart
      | _LocationState::ElementEnd => Location::Element,
      _ => panic!("invalid location state"),
    }
  }
  fn _throw(&self, c: char, kind: ErrorKind) -> ParserError {
    ParserError {
      character: c,
      position: self.position,
      line: self.line,
      column: self.column,
      kind,
    }
  }

  fn _handle_comma(&mut self) -> Result<(TokenInfo, Location), ParserError> {
    match self.location {
      _LocationState::ValueEnd => {
        self.location = _LocationState::KeyStart;
        Ok((TokenInfo::ObjectNext, Location::Object))
      }
      _LocationState::ElementEnd => {
        self.location = _LocationState::ElementStart;
        Ok((TokenInfo::ArrayNext, Location::Array))
      }
      _LocationState::ElementFirstStart => Err(self._throw(',', ErrorKind::TrailingCommaForbidden)),
      _LocationState::ValueStart => Err(self._throw(',', ErrorKind::EmptyValueInObject)),
      _ => Err(self._throw(',', ErrorKind::Unexpected)),
    }
  }
  fn _handle_array_end(&mut self) -> Result<(TokenInfo, Location), ParserError> {
    match self.location {
      _LocationState::ElementFirstStart | _LocationState::ElementEnd => {
        self.location = Self::_next_location(self.stack.pop().unwrap());
        self.state = _ValueState::Empty;
        Ok((TokenInfo::ArrayEnd, Self::_transform_location(self.location)))
      }
      _LocationState::ElementStart => {
        if self.option.accept_trailing_comma_in_array {
          unsafe {
            self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
          }
          self.state = _ValueState::Empty;
          Ok((TokenInfo::ArrayEnd, Self::_transform_location(self.location)))
        } else {
          Err(self._throw(']', ErrorKind::CommaInEmptyArray))
        }
      }
      _ => Err(self._throw(']', ErrorKind::WrongBracket)),
    }
  }
  fn _handle_object_end(&mut self) -> Result<(TokenInfo, Location), ParserError> {
    match self.location {
      _LocationState::KeyFirstStart | _LocationState::ValueEnd => {
        unsafe {
          self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
        }
        self.state = _ValueState::Empty;
        Ok((TokenInfo::ObjectEnd, Self::_transform_location(self.location)))
      }
      _LocationState::KeyStart => {
        if self.option.accept_trailing_comma_in_object {
          unsafe {
            self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
          }
          self.state = _ValueState::Empty;
          Ok((TokenInfo::ObjectEnd, Self::_transform_location(self.location)))
        } else {
          Err(self._throw('}', ErrorKind::CommaInEmptyObject))
        }
      }
      _ => Err(self._throw('}', ErrorKind::WrongBracket)),
    }
  }
  fn _handle_eof(&mut self) -> Result<TokenInfo, ParserError> {
    match self.location {
      _LocationState::RootStart | _LocationState::RootEnd => {
        self.location = _LocationState::Eof;
        Ok(TokenInfo::Eof)
      }
      _ => Err(self._throw('\0', ErrorKind::Eof)),
    }
  }
  fn _handle_slash(&mut self) -> Result<(TokenInfo, Location), ParserError> {
    if self.option.accept_single_line_comment || self.option.accpet_multi_line_comment {
      self.state = _ValueState::CommentMayStart;
      Ok((TokenInfo::CommentSingleLineMayStart, Self::_transform_location(self.location)))
    } else {
      Err(self._throw('/', ErrorKind::CommentForbidden))
    }
  }
  fn _handle_number_separator(&mut self, c: char) -> Result<(TokenInfo, Location), ParserError> {
    self.state = _ValueState::Empty;
    let old_location = self.location;
    self.location = Self::_next_location(self.location);
    match c {
      '\0' => Self::_handle_eof(self).map(|info| (info, Self::_transform_location(old_location))),
      '}' => Self::_handle_object_end(self),
      ']' => Self::_handle_array_end(self),
      ',' => Self::_handle_comma(self),
      '/' => Self::_handle_slash(self),
      _ => Ok((TokenInfo::Whitespace, Self::_transform_location(self.location))),
    }
  }

  fn _step_empty(&mut self, c: char) -> TokenResult {
    let olocation = Self::_transform_location(self.location);

    if _is_whitespace(c, self.option.accept_json5_whitespace) {
      return Ok(Token { c, info: TokenInfo::Whitespace, location: olocation });
    }
    if c == '\0' {
      return self._handle_eof().map(|info| Token { c, info: info, location: olocation });
    }
    if c == '/' {
      return self._handle_slash().map(|info| Token { c, info: info.0, location: info.1 });
    }
    if matches!(self.location, _LocationState::RootEnd) {
      return Err(self._throw(c, ErrorKind::NonwhitespaceAfterEnd));
    }

    if c == '"' || (c == '\'' && self.option.accept_single_quote) {
      self.state = _ValueState::String(c == '\'');
      return Ok(Token { c, info: TokenInfo::StringStart, location: olocation });
    }
    if c == '\'' {
      return Err(self._throw(c, ErrorKind::SingleQuoteForbidden));
    }
    if matches!(self.location, _LocationState::KeyFirstStart | _LocationState::KeyStart) {
      if self.option.accept_identifier_key {
        if _is_identifier_start(c) {
          self.state = _ValueState::Identifier;
          return Ok(Token { c, info: TokenInfo::IdentifierNormal, location: Location::Key });
        } else if c == '\\' {
          self.state = _ValueState::IdentifierEscape(false, 0, 0);
          return Ok(Token {
            c,
            info: TokenInfo::IdentifierEscape(None, 0),
            location: Location::Key,
          });
        }
      }
      if c != '}' {
        return Err(self._throw(c, ErrorKind::BadPropertyNameInObject));
      }
    }

    if c == ':' {
      if matches!(self.location, _LocationState::KeyEnd) {
        self.location = _LocationState::ValueStart;
        self.state = _ValueState::Empty;
        return Ok(Token { c, info: TokenInfo::ObjectValueStart, location: olocation });
      }
      return Err(self._throw(
        c,
        if matches!(self.location, _LocationState::ValueStart) {
          ErrorKind::RepeatedColon
        } else {
          ErrorKind::WrongColon
        },
      ));
    }
    if matches!(self.location, _LocationState::KeyEnd) {
      return Err(self._throw(c, ErrorKind::ExpectedColon));
    }

    match c {
      '[' => {
        self.stack.push(self.location);
        self.location = _LocationState::ElementFirstStart;
        self.state = _ValueState::Empty;
        Ok(Token { c, info: TokenInfo::ArrayStart, location: olocation })
      }
      ']' => self._handle_array_end().map(|(info, location)| Token { c, info, location }),

      '{' => {
        self.stack.push(self.location);
        self.location = _LocationState::KeyFirstStart;
        self.state = _ValueState::Empty;
        Ok(Token { c, info: TokenInfo::ObjectStart, location: olocation })
      }
      '}' => self._handle_object_end().map(|(info, location)| Token { c, info, location }),

      ',' => self._handle_comma().map(|(info, location)| Token { c, info, location }),

      '+' | '-' => {
        if c == '+' && self.option.accept_positive_sign {
          Err(self._throw(c, ErrorKind::PositiveSignForbidden))
        } else {
          self.state = _ValueState::Number(_ValueNumberState::Sign);
          Ok(Token { c, info: TokenInfo::NumberIntegerSign, location: olocation })
        }
      }

      '0'..='9' => {
        self.state = _ValueState::Number(if c == '0' {
          _ValueNumberState::Zero
        } else {
          _ValueNumberState::Digit
        });
        Ok(Token { c, info: TokenInfo::NumberIntegerDigit, location: olocation })
      }
      '.' => {
        if self.option.accept_empty_integer {
          self.state = _ValueState::NumberFraction(false);
          Ok(Token { c, info: TokenInfo::NumberFractionStart, location: olocation })
        } else {
          Err(self._throw(c, ErrorKind::EmptyIntegerPart))
        }
      }

      'N' => {
        if self.option.accept_nan {
          self.state = _ValueState::NumberNan(0);
          Ok(Token { c, info: TokenInfo::NumberNan(false, 0), location: olocation })
        } else {
          Err(self._throw(c, ErrorKind::Unexpected))
        }
      }
      'I' => {
        if self.option.accept_infinity {
          self.state = _ValueState::NumberInfinity(0);
          Ok(Token { c, info: TokenInfo::NumberInfinity(false, 0), location: olocation })
        } else {
          Err(self._throw(c, ErrorKind::Unexpected))
        }
      }

      'n' => {
        self.state = _ValueState::Null(1);
        Ok(Token { c, info: TokenInfo::Null(false, 0), location: olocation })
      }
      't' => {
        self.state = _ValueState::True(1);
        Ok(Token { c, info: TokenInfo::True(false, 0), location: olocation })
      }
      'f' => {
        self.state = _ValueState::False(1);
        Ok(Token { c, info: TokenInfo::False(false, 0), location: olocation })
      }

      _ => Err(self._throw(c, ErrorKind::Unexpected)),
    }
  }
  fn _step_string(&mut self, c: char, single_quote: bool) -> TokenResult {
    if c == if single_quote { '\'' } else { '"' } {
      self.location = Self::_next_location(self.location);
      self.state = _ValueState::Empty;
      Ok(Token {
        c,
        info: TokenInfo::StringEnd,
        location: Self::_transform_location(self.location),
      })
    } else if c == '\\' {
      self.state = _ValueState::StringEscape(single_quote);
      Ok(Token {
        c,
        info: TokenInfo::StringEscapeStart,
        location: Self::_transform_location(self.location),
      })
    } else if c == '\0' {
      Err(self._throw(c, ErrorKind::Eof))
    } else if _is_control(c) {
      Err(self._throw(c, ErrorKind::ControlCharacterForbiddenInString))
    } else {
      Ok(Token {
        c,
        info: TokenInfo::StringNormal,
        location: Self::_transform_location(self.location),
      })
    }
  }
  fn _step(&mut self, c: char) -> TokenResult {
    static _NULL: [char; 4] = ['n', 'u', 'l', 'l'];
    static _TRUE: [char; 4] = ['t', 'r', 'u', 'e'];
    static _FALSE: [char; 5] = ['f', 'a', 'l', 's', 'e'];
    static _NAN: [char; 3] = ['N', 'a', 'N'];
    static _INFINITY: [char; 8] = ['I', 'n', 'f', 'i', 'n', 'i', 't', 'y'];

    let olocation = Self::_transform_location(self.location);
    match self.state {
      _ValueState::Empty => self._step_empty(c),
      _ValueState::Null(ref mut idx) => {
        let old_idx = *idx;
        if c == _NULL[*idx as usize] {
          *idx += 1;
          Ok(Token {
            c,
            info: TokenInfo::Null(
              if old_idx == (_NULL.len() - 1) as u8 {
                self.state = _ValueState::Empty;
                self.location = Self::_next_location(self.location);
                true
              } else {
                false
              },
              old_idx,
            ),
            location: olocation,
          })
        } else {
          Err(self._throw(c, ErrorKind::Unexpected))
        }
      }
      _ValueState::True(ref mut idx) => {
        let old_idx = *idx;
        if c == _TRUE[*idx as usize] {
          *idx += 1;
          Ok(Token {
            c,
            info: TokenInfo::True(
              if old_idx == (_TRUE.len() - 1) as u8 {
                self.state = _ValueState::Empty;
                self.location = Self::_next_location(self.location);
                true
              } else {
                false
              },
              old_idx,
            ),
            location: olocation,
          })
        } else {
          Err(self._throw(c, ErrorKind::Unexpected))
        }
      }
      _ValueState::False(ref mut idx) => {
        let old_idx = *idx;
        if c == _FALSE[*idx as usize] {
          *idx += 1;
          Ok(Token {
            c,
            info: TokenInfo::False(
              if old_idx == (_FALSE.len() - 1) as u8 {
                self.state = _ValueState::Empty;
                self.location = Self::_next_location(self.location);
                true
              } else {
                false
              },
              old_idx,
            ),
            location: olocation,
          })
        } else {
          Err(self._throw(c, ErrorKind::Unexpected))
        }
      }
      _ValueState::NumberInfinity(ref mut idx) => {
        let old_idx = *idx;
        if c == _INFINITY[*idx as usize] {
          *idx += 1;
          Ok(Token {
            c,
            info: TokenInfo::NumberInfinity(
              if old_idx == (_INFINITY.len() - 1) as u8 {
                self.state = _ValueState::Empty;
                self.location = Self::_next_location(self.location);
                true
              } else {
                false
              },
              old_idx,
            ),
            location: olocation,
          })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }
      _ValueState::NumberNan(ref mut idx) => {
        let old_idx = *idx;
        if c == _NAN[*idx as usize] {
          *idx += 1;
          Ok(Token {
            c,
            info: TokenInfo::NumberNan(
              if old_idx == (_NAN.len() - 1) as u8 {
                self.state = _ValueState::Empty;
                self.location = Self::_next_location(self.location);
                true
              } else {
                false
              },
              old_idx,
            ),
            location: olocation,
          })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }

      _ValueState::StringMultilineCr(bl) => {
        if c == '\n' {
          self.state = _ValueState::String(bl);
          Ok(Token { c, info: TokenInfo::StringNextLine, location: olocation })
        } else {
          self._step_string(c, bl)
        }
      }
      _ValueState::String(bl) => self._step_string(c, bl),
      _ValueState::StringEscape(bl) => {
        if c == 'u' {
          self.state = _ValueState::StringUnicode(bl, 0, 0);
          Ok(Token {
            c,
            info: TokenInfo::StringEscapeUnicodeStart,
            location: Self::_transform_location(self.location),
          })
        } else {
          match _escape(c, self.option.accpet_json5_string_escape) {
            Some(escaped_char) => {
              self.state = _ValueState::String(bl);
              Ok(Token {
                c,
                info: TokenInfo::StringEscape(escaped_char),
                location: Self::_transform_location(self.location),
              })
            }
            None => {
              if self.option.accept_multiline_string && _is_next_line(c) {
                self.state = if c == '\r' {
                  _ValueState::StringMultilineCr(bl)
                } else {
                  _ValueState::String(bl)
                };
                Ok(Token {
                  c,
                  info: TokenInfo::StringNextLine,
                  location: Self::_transform_location(self.location),
                })
              } else if self.option.accpet_json5_string_escape && c == 'x' {
                self.state = _ValueState::StringEscapeHex(bl, 0, 0);
                Ok(Token {
                  c,
                  info: TokenInfo::StringEscapeHexStart,
                  location: Self::_transform_location(self.location),
                })
              } else {
                Err(self._throw(c, ErrorKind::BadEscapeInString))
              }
            }
          }
        }
      }
      _ValueState::StringUnicode(bl, ref mut idx, ref mut num) => {
        if _is_hex(c) {
          *num = *num << 4 | _to_digit(c) as u16;
          *idx += 1;
          if *idx == 4 {
            let num = *num;
            let _ = idx;
            self.state = _ValueState::String(bl);
            Ok(Token {
              c,
              info: TokenInfo::StringEscapeUnicode(char::from_u32(num as u32), 4),
              location: olocation,
            })
          } else {
            Ok(Token {
              c,
              info: TokenInfo::StringEscapeUnicode(None, *idx - 1),
              location: olocation,
            })
          }
        } else {
          Err(self._throw(c, ErrorKind::BadUnicodeEscapeInString))
        }
      }
      _ValueState::StringEscapeHex(bl, idx, num) => {
        if _is_hex(c) {
          if idx == 1 {
            let num = num << 4 | _to_digit(c);
            self.state = _ValueState::String(bl);

            Ok(Token {
              c,
              info: unsafe {
                TokenInfo::StringEscapeHex(Some(char::from_u32_unchecked(num as u32)), 4)
              },
              location: olocation,
            })
          } else {
            self.state = _ValueState::StringEscapeHex(bl, 1, _to_digit(c));
            Ok(Token { c, info: TokenInfo::StringEscapeHex(None, 0), location: olocation })
          }
        } else {
          Err(self._throw(c, ErrorKind::BadHexEscapeInString))
        }
      }

      _ValueState::Number(ref mut state) => {
        if c == '0' {
          if matches!(*state, _ValueNumberState::Zero) {
            Err(self._throw(c, ErrorKind::LeadingZeroForbidden))
          } else {
            if matches!(*state, _ValueNumberState::Sign) {
              *state = _ValueNumberState::Zero;
            }
            Ok(Token { c, info: TokenInfo::NumberIntegerDigit, location: olocation })
          }
        } else if matches!(c, '1'..='9') {
          if matches!(*state, _ValueNumberState::Zero) {
            Err(self._throw(c, ErrorKind::LeadingZeroForbidden))
          } else {
            if matches!(*state, _ValueNumberState::Sign) {
              *state = _ValueNumberState::Digit;
            }
            Ok(Token { c, info: TokenInfo::NumberIntegerDigit, location: olocation })
          }
        } else if c == '.' {
          if matches!(*state, _ValueNumberState::Sign) && !self.option.accept_empty_integer {
            Err(self._throw(c, ErrorKind::EmptyIntegerPart))
          } else {
            self.state = _ValueState::NumberFraction(false);
            Ok(Token { c, info: TokenInfo::NumberFractionStart, location: olocation })
          }
        } else if matches!(*state, _ValueNumberState::Sign) {
          if self.option.accept_infinity && c == 'I' {
            self.state = _ValueState::NumberInfinity(1);
            Ok(Token { c, info: TokenInfo::NumberInfinity(false, 0), location: olocation })
          } else if self.option.accept_nan && c == 'N' {
            self.state = _ValueState::NumberNan(1);
            Ok(Token { c, info: TokenInfo::NumberNan(false, 0), location: olocation })
          } else {
            Err(self._throw(c, ErrorKind::EmptyIntegerPart))
          }
        } else {
          if matches!(*state, _ValueNumberState::Zero) {
            match c {
              'x' | 'X' => {
                if self.option.accept_hexadecimal_integer {
                  self.state = _ValueState::NumberHex(false);
                  return Ok(Token { c, info: TokenInfo::NumberHexStart, location: olocation });
                }
              }
              'o' | 'O' => {
                if self.option.accept_octal_integer {
                  self.state = _ValueState::NumberOct(false);
                  return Ok(Token { c, info: TokenInfo::NumberOctStart, location: olocation });
                }
              }
              'b' | 'B' => {
                if self.option.accept_binary_integer {
                  self.state = _ValueState::NumberBin(false);
                  return Ok(Token { c, info: TokenInfo::NumberBinStart, location: olocation });
                }
              }
              _ => {}
            };
          }

          if c == 'e' || c == 'E' {
            self.state = _ValueState::NumberExponent(_ValueExponentState::Desire);
            Ok(Token { c, info: TokenInfo::NumberExponentStart, location: olocation })
          } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
            self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
          } else {
            Err(self._throw(c, ErrorKind::UnexpectedInNumber))
          }
        }
      }
      _ValueState::NumberFraction(ref mut state) => {
        if matches!(c, '0'..='9') {
          *state = true;
          Ok(Token { c, info: TokenInfo::NumberFractionDigit, location: olocation })
        } else if !*state && !self.option.accept_empty_fraction {
          Err(self._throw(c, ErrorKind::EmptyFractionPart))
        } else if c == 'e' || c == 'E' {
          self.state = _ValueState::NumberExponent(_ValueExponentState::Desire);
          Ok(Token { c, info: TokenInfo::NumberExponentStart, location: olocation })
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }
      _ValueState::NumberExponent(ref mut state) => {
        if c == '+' || c == '-' {
          match *state {
            _ValueExponentState::Desire => {
              *state = _ValueExponentState::Sign;
              Ok(Token { c, info: TokenInfo::NumberExponentSign, location: olocation })
            }
            _ => Err(self._throw(c, ErrorKind::UnexpectedInNumber)),
          }
        } else if matches!(c, '0'..='9') {
          *state = _ValueExponentState::Digit;
          Ok(Token { c, info: TokenInfo::NumberExponentDigit, location: olocation })
        } else if matches!(*state, _ValueExponentState::Desire | _ValueExponentState::Sign) {
          Err(self._throw(c, ErrorKind::EmptyExponentPart))
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }

      _ValueState::NumberHex(ref mut state) => {
        if _is_hex(c) {
          *state = true;
          Ok(Token { c, info: TokenInfo::NumberHex, location: olocation })
        } else if c == '.' {
          Err(self._throw(c, ErrorKind::FractionNotAllowed))
        } else if !*state {
          Err(self._throw(c, ErrorKind::EmptyIntegerPart))
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }
      _ValueState::NumberOct(ref mut state) => {
        if matches!(c, '0'..='7') {
          *state = true;
          Ok(Token { c, info: TokenInfo::NumberOct, location: olocation })
        } else if c == '.' {
          Err(self._throw(c, ErrorKind::ExponentNotAllowed))
        } else if !*state {
          Err(self._throw(c, ErrorKind::FractionNotAllowed))
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }
      _ValueState::NumberBin(ref mut state) => {
        if _is_hex(c) {
          *state = true;
          Ok(Token { c, info: TokenInfo::NumberBin, location: olocation })
        } else if c == '.' {
          Err(self._throw(c, ErrorKind::ExponentNotAllowed))
        } else if !*state {
          Err(self._throw(c, ErrorKind::FractionNotAllowed))
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| Token { c, info, location })
        } else {
          Err(self._throw(c, ErrorKind::UnexpectedInNumber))
        }
      }

      _ValueState::CommentMayStart => {
        if self.option.accept_single_line_comment && c == '/' {
          self.state = _ValueState::CommentSingleLine;
          Ok(Token { c, info: TokenInfo::CommentSingleLine, location: olocation })
        } else if self.option.accpet_multi_line_comment && c == '*' {
          self.state = _ValueState::CommentMultiLine;
          Ok(Token { c, info: TokenInfo::CommentMultiLine, location: olocation })
        } else {
          Err(self._throw(c, ErrorKind::CommentForbidden))
        }
      }
      _ValueState::CommentSingleLine => {
        if _is_next_line(c) {
          self.state = _ValueState::Empty;
        }
        Ok(Token { c, info: TokenInfo::CommentSingleLine, location: olocation })
      }
      _ValueState::CommentMultiLine => {
        if c == '*' {
          self.state = _ValueState::CommentMultiLineMayEnd;
        }
        Ok(Token { c, info: TokenInfo::CommentMultiLine, location: olocation })
      }
      _ValueState::CommentMultiLineMayEnd => {
        if c == '/' {
          self.state = _ValueState::Empty;
          Ok(Token {
            c,
            info: TokenInfo::CommentMultiLineEnd,
            location: Self::_transform_location(self.location),
          })
        } else if c == '*' {
          Ok(Token { c, info: TokenInfo::CommentMultiLine, location: olocation })
        } else {
          self.state = _ValueState::CommentMultiLine;
          Ok(Token { c, info: TokenInfo::CommentMultiLine, location: olocation })
        }
      }

      _ValueState::Identifier => {
        if c == ':' {
          self.location = _LocationState::ValueStart;
          self.state = _ValueState::Empty;
          Ok(Token { c, info: TokenInfo::ObjectValueStart, location: Location::Object })
        } else if _is_whitespace(c, self.option.accept_json5_whitespace) {
          self.location = _LocationState::KeyEnd;
          self.state = _ValueState::Empty;
          Ok(Token { c, info: TokenInfo::Whitespace, location: Location::Key })
        } else if _is_identifier_next(c) {
          Ok(Token { c, info: TokenInfo::IdentifierNormal, location: Location::Key })
        } else {
          Err(self._throw(c, ErrorKind::InvalidIdentifier))
        }
      }
      _ValueState::IdentifierEscape(ref mut prefix, ref mut idx, ref mut num) => {
        if !*prefix {
          if c == 'u' {
            *prefix = true;
            Ok(Token {
              c,
              info: TokenInfo::IdentifierEscapeStart(true, 1),
              location: Location::Key,
            })
          } else {
            Err(self._throw(c, ErrorKind::BadIdentifierEscape))
          }
        } else {
          if _is_hex(c) {
            *num = *num << 4 | _to_digit(c) as u16;
            *idx += 1;
            if *idx == 4 {
              let num = *num;
              let _ = idx;
              self.state = _ValueState::Empty;
              self.location = Self::_next_location(self.location);
              Ok(Token {
                c,
                info: TokenInfo::IdentifierEscape(char::from_u32(num as u32), 4),
                location: Location::Key,
              })
            } else {
              Ok(Token {
                c,
                info: TokenInfo::IdentifierEscape(None, *idx - 1),
                location: Location::Key,
              })
            }
          } else {
            Err(self._throw(c, ErrorKind::InvalidIdentifierEscape))
          }
        }
      }
    }
  }
}
impl StreamParser {
  pub fn new(option: JsonParserOption) -> Self {
    StreamParser {
      position: 0,
      line: 0,
      column: 0,
      meet_cr: false,
      location: _LocationState::RootStart,
      state: _ValueState::Empty,
      stack: Vec::new(),
      option: option,
    }
  }

  pub fn feed_one(&mut self, c: char) -> TokenResult {
    if self.meet_cr {
      if c != '\n' {
        self.line += 1;
        self.column = 0;
      }
      self.meet_cr = false;
    }
    let ret = self._step(c);
    if ret.is_ok() {
      self.position += 1;
      if _is_next_line(c) {
        if c == '\r' {
          self.column += 1;
          self.meet_cr = true;
        } else {
          self.line += 1;
          self.column = 0;
        }
      } else if c != '\0' {
        self.column += 1;
      }
    }
    return ret;
  }
  pub fn feed(&mut self, s: &str) -> Result<Vec<Token>, ParserError> {
    let mut tokens = Vec::new();
    for c in s.chars() {
      tokens.push(self.feed_one(c)?);
    }
    Ok(tokens)
  }
  pub fn end(&mut self) -> Result<Token, ParserError> {
    self.feed_one('\0')
  }
}
impl StreamParser {
  pub fn parse(option: JsonParserOption, s: &str) -> Result<Vec<Token>, ParserError> {
    let mut parser = StreamParser::new(option);
    let mut tokens = parser.feed(s)?;
    tokens.push(parser.end()?);
    Ok(tokens)
  }
}

pub trait StreamParserBase {
  fn get_position(&self) -> usize;
  fn get_line(&self) -> usize;
  fn get_column(&self) -> usize;
  fn get_stage(&self) -> ParserStage;
}
impl StreamParserBase for StreamParser {
  fn get_position(&self) -> usize {
    self.position
  }
  fn get_line(&self) -> usize {
    self.line
  }
  fn get_column(&self) -> usize {
    self.column
  }
  fn get_stage(&self) -> ParserStage {
    match self.state {
      _ValueState::Empty => match self.location {
        _LocationState::RootStart => ParserStage::NotStarted,
        _LocationState::RootEnd | _LocationState::Eof => ParserStage::Ended,
        _ => ParserStage::Parsing,
      },
      _ => ParserStage::Parsing,
    }
  }
}
