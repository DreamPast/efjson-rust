mod outer {
  pub type StackLength = std::ffi::c_uint;
  pub type Position = usize;
  #[repr(C)]
  pub struct Token {
    pub r#type: u8,
    _dummy: u8,
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
    pub fn efjsonStreamParser_getLocation(parser: *const StreamParser) -> super::Location;
    pub fn efjsonStreamParser_getStage(parser: *const StreamParser) -> std::ffi::c_int;
  }

  /**
  A state machine capable of parsing JSON data one code point at a time and outputting Token information.

  Note: The underlying implementation of this class is in C, and all unsafe parts have been encapsulated.
  */
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

/**
The error kind of the parser.

It's not commended to use this enum directly, instead use the `stringify` method to get a human-readable string.
 */
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
  Eof = 0x80,
  NonwhitespaceAfterEnd,
  ContentAfterEof,
  TrailingCommaForbidden,
  Unexpected,
  WrongBracket,
  WrongColon,
  EmptyValue,
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
  LoneDecimalPoint,
  /* << comment >> */
  CommentForbidden,
  CommentNotClosed,
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
impl Into<Box<dyn std::error::Error + Send + Sync>> for StreamError {
  fn into(self) -> Box<dyn std::error::Error + Send + Sync> {
    format!("{}", self).into()
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
#[repr(u8, C)]
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
    unsafe { std::mem::transmute(*(self as *const TokenInfo as *const u8) >> 4) }
  }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
  pub c: char,
  pub info: TokenInfo,
}
impl Token {
  pub fn is_space(&self) -> bool {
    matches!(
      self.info,
      TokenInfo::Whitespace
        | TokenInfo::CommentMayStart
        | TokenInfo::CommentMultiLine
        | TokenInfo::CommentSingleLine
        | TokenInfo::CommentMultiLineEnd
        | TokenInfo::Eof
    )
  }
}

impl StreamParser {
  /** Get the current line number (starting from 0) */
  pub fn get_line(&self) -> usize {
    unsafe { efjsonStreamParser_getLine(self) as usize }
  }
  /** Get the current column number (starting from 0) */
  pub fn get_column(&self) -> usize {
    unsafe { efjsonStreamParser_getColumn(self) as usize }
  }
  /** Get the current position number (starting from 0) */
  pub fn get_position(&self) -> usize {
    unsafe { efjsonStreamParser_getPosition(self) as usize }
  }
  /** Get the current location */
  pub fn get_location(&self) -> Location {
    unsafe { efjsonStreamParser_getLocation(self) }
  }
  /** Get the current stage */
  pub fn get_stage(&self) -> Stage {
    match unsafe { efjsonStreamParser_getStage(self) } {
      -1 => Stage::NotStarted,
      0 => Stage::Parsing,
      1 => Stage::Ended,
      _ => unsafe { std::hint::unreachable_unchecked() },
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

  /**
  Feed a single character to the parser and return the next token.

  # Note
  If the string is ended, you need to explicitly pass `'\0'` to notify the parser.

  # Example
  ```rust
  let parser = StreamParser::new();
  println!("{:?}", parser.feed_one('n'));
  ```

  # Panics
  The function does not panic.

  # Errors
  If the character is invalid or the parser encounters an error, it will return a `StreamError`.
  And the state machine will remain unchanged. You can choose to ignore this error and continue parsing,
  but note that this may lead to incorrect parsing results.
   */
  pub fn feed_one(&mut self, c: char) -> Result<Token, StreamError> {
    let ctoken = unsafe { efjsonStreamParser_feedOne(self, c as u32) };
    let info = match ctoken.r#type {
      0x00 => {
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
        let mut info = std::mem::MaybeUninit::zeroed();
        (info.as_mut_ptr() as *mut u8).write(typ);
        info.assume_init()
      },
    };
    return Ok(Token { c, info });
  }

  /**
  Feed an iterator of characters to the parser and return a vector of tokens.

  # Note
  If the string is ended, you need to explicitly pass `'\0'` to notify the parser.

  # Panics
  Panics if the number of tokens exceeds `isize::MAX`.

  # Errors
  If the character is invalid or the parser encounters an error, it will return a `StreamError`.
  And the state machine will remain unchanged,
  but the function will not roll back the characters that have already been fed.
  */
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

impl StreamParser {
  pub fn parse(option: ParserOption, s: &str) -> Result<Vec<Token>, StreamError> {
    let mut parser = StreamParser::new(option);
    let mut tokens = parser.feed(s)?;
    tokens.push(parser.feed_one('\0')?);
    Ok(tokens)
  }
  pub fn parse_iter(
    option: ParserOption,
    iter: impl Iterator<Item = char>,
  ) -> Result<Vec<Token>, StreamError> {
    let mut parser = StreamParser::new(option);
    let mut tokens = parser.feed_iter(iter)?;
    tokens.push(parser.feed_one('\0')?);
    Ok(tokens)
  }
  pub fn create_iter(
    option: ParserOption,
    iter: impl Iterator<Item = char>,
  ) -> impl Iterator<Item = Result<Token, StreamError>> {
    let mut parser = StreamParser::new(option);
    iter.chain(std::iter::once('\0')).map(move |c| parser.feed_one(c))
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
unsafe impl Send for StreamParser {}
unsafe impl Sync for StreamParser {}
