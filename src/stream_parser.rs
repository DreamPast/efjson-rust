mod outer {
  pub type StackLength = std::ffi::c_uint;
  pub type Position = usize;
  #[repr(C)]
  pub struct Token {
    pub r#type: u8,
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
#[repr(u8)]
pub enum TokenInfo {
  Whitespace = 1 << 4 | 0x0,
  Eof = 2 << 4 | 0x0,
  Null(u8, bool) = 3 << 4 | 0x0,

  False(u8, bool) = 4 << 4 | 0x0,
  True(u8, bool) = 4 << 4 | 0x1,

  StringStart = 5 << 4 | 0x0,
  StringEnd = 5 << 4 | 0x1,
  StringNormal = 5 << 4 | 0x2,
  StringEscapeStart = 5 << 4 | 0x3,
  StringEscape(char) = 5 << 4 | 0x4,
  StringEscapeUnicodeStart = 5 << 4 | 0x5,
  StringEscapeUnicode(u8, Option<char>) = 5 << 4 | 0x6,
  StringNextLine = 5 << 4 | 0x7,
  StringEscapeHexStart = 5 << 4 | 0x8,
  StringEscapeHex(u8, Option<char>) = 5 << 4 | 0x9,

  NumberIntegerDigit = 6 << 4 | 0x0,
  NumberFractionDigit = 6 << 4 | 0x1,
  NumberExponentDigit = 6 << 4 | 0x2,
  NumberIntegerSign = 6 << 4 | 0x3,
  NumberExponentSign = 6 << 4 | 0x4,
  NumberFractionStart = 6 << 4 | 0x5,
  NumberExponentStart = 6 << 4 | 0x6,
  NumberNan(u8, bool) = 6 << 4 | 0x7,
  NumberInfinity(u8, bool) = 6 << 4 | 0x8,
  NumberHexStart = 6 << 4 | 0x9,
  NumberHex = 6 << 4 | 0xA,
  NumberOctStart = 6 << 4 | 0xB,
  NumberOct = 6 << 4 | 0xC,
  NumberBinStart = 6 << 4 | 0xD,
  NumberBin = 6 << 4 | 0xE,

  ObjectStart = 7 << 4 | 0x0,
  ObjectNext = 7 << 4 | 0x1,
  ObjectValueStart = 7 << 4 | 0x2,
  ObjectEnd = 7 << 4 | 0x3,

  ArrayStart = 8 << 4 | 0x0,
  ArrayNext = 8 << 4 | 0x1,
  ArrayEnd = 8 << 4 | 0x2,

  IdentifierNormal = 9 << 4 | 0x0,
  IdentifierEscapeStart(u8, bool) = 9 << 4 | 0x1,
  IdentifierEscape(u8, Option<char>) = 9 << 4 | 0x2,

  CommentMayStart = 10 << 4 | 0x0,
  CommentSingleLine = 10 << 4 | 0x1,
  CommentMultiLine = 10 << 4 | 0x3,
  CommentMultiLineEnd = 10 << 4 | 0x4,
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
  pub info: TokenInfo,
  pub location: Location,
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
      efjsonStreamParser_init(parser.as_mut_ptr(), option.bits());
      parser.assume_init()
    }
  }

  pub fn feed_one(&mut self, c: char) -> Result<Token, StreamError> {
    let ctoken = unsafe { efjsonStreamParser_feedOne(self, c as u32) };
    let info = match ctoken.r#type {
      0 => {
        return Err(StreamError {
          character: c,
          position: self.get_position(),
          line: self.get_line(),
          column: self.get_column(),
          kind: unsafe { std::mem::transmute(ctoken.extra as u8) },
        });
      }
      0x30 => TokenInfo::Null(ctoken.index, ctoken.done != 0),
      0x40 => TokenInfo::False(ctoken.index, ctoken.done != 0),
      0x41 => TokenInfo::True(ctoken.index, ctoken.done != 0),
      0x54 => TokenInfo::StringEscape(unsafe { char::from_u32_unchecked(ctoken.extra) }),
      0x56 => TokenInfo::StringEscapeUnicode(
        ctoken.index,
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
      ),
      0x59 => TokenInfo::StringEscapeHex(
        ctoken.index,
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
      ),
      0x67 => TokenInfo::NumberNan(ctoken.index, ctoken.done != 0),
      0x68 => TokenInfo::NumberInfinity(ctoken.index, ctoken.done != 0),
      0x91 => TokenInfo::IdentifierEscapeStart(ctoken.index, ctoken.done != 0),
      0x92 => TokenInfo::IdentifierEscape(
        ctoken.index,
        if ctoken.done != 0 {
          Some(unsafe { char::from_u32_unchecked(ctoken.extra) })
        } else {
          None
        },
      ),
      typ => unsafe {
        let mut info = std::mem::zeroed::<TokenInfo>();
        std::ptr::copy_nonoverlapping(
          &typ as *const u8,
          &mut info as *mut TokenInfo as *mut u8,
          std::mem::size_of::<TokenInfo>(),
        );
        info
      },
    };
    return Ok(Token { c, location: ctoken.location, info });
  }
  pub fn end(&mut self) -> Result<Token, StreamError> {
    self.feed_one('\0')
  }

  pub fn feed_iter(&mut self, iter: impl Iterator<Item = char>) -> Result<Vec<Token>, StreamError> {
    let mut tokens = Vec::new();
    for c in iter {
      tokens.push(self.feed_one(c)?);
    }
    Ok(tokens)
  }
  pub fn feed(&mut self, s: &str) -> Result<Vec<Token>, StreamError> {
    self.feed_iter(s.chars())
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
