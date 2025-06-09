mod outer {
  #[repr(u8)]
  pub enum TokenEnum {
    _Error = 0,
    _Whitespace = 1 << 4 | 0x0,
    _Eof = 2 << 4 | 0x0,
    _Null = 3 << 4 | 0x0,

    _False = 4 << 4 | 0x0,
    _True = 4 << 4 | 0x1,

    _StringStart = 5 << 4 | 0x0,
    _StringEnd = 5 << 4 | 0x1,
    _StringNormal = 5 << 4 | 0x2,
    _StringEscapeStart = 5 << 4 | 0x3,
    _StringEscape = 5 << 4 | 0x4,
    _StringEscapeUnicodeStart = 5 << 4 | 0x5,
    _StringEscapeUnicode = 5 << 4 | 0x6,
    _StringNextLine = 5 << 4 | 0x7,
    _StringEscapeHexStart = 5 << 4 | 0x8,
    _StringEscapeHex = 5 << 4 | 0x9,

    _NumberIntegerDigit = 6 << 4 | 0x0,
    _NumberFractionDigit = 6 << 4 | 0x1,
    _NumberExponentDigit = 6 << 4 | 0x2,
    _NumberIntegerSign = 6 << 4 | 0x3,
    _NumberExponentSign = 6 << 4 | 0x4,
    _NumberFractionStart = 6 << 4 | 0x5,
    _NumberExponentStart = 6 << 4 | 0x6,
    _NumberNan = 6 << 4 | 0x7,
    _NumberInfinity = 6 << 4 | 0x8,
    _NumberHexStart = 6 << 4 | 0x9,
    _NumberHex = 6 << 4 | 0xA,
    _NumberOctStart = 6 << 4 | 0xB,
    _NumberOct = 6 << 4 | 0xC,
    _NumberBinStart = 6 << 4 | 0xD,
    _NumberBin = 6 << 4 | 0xE,

    _ObjectStart = 7 << 4 | 0x0,
    _ObjectNext = 7 << 4 | 0x1,
    _ObjectValueStart = 7 << 4 | 0x2,
    _ObjectEnd = 7 << 4 | 0x3,

    _ArrayStart = 8 << 4 | 0x0,
    _ArrayNext = 8 << 4 | 0x1,
    _ArrayEnd = 8 << 4 | 0x2,

    _IdentifierNormal = 9 << 4 | 0x0,
    _IdentifierEscapeStart = 9 << 4 | 0x1,
    _IdentifierEscape = 9 << 4 | 0x2,

    _CommentMayStart = 10 << 4 | 0x0,
    _CommentSingleLine = 10 << 4 | 0x1,
    _CommentMultiLine = 10 << 4 | 0x3,
    _CommentMultiLineEnd = 10 << 4 | 0x4,
  }

  pub type StackLength = std::ffi::c_uint;
  pub type Position = usize;
  #[repr(C)]
  pub struct Token {
    pub r#type: TokenEnum,
    pub location: super::Location,
    pub index: u8,
    pub done: u8,
    pub extra: u32,
  }
  unsafe extern "C" {
    pub fn efjson_stringifyError(error: u8) -> *const std::ffi::c_char;

    pub fn efjsonStreamParser_init(parser: *mut StreamParser, option: u32) -> ();
    pub fn efjsonStreamParser_deinit(parser: *mut StreamParser) -> ();
    pub fn efjsonStreamParser_initCopy(
      parser: *mut StreamParser,
      src: *const StreamParser,
    ) -> std::ffi::c_int;

    pub fn efjsonStreamParser_feedOne(parser: *mut StreamParser, u: u32) -> Token;

    pub fn efjsonStreamParser_getLine(parser: *const StreamParser) -> Position;
    pub fn efjsonStreamParser_getColumn(parser: *const StreamParser) -> Position;
    pub fn efjsonStreamParser_getPosition(parser: *const StreamParser) -> Position;
    pub fn efjsonStreamParser_getStage(parser: *const StreamParser) -> std::ffi::c_int;
  }

  #[repr(C)]
  #[derive(Debug)]
  pub struct StreamParser {
    position: Position,
    line: Position,
    column: Position,
    option: u32,
    location: u8,
    state: u8,
    flag: u8,
    substate: u8,
    escape: u16,
    prev_pair: u16,

    len: StackLength,
    cap: StackLength,
    stack: *mut u8,
  }
}
pub use outer::StreamParser;
use outer::*;

use crate::ParserOption;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorKind {
  AllocFailed = 1,
  TooManyRecursions,
  PositionOverflow,
  InvalidInputUtf,
  InvalidEscapedUtf,
  IncompleteSurrogatePair,
  /* << other >> */
  CommentForbidden = 0x80,
  Eof,
  NonwhitespaceAfterEnd,
  ContentAfterEof,
  TrailingCommaForbidden,
  Unexpected,
  WrongBracket,
  WrongColon,
  /* << array >> */
  CommaInEmptyArray,
  /* << object >> */
  BadIdentifierEscape,
  BadPropertyNameInObject,
  CommaInEmptyObject,
  EmptyValueInObject,
  ExpectedColon,
  InvalidIdentifier,
  InvalidIdentifierEscape,
  RepeatedColon,
  /* << string >> */
  BadEscapeInString,
  BadHexEscapeInString,
  BadUnicodeEscapeInString,
  ControlCharacterForbiddenInString,
  SingleQuoteForbidden,
  /* << number >> */
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
  pub fn stringify(self) -> &'static str {
    unsafe {
      let ptr = efjson_stringifyError(self as u8);
      std::ffi::CStr::from_ptr(ptr).to_str().unwrap_unchecked()
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Category {
  // Error = 0,
  Whitespace = 1,
  Eof,
  Null,
  Boolean,
  String,
  Number,
  Object,
  Array,
  Identifier,
  Comment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Location {
  Root = 0,
  Key,
  Value,
  Element,
  Array,
  Object,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamError {
  pub position: usize,
  pub line: usize,
  pub column: usize,
  pub character: char,
  pub kind: ErrorKind,
}
impl std::fmt::Display for StreamError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "at {}:{}({}), character {} - {}",
      self.line,
      self.column,
      self.position,
      self.character,
      self.kind.stringify(),
    )
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Stage {
  NotStarted = -1,
  Parsing = 0,
  Ended = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenInfo {
  Whitespace,
  Eof,
  Null(bool, u8),

  False(bool, u8),
  True(bool, u8),

  StringStart,
  StringEnd,
  StringNormal,
  StringEscapeStart,
  StringEscape(char),
  StringEscapeUnicodeStart,
  StringEscapeUnicode(Option<char>, u8),
  StringNextLine,
  StringEscapeHexStart,
  StringEscapeHex(Option<char>, u8),

  NumberIntegerDigit,
  NumberFractionDigit,
  NumberExponentDigit,
  NumberIntegerSign,
  NumberExponentSign,
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

  CommentMayStart,
  CommentSingleLine,
  CommentMultiLine,
  CommentMultiLineEnd,
}
impl TokenInfo {
  pub fn get_category(&self) -> Category {
    match self {
      TokenInfo::Whitespace => Category::Whitespace,
      TokenInfo::Eof => Category::Eof,
      TokenInfo::Null(_, _) | TokenInfo::False(_, _) | TokenInfo::True(_, _) => Category::Boolean,
      TokenInfo::StringStart
      | TokenInfo::StringEnd
      | TokenInfo::StringNormal
      | TokenInfo::StringEscapeStart
      | TokenInfo::StringEscape(_)
      | TokenInfo::StringEscapeUnicodeStart
      | TokenInfo::StringEscapeUnicode(_, _)
      | TokenInfo::StringNextLine
      | TokenInfo::StringEscapeHexStart
      | TokenInfo::StringEscapeHex(_, _) => Category::String,
      TokenInfo::NumberIntegerDigit
      | TokenInfo::NumberFractionDigit
      | TokenInfo::NumberExponentDigit
      | TokenInfo::NumberIntegerSign
      | TokenInfo::NumberExponentSign
      | TokenInfo::NumberFractionStart
      | TokenInfo::NumberExponentStart
      | TokenInfo::NumberNan(_, _)
      | TokenInfo::NumberInfinity(_, _)
      | TokenInfo::NumberHexStart
      | TokenInfo::NumberHex
      | TokenInfo::NumberOctStart
      | TokenInfo::NumberOct
      | TokenInfo::NumberBinStart
      | TokenInfo::NumberBin => Category::Number,
      TokenInfo::ObjectStart
      | TokenInfo::ObjectNext
      | TokenInfo::ObjectValueStart
      | TokenInfo::ObjectEnd => Category::Object,
      TokenInfo::ArrayStart | TokenInfo::ArrayNext | TokenInfo::ArrayEnd => Category::Array,
      TokenInfo::IdentifierNormal
      | TokenInfo::IdentifierEscapeStart(_, _)
      | TokenInfo::IdentifierEscape(_, _) => Category::Identifier,
      TokenInfo::CommentMayStart
      | TokenInfo::CommentSingleLine
      | TokenInfo::CommentMultiLine
      | TokenInfo::CommentMultiLineEnd => Category::Comment,
    }
  }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
  pub c: char,
  pub location: Location,
  pub info: TokenInfo,
}

impl StreamParser {
  pub fn get_line(&self) -> usize {
    unsafe { efjsonStreamParser_getLine(self) as usize }
  }
  pub fn get_column(&self) -> usize {
    unsafe { efjsonStreamParser_getColumn(self) as usize }
  }
  pub fn get_position(&self) -> usize {
    unsafe { efjsonStreamParser_getPosition(self) as usize }
  }
  pub fn get_stage(&self) -> Stage {
    match unsafe { efjsonStreamParser_getStage(self) } {
      -1 => Stage::NotStarted,
      0 => Stage::Parsing,
      1 => Stage::Ended,
      _ => unreachable!(),
    }
  }
}
impl StreamParser {
  pub fn new(option: ParserOption) -> Self {
    let mut parser = std::mem::MaybeUninit::<StreamParser>::uninit();
    unsafe {
      efjsonStreamParser_init(parser.as_mut_ptr(), option.into());
      parser.assume_init()
    }
  }

  pub fn feed_one(&mut self, c: char) -> Result<Token, StreamError> {
    let ctoken = unsafe { efjsonStreamParser_feedOne(self, c as u32) };
    let info = match ctoken.r#type {
      TokenEnum::_Error => {
        return Err(StreamError {
          character: c,
          position: self.get_position(),
          line: self.get_line(),
          column: self.get_column(),
          kind: unsafe { std::mem::transmute(ctoken.extra as u8) },
        });
      }
      TokenEnum::_Whitespace => TokenInfo::Whitespace,
      TokenEnum::_Eof => TokenInfo::Eof,
      TokenEnum::_Null => TokenInfo::Null(ctoken.done != 0, ctoken.index),

      TokenEnum::_False => TokenInfo::False(ctoken.done != 0, ctoken.index),
      TokenEnum::_True => TokenInfo::True(ctoken.done != 0, ctoken.index),

      TokenEnum::_StringStart => TokenInfo::StringStart,
      TokenEnum::_StringEnd => TokenInfo::StringEnd,
      TokenEnum::_StringNormal => TokenInfo::StringNormal,
      TokenEnum::_StringEscapeStart => TokenInfo::StringEscapeStart,
      TokenEnum::_StringEscape => {
        TokenInfo::StringEscape(unsafe { char::from_u32_unchecked(ctoken.extra) })
      }
      TokenEnum::_StringEscapeUnicodeStart => TokenInfo::StringEscapeUnicodeStart,
      TokenEnum::_StringEscapeUnicode => TokenInfo::StringEscapeUnicode(
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
        ctoken.index,
      ),
      TokenEnum::_StringNextLine => TokenInfo::StringNextLine,
      TokenEnum::_StringEscapeHexStart => TokenInfo::StringEscapeHexStart,
      TokenEnum::_StringEscapeHex => TokenInfo::StringEscapeHex(
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
        ctoken.index,
      ),

      TokenEnum::_NumberIntegerDigit => TokenInfo::NumberIntegerDigit,
      TokenEnum::_NumberFractionDigit => TokenInfo::NumberFractionDigit,
      TokenEnum::_NumberExponentDigit => TokenInfo::NumberExponentDigit,
      TokenEnum::_NumberIntegerSign => TokenInfo::NumberIntegerSign,
      TokenEnum::_NumberExponentSign => TokenInfo::NumberExponentSign,
      TokenEnum::_NumberFractionStart => TokenInfo::NumberFractionStart,
      TokenEnum::_NumberExponentStart => TokenInfo::NumberExponentStart,
      TokenEnum::_NumberNan => TokenInfo::NumberNan(ctoken.done != 0, ctoken.index),
      TokenEnum::_NumberInfinity => TokenInfo::NumberInfinity(ctoken.done != 0, ctoken.index),
      TokenEnum::_NumberHexStart => TokenInfo::NumberHexStart,
      TokenEnum::_NumberHex => TokenInfo::NumberHex,
      TokenEnum::_NumberOctStart => TokenInfo::NumberOctStart,
      TokenEnum::_NumberOct => TokenInfo::NumberOct,
      TokenEnum::_NumberBinStart => TokenInfo::NumberBinStart,
      TokenEnum::_NumberBin => TokenInfo::NumberBin,

      TokenEnum::_ObjectStart => TokenInfo::ObjectStart,
      TokenEnum::_ObjectNext => TokenInfo::ObjectNext,
      TokenEnum::_ObjectValueStart => TokenInfo::ObjectValueStart,
      TokenEnum::_ObjectEnd => TokenInfo::ObjectEnd,

      TokenEnum::_ArrayStart => TokenInfo::ArrayStart,
      TokenEnum::_ArrayNext => TokenInfo::ArrayNext,
      TokenEnum::_ArrayEnd => TokenInfo::ArrayEnd,

      TokenEnum::_IdentifierNormal => TokenInfo::IdentifierNormal,
      TokenEnum::_IdentifierEscapeStart => {
        TokenInfo::IdentifierEscapeStart(ctoken.done != 0, ctoken.index)
      }
      TokenEnum::_IdentifierEscape => TokenInfo::IdentifierEscape(
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
        ctoken.index,
      ),

      TokenEnum::_CommentMayStart => TokenInfo::CommentMayStart,
      TokenEnum::_CommentSingleLine => TokenInfo::CommentSingleLine,
      TokenEnum::_CommentMultiLine => TokenInfo::CommentMultiLine,
      TokenEnum::_CommentMultiLineEnd => TokenInfo::CommentMultiLineEnd,
    };
    return Ok(Token { c, location: ctoken.location, info });
  }

  pub fn feed(&mut self, s: &str) -> Result<Vec<Token>, StreamError> {
    let mut tokens = Vec::new();
    for c in s.chars() {
      tokens.push(self.feed_one(c)?);
    }
    Ok(tokens)
  }
  pub fn end(&mut self) -> Result<Token, StreamError> {
    self.feed_one('\0')
  }
}
impl Drop for StreamParser {
  fn drop(&mut self) {
    unsafe { efjsonStreamParser_deinit(self) }
  }
}
impl Clone for StreamParser {
  fn clone(&self) -> Self {
    let mut parser = std::mem::MaybeUninit::<StreamParser>::uninit();
    unsafe {
      efjsonStreamParser_initCopy(parser.as_mut_ptr(), self);
      parser.assume_init()
    }
  }
}

impl StreamParser {
  pub fn parse(option: ParserOption, s: &str) -> Result<Vec<Token>, StreamError> {
    let mut parser = StreamParser::new(option);
    let mut tokens = parser.feed(s)?;
    tokens.push(parser.end()?);
    Ok(tokens)
  }
}
