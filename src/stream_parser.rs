use std::hint::unreachable_unchecked;

#[derive(Default, Clone)]
pub struct JsonOption {
  // << white space >>
  /**
   * whether to accept whitespace in JSON5
   */
  pub accept_json5_whitespace: bool,

  // << array >>
  /**
   * whether to accept a single trailing comma in array
   * @example '[1,]'
   */
  pub accept_trailing_comma_in_array: bool,

  // << object >>
  /**
   * whether to accept a single trailing comma in object
   * @example '{"a":1,}'
   */
  pub accept_trailing_comma_in_object: bool,
  /**
   * whether to accept identifier key in object
   * @example '{a:1}'
   */
  pub accept_identifier_key: bool,

  // << string >>
  /**
   * whether to accept single quote in string
   * @example "'a'"
   */
  pub accept_single_quote: bool,
  /**
   * whether to accept multi-line string
   * @example '"a\\\nb"'
   */
  pub accept_multiline_string: bool,
  /**
   * whether to accept JSON5 string escape
   * @example '"\\x01"', '\\v', '\\0'
   */
  pub accpet_json5_string_escape: bool,

  // << number >>
  /**
   * whether to accept positive sign in number
   * @example '+1', '+0'
   */
  pub accept_positive_sign: bool,
  /**
   * whether to accept empty fraction in number
   * @example '1.', '0.'
   */
  pub accept_empty_fraction: bool,
  /**
   * whether to accept empty integer in number
   * @example '.1', '.0'
   */
  pub accept_empty_integer: bool,
  /**
   * whether to accept NaN
   */
  pub accept_nan: bool,
  /**
   * whether to accept Infinity
   */
  pub accept_infinity: bool,
  /**
   * whether to accept hexadecimal integer
   * @example '0x1', '0x0'
   */
  pub accept_hexadecimal_integer: bool,
  /**
   * whether to accept octal integer
   * @example '0o1', '0o0'
   */
  pub accept_octal_integer: bool,
  /**
   * whether to accept binary integer
   * @example '0b1', '0b0'
   */
  pub accept_binary_integer: bool,

  // << comment >>
  /**
   * whether to accept single line comment
   * @example '// a comment'
   */
  pub accept_single_line_comment: bool,
  /**
   * whether to accept multi-line comment
   */
  pub accpet_multi_line_comment: bool,
}
impl JsonOption {
  pub fn new_jsonc() -> Self {
    JsonOption {
      // << comment >>
      accept_single_line_comment: true,
      accpet_multi_line_comment: true,
      ..Default::default()
    }
  }
  pub fn new_json5() -> Self {
    JsonOption {
      // << white space >>
      accept_json5_whitespace: true,
      // << array >>
      accept_trailing_comma_in_array: true,
      // << object >>
      accept_trailing_comma_in_object: true,
      accept_identifier_key: true,
      // << string >>
      accept_single_quote: true,
      accept_multiline_string: true,
      accpet_json5_string_escape: true,
      // << number >>
      accept_positive_sign: true,
      accept_empty_fraction: true,
      accept_empty_integer: true,
      accept_nan: true,
      accept_infinity: true,
      accept_hexadecimal_integer: true,
      // << comment >>
      accept_single_line_comment: true,
      accpet_multi_line_comment: true,
      ..Default::default()
    }
  }
  pub fn new_full() -> Self {
    return JsonOption {
      accept_octal_integer: true,
      accept_binary_integer: true,
      ..Self::new_json5()
    };
  }
}

#[derive(Clone)]
enum _ValueNumberState {
  Sign,
  Zero,
  Digit,
}
#[derive(Clone)]
enum _ValueExponentState {
  Desire,
  Sign,
  Digit,
}

#[derive(Clone)]
enum _ValueState {
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

#[derive(Clone, Copy)]
enum _LocationState {
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

#[derive(Clone, Copy, Debug)]
pub enum JsonLocation {
  Root,
  Key,
  Value,
  Element,
  Object,
  Array,
}
#[derive(Clone, Copy, Debug)]
pub enum JsonTokenInfo {
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
pub struct JsonToken {
  pub c: char,
  pub info: JsonTokenInfo,
  pub location: JsonLocation,
}

pub type JsonParserPosition = u32;
#[derive(Clone)]
pub struct JsonStreamParser {
  position: JsonParserPosition,
  line: JsonParserPosition,
  column: JsonParserPosition,
  meet_cr: bool,
  location: _LocationState,
  state: _ValueState,
  stack: Vec<_LocationState>,
  option: JsonOption,
}

type JsonStreamParserError = &'static str;

mod char_check {
  use std::hint::unreachable_unchecked;

  pub fn _is_whitespace(c: char, fit_json5: bool) -> bool {
    static _EXTRA_WHITESPACE: &str = "\u{000B}\u{000C}\u{00A0}\u{FEFF}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}";
    " \t\n\r".contains(c) || (fit_json5 && _EXTRA_WHITESPACE.contains(c))
  }
  pub fn _is_next_line(c: char) -> bool {
    "\n\u{2028}\u{2029}\r".contains(c)
  }
  pub fn _is_number_separator(c: char, fit_json5: bool) -> bool {
    _is_whitespace(c, fit_json5) || "\0,]}/".contains(c)
  }
  pub fn _is_control(c: char) -> bool {
    c >= '\u{0000}' && c <= '\u{001F}'
  }
  pub fn _is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
  }
  #[rustfmt::skip]
  static _IDENTIFIER_START1: [[u16;2];383] = [
    [ 0x0024, 0x0024 ], [ 0x0041, 0x005A ], [ 0x005F, 0x005F ], [ 0x0061, 0x007A ], [ 0x00AA, 0x00AA ],
    [ 0x00B5, 0x00B5 ], [ 0x00BA, 0x00BA ], [ 0x00C0, 0x00D6 ], [ 0x00D8, 0x00F6 ], [ 0x00F8, 0x02C1 ],
    [ 0x02C6, 0x02D1 ], [ 0x02E0, 0x02E4 ], [ 0x02EC, 0x02EC ], [ 0x02EE, 0x02EE ], [ 0x0370, 0x0374 ],
    [ 0x0376, 0x0377 ], [ 0x037A, 0x037D ], [ 0x037F, 0x037F ], [ 0x0386, 0x0386 ], [ 0x0388, 0x038A ],
    [ 0x038C, 0x038C ], [ 0x038E, 0x03A1 ], [ 0x03A3, 0x03F5 ], [ 0x03F7, 0x0481 ], [ 0x048A, 0x052F ],
    [ 0x0531, 0x0556 ], [ 0x0559, 0x0559 ], [ 0x0560, 0x0588 ], [ 0x05D0, 0x05EA ], [ 0x05EF, 0x05F2 ],
    [ 0x0620, 0x064A ], [ 0x066E, 0x066F ], [ 0x0671, 0x06D3 ], [ 0x06D5, 0x06D5 ], [ 0x06E5, 0x06E6 ],
    [ 0x06EE, 0x06EF ], [ 0x06FA, 0x06FC ], [ 0x06FF, 0x06FF ], [ 0x0710, 0x0710 ], [ 0x0712, 0x072F ],
    [ 0x074D, 0x07A5 ], [ 0x07B1, 0x07B1 ], [ 0x07CA, 0x07EA ], [ 0x07F4, 0x07F5 ], [ 0x07FA, 0x07FA ],
    [ 0x0800, 0x0815 ], [ 0x081A, 0x081A ], [ 0x0824, 0x0824 ], [ 0x0828, 0x0828 ], [ 0x0840, 0x0858 ],
    [ 0x0860, 0x086A ], [ 0x0870, 0x0887 ], [ 0x0889, 0x088E ], [ 0x08A0, 0x08C9 ], [ 0x0904, 0x0939 ],
    [ 0x093D, 0x093D ], [ 0x0950, 0x0950 ], [ 0x0958, 0x0961 ], [ 0x0971, 0x0980 ], [ 0x0985, 0x098C ],
    [ 0x098F, 0x0990 ], [ 0x0993, 0x09A8 ], [ 0x09AA, 0x09B0 ], [ 0x09B2, 0x09B2 ], [ 0x09B6, 0x09B9 ],
    [ 0x09BD, 0x09BD ], [ 0x09CE, 0x09CE ], [ 0x09DC, 0x09DD ], [ 0x09DF, 0x09E1 ], [ 0x09F0, 0x09F1 ],
    [ 0x09FC, 0x09FC ], [ 0x0A05, 0x0A0A ], [ 0x0A0F, 0x0A10 ], [ 0x0A13, 0x0A28 ], [ 0x0A2A, 0x0A30 ],
    [ 0x0A32, 0x0A33 ], [ 0x0A35, 0x0A36 ], [ 0x0A38, 0x0A39 ], [ 0x0A59, 0x0A5C ], [ 0x0A5E, 0x0A5E ],
    [ 0x0A72, 0x0A74 ], [ 0x0A85, 0x0A8D ], [ 0x0A8F, 0x0A91 ], [ 0x0A93, 0x0AA8 ], [ 0x0AAA, 0x0AB0 ],
    [ 0x0AB2, 0x0AB3 ], [ 0x0AB5, 0x0AB9 ], [ 0x0ABD, 0x0ABD ], [ 0x0AD0, 0x0AD0 ], [ 0x0AE0, 0x0AE1 ],
    [ 0x0AF9, 0x0AF9 ], [ 0x0B05, 0x0B0C ], [ 0x0B0F, 0x0B10 ], [ 0x0B13, 0x0B28 ], [ 0x0B2A, 0x0B30 ],
    [ 0x0B32, 0x0B33 ], [ 0x0B35, 0x0B39 ], [ 0x0B3D, 0x0B3D ], [ 0x0B5C, 0x0B5D ], [ 0x0B5F, 0x0B61 ],
    [ 0x0B71, 0x0B71 ], [ 0x0B83, 0x0B83 ], [ 0x0B85, 0x0B8A ], [ 0x0B8E, 0x0B90 ], [ 0x0B92, 0x0B95 ],
    [ 0x0B99, 0x0B9A ], [ 0x0B9C, 0x0B9C ], [ 0x0B9E, 0x0B9F ], [ 0x0BA3, 0x0BA4 ], [ 0x0BA8, 0x0BAA ],
    [ 0x0BAE, 0x0BB9 ], [ 0x0BD0, 0x0BD0 ], [ 0x0C05, 0x0C0C ], [ 0x0C0E, 0x0C10 ], [ 0x0C12, 0x0C28 ],
    [ 0x0C2A, 0x0C39 ], [ 0x0C3D, 0x0C3D ], [ 0x0C58, 0x0C5A ], [ 0x0C5D, 0x0C5D ], [ 0x0C60, 0x0C61 ],
    [ 0x0C80, 0x0C80 ], [ 0x0C85, 0x0C8C ], [ 0x0C8E, 0x0C90 ], [ 0x0C92, 0x0CA8 ], [ 0x0CAA, 0x0CB3 ],
    [ 0x0CB5, 0x0CB9 ], [ 0x0CBD, 0x0CBD ], [ 0x0CDD, 0x0CDE ], [ 0x0CE0, 0x0CE1 ], [ 0x0CF1, 0x0CF2 ],
    [ 0x0D04, 0x0D0C ], [ 0x0D0E, 0x0D10 ], [ 0x0D12, 0x0D3A ], [ 0x0D3D, 0x0D3D ], [ 0x0D4E, 0x0D4E ],
    [ 0x0D54, 0x0D56 ], [ 0x0D5F, 0x0D61 ], [ 0x0D7A, 0x0D7F ], [ 0x0D85, 0x0D96 ], [ 0x0D9A, 0x0DB1 ],
    [ 0x0DB3, 0x0DBB ], [ 0x0DBD, 0x0DBD ], [ 0x0DC0, 0x0DC6 ], [ 0x0E01, 0x0E30 ], [ 0x0E32, 0x0E33 ],
    [ 0x0E40, 0x0E46 ], [ 0x0E81, 0x0E82 ], [ 0x0E84, 0x0E84 ], [ 0x0E86, 0x0E8A ], [ 0x0E8C, 0x0EA3 ],
    [ 0x0EA5, 0x0EA5 ], [ 0x0EA7, 0x0EB0 ], [ 0x0EB2, 0x0EB3 ], [ 0x0EBD, 0x0EBD ], [ 0x0EC0, 0x0EC4 ],
    [ 0x0EC6, 0x0EC6 ], [ 0x0EDC, 0x0EDF ], [ 0x0F00, 0x0F00 ], [ 0x0F40, 0x0F47 ], [ 0x0F49, 0x0F6C ],
    [ 0x0F88, 0x0F8C ], [ 0x1000, 0x102A ], [ 0x103F, 0x103F ], [ 0x1050, 0x1055 ], [ 0x105A, 0x105D ],
    [ 0x1061, 0x1061 ], [ 0x1065, 0x1066 ], [ 0x106E, 0x1070 ], [ 0x1075, 0x1081 ], [ 0x108E, 0x108E ],
    [ 0x10A0, 0x10C5 ], [ 0x10C7, 0x10C7 ], [ 0x10CD, 0x10CD ], [ 0x10D0, 0x10FA ], [ 0x10FC, 0x1248 ],
    [ 0x124A, 0x124D ], [ 0x1250, 0x1256 ], [ 0x1258, 0x1258 ], [ 0x125A, 0x125D ], [ 0x1260, 0x1288 ],
    [ 0x128A, 0x128D ], [ 0x1290, 0x12B0 ], [ 0x12B2, 0x12B5 ], [ 0x12B8, 0x12BE ], [ 0x12C0, 0x12C0 ],
    [ 0x12C2, 0x12C5 ], [ 0x12C8, 0x12D6 ], [ 0x12D8, 0x1310 ], [ 0x1312, 0x1315 ], [ 0x1318, 0x135A ],
    [ 0x1380, 0x138F ], [ 0x13A0, 0x13F5 ], [ 0x13F8, 0x13FD ], [ 0x1401, 0x166C ], [ 0x166F, 0x167F ],
    [ 0x1681, 0x169A ], [ 0x16A0, 0x16EA ], [ 0x16EE, 0x16F8 ], [ 0x1700, 0x1711 ], [ 0x171F, 0x1731 ],
    [ 0x1740, 0x1751 ], [ 0x1760, 0x176C ], [ 0x176E, 0x1770 ], [ 0x1780, 0x17B3 ], [ 0x17D7, 0x17D7 ],
    [ 0x17DC, 0x17DC ], [ 0x1820, 0x1878 ], [ 0x1880, 0x1884 ], [ 0x1887, 0x18A8 ], [ 0x18AA, 0x18AA ],
    [ 0x18B0, 0x18F5 ], [ 0x1900, 0x191E ], [ 0x1950, 0x196D ], [ 0x1970, 0x1974 ], [ 0x1980, 0x19AB ],
    [ 0x19B0, 0x19C9 ], [ 0x1A00, 0x1A16 ], [ 0x1A20, 0x1A54 ], [ 0x1AA7, 0x1AA7 ], [ 0x1B05, 0x1B33 ],
    [ 0x1B45, 0x1B4C ], [ 0x1B83, 0x1BA0 ], [ 0x1BAE, 0x1BAF ], [ 0x1BBA, 0x1BE5 ], [ 0x1C00, 0x1C23 ],
    [ 0x1C4D, 0x1C4F ], [ 0x1C5A, 0x1C7D ], [ 0x1C80, 0x1C8A ], [ 0x1C90, 0x1CBA ], [ 0x1CBD, 0x1CBF ],
    [ 0x1CE9, 0x1CEC ], [ 0x1CEE, 0x1CF3 ], [ 0x1CF5, 0x1CF6 ], [ 0x1CFA, 0x1CFA ], [ 0x1D00, 0x1DBF ],
    [ 0x1E00, 0x1F15 ], [ 0x1F18, 0x1F1D ], [ 0x1F20, 0x1F45 ], [ 0x1F48, 0x1F4D ], [ 0x1F50, 0x1F57 ],
    [ 0x1F59, 0x1F59 ], [ 0x1F5B, 0x1F5B ], [ 0x1F5D, 0x1F5D ], [ 0x1F5F, 0x1F7D ], [ 0x1F80, 0x1FB4 ],
    [ 0x1FB6, 0x1FBC ], [ 0x1FBE, 0x1FBE ], [ 0x1FC2, 0x1FC4 ], [ 0x1FC6, 0x1FCC ], [ 0x1FD0, 0x1FD3 ],
    [ 0x1FD6, 0x1FDB ], [ 0x1FE0, 0x1FEC ], [ 0x1FF2, 0x1FF4 ], [ 0x1FF6, 0x1FFC ], [ 0x2071, 0x2071 ],
    [ 0x207F, 0x207F ], [ 0x2090, 0x209C ], [ 0x2102, 0x2102 ], [ 0x2107, 0x2107 ], [ 0x210A, 0x2113 ],
    [ 0x2115, 0x2115 ], [ 0x2119, 0x211D ], [ 0x2124, 0x2124 ], [ 0x2126, 0x2126 ], [ 0x2128, 0x2128 ],
    [ 0x212A, 0x212D ], [ 0x212F, 0x2139 ], [ 0x213C, 0x213F ], [ 0x2145, 0x2149 ], [ 0x214E, 0x214E ],
    [ 0x2160, 0x2188 ], [ 0x2C00, 0x2CE4 ], [ 0x2CEB, 0x2CEE ], [ 0x2CF2, 0x2CF3 ], [ 0x2D00, 0x2D25 ],
    [ 0x2D27, 0x2D27 ], [ 0x2D2D, 0x2D2D ], [ 0x2D30, 0x2D67 ], [ 0x2D6F, 0x2D6F ], [ 0x2D80, 0x2D96 ],
    [ 0x2DA0, 0x2DA6 ], [ 0x2DA8, 0x2DAE ], [ 0x2DB0, 0x2DB6 ], [ 0x2DB8, 0x2DBE ], [ 0x2DC0, 0x2DC6 ],
    [ 0x2DC8, 0x2DCE ], [ 0x2DD0, 0x2DD6 ], [ 0x2DD8, 0x2DDE ], [ 0x2E2F, 0x2E2F ], [ 0x3005, 0x3007 ],
    [ 0x3021, 0x3029 ], [ 0x3031, 0x3035 ], [ 0x3038, 0x303C ], [ 0x3041, 0x3096 ], [ 0x309D, 0x309F ],
    [ 0x30A1, 0x30FA ], [ 0x30FC, 0x30FF ], [ 0x3105, 0x312F ], [ 0x3131, 0x318E ], [ 0x31A0, 0x31BF ],
    [ 0x31F0, 0x31FF ], [ 0x3400, 0x4DBF ], [ 0x4E00, 0xA48C ], [ 0xA4D0, 0xA4FD ], [ 0xA500, 0xA60C ],
    [ 0xA610, 0xA61F ], [ 0xA62A, 0xA62B ], [ 0xA640, 0xA66E ], [ 0xA67F, 0xA69D ], [ 0xA6A0, 0xA6EF ],
    [ 0xA717, 0xA71F ], [ 0xA722, 0xA788 ], [ 0xA78B, 0xA7CD ], [ 0xA7D0, 0xA7D1 ], [ 0xA7D3, 0xA7D3 ],
    [ 0xA7D5, 0xA7DC ], [ 0xA7F2, 0xA801 ], [ 0xA803, 0xA805 ], [ 0xA807, 0xA80A ], [ 0xA80C, 0xA822 ],
    [ 0xA840, 0xA873 ], [ 0xA882, 0xA8B3 ], [ 0xA8F2, 0xA8F7 ], [ 0xA8FB, 0xA8FB ], [ 0xA8FD, 0xA8FE ],
    [ 0xA90A, 0xA925 ], [ 0xA930, 0xA946 ], [ 0xA960, 0xA97C ], [ 0xA984, 0xA9B2 ], [ 0xA9CF, 0xA9CF ],
    [ 0xA9E0, 0xA9E4 ], [ 0xA9E6, 0xA9EF ], [ 0xA9FA, 0xA9FE ], [ 0xAA00, 0xAA28 ], [ 0xAA40, 0xAA42 ],
    [ 0xAA44, 0xAA4B ], [ 0xAA60, 0xAA76 ], [ 0xAA7A, 0xAA7A ], [ 0xAA7E, 0xAAAF ], [ 0xAAB1, 0xAAB1 ],
    [ 0xAAB5, 0xAAB6 ], [ 0xAAB9, 0xAABD ], [ 0xAAC0, 0xAAC0 ], [ 0xAAC2, 0xAAC2 ], [ 0xAADB, 0xAADD ],
    [ 0xAAE0, 0xAAEA ], [ 0xAAF2, 0xAAF4 ], [ 0xAB01, 0xAB06 ], [ 0xAB09, 0xAB0E ], [ 0xAB11, 0xAB16 ],
    [ 0xAB20, 0xAB26 ], [ 0xAB28, 0xAB2E ], [ 0xAB30, 0xAB5A ], [ 0xAB5C, 0xAB69 ], [ 0xAB70, 0xABE2 ],
    [ 0xAC00, 0xD7A3 ], [ 0xD7B0, 0xD7C6 ], [ 0xD7CB, 0xD7FB ], [ 0xF900, 0xFA6D ], [ 0xFA70, 0xFAD9 ],
    [ 0xFB00, 0xFB06 ], [ 0xFB13, 0xFB17 ], [ 0xFB1D, 0xFB1D ], [ 0xFB1F, 0xFB28 ], [ 0xFB2A, 0xFB36 ],
    [ 0xFB38, 0xFB3C ], [ 0xFB3E, 0xFB3E ], [ 0xFB40, 0xFB41 ], [ 0xFB43, 0xFB44 ], [ 0xFB46, 0xFBB1 ],
    [ 0xFBD3, 0xFD3D ], [ 0xFD50, 0xFD8F ], [ 0xFD92, 0xFDC7 ], [ 0xFDF0, 0xFDFB ], [ 0xFE70, 0xFE74 ],
    [ 0xFE76, 0xFEFC ], [ 0xFF21, 0xFF3A ], [ 0xFF41, 0xFF5A ], [ 0xFF66, 0xFFBE ], [ 0xFFC2, 0xFFC7 ],
    [ 0xFFCA, 0xFFCF ], [ 0xFFD2, 0xFFD7 ], [ 0xFFDA, 0xFFDC ]
  ];
  #[rustfmt::skip]
  static _IDENTIFIER_START2:[[u16;2];290] = [
    [ 0x0000, 0x000B ], [ 0x000D, 0x0026 ], [ 0x0028, 0x003A ], [ 0x003C, 0x003D ], [ 0x003F, 0x004D ],
    [ 0x0050, 0x005D ], [ 0x0080, 0x00FA ], [ 0x0140, 0x0174 ], [ 0x0280, 0x029C ], [ 0x02A0, 0x02D0 ],
    [ 0x0300, 0x031F ], [ 0x032D, 0x034A ], [ 0x0350, 0x0375 ], [ 0x0380, 0x039D ], [ 0x03A0, 0x03C3 ],
    [ 0x03C8, 0x03CF ], [ 0x03D1, 0x03D5 ], [ 0x0400, 0x049D ], [ 0x04B0, 0x04D3 ], [ 0x04D8, 0x04FB ],
    [ 0x0500, 0x0527 ], [ 0x0530, 0x0563 ], [ 0x0570, 0x057A ], [ 0x057C, 0x058A ], [ 0x058C, 0x0592 ],
    [ 0x0594, 0x0595 ], [ 0x0597, 0x05A1 ], [ 0x05A3, 0x05B1 ], [ 0x05B3, 0x05B9 ], [ 0x05BB, 0x05BC ],
    [ 0x05C0, 0x05F3 ], [ 0x0600, 0x0736 ], [ 0x0740, 0x0755 ], [ 0x0760, 0x0767 ], [ 0x0780, 0x0785 ],
    [ 0x0787, 0x07B0 ], [ 0x07B2, 0x07BA ], [ 0x0800, 0x0805 ], [ 0x0808, 0x0808 ], [ 0x080A, 0x0835 ],
    [ 0x0837, 0x0838 ], [ 0x083C, 0x083C ], [ 0x083F, 0x0855 ], [ 0x0860, 0x0876 ], [ 0x0880, 0x089E ],
    [ 0x08E0, 0x08F2 ], [ 0x08F4, 0x08F5 ], [ 0x0900, 0x0915 ], [ 0x0920, 0x0939 ], [ 0x0980, 0x09B7 ],
    [ 0x09BE, 0x09BF ], [ 0x0A00, 0x0A00 ], [ 0x0A10, 0x0A13 ], [ 0x0A15, 0x0A17 ], [ 0x0A19, 0x0A35 ],
    [ 0x0A60, 0x0A7C ], [ 0x0A80, 0x0A9C ], [ 0x0AC0, 0x0AC7 ], [ 0x0AC9, 0x0AE4 ], [ 0x0B00, 0x0B35 ],
    [ 0x0B40, 0x0B55 ], [ 0x0B60, 0x0B72 ], [ 0x0B80, 0x0B91 ], [ 0x0C00, 0x0C48 ], [ 0x0C80, 0x0CB2 ],
    [ 0x0CC0, 0x0CF2 ], [ 0x0D00, 0x0D23 ], [ 0x0D4A, 0x0D65 ], [ 0x0D6F, 0x0D85 ], [ 0x0E80, 0x0EA9 ],
    [ 0x0EB0, 0x0EB1 ], [ 0x0EC2, 0x0EC4 ], [ 0x0F00, 0x0F1C ], [ 0x0F27, 0x0F27 ], [ 0x0F30, 0x0F45 ],
    [ 0x0F70, 0x0F81 ], [ 0x0FB0, 0x0FC4 ], [ 0x0FE0, 0x0FF6 ], [ 0x1003, 0x1037 ], [ 0x1071, 0x1072 ],
    [ 0x1075, 0x1075 ], [ 0x1083, 0x10AF ], [ 0x10D0, 0x10E8 ], [ 0x1103, 0x1126 ], [ 0x1144, 0x1144 ],
    [ 0x1147, 0x1147 ], [ 0x1150, 0x1172 ], [ 0x1176, 0x1176 ], [ 0x1183, 0x11B2 ], [ 0x11C1, 0x11C4 ],
    [ 0x11DA, 0x11DA ], [ 0x11DC, 0x11DC ], [ 0x1200, 0x1211 ], [ 0x1213, 0x122B ], [ 0x123F, 0x1240 ],
    [ 0x1280, 0x1286 ], [ 0x1288, 0x1288 ], [ 0x128A, 0x128D ], [ 0x128F, 0x129D ], [ 0x129F, 0x12A8 ],
    [ 0x12B0, 0x12DE ], [ 0x1305, 0x130C ], [ 0x130F, 0x1310 ], [ 0x1313, 0x1328 ], [ 0x132A, 0x1330 ],
    [ 0x1332, 0x1333 ], [ 0x1335, 0x1339 ], [ 0x133D, 0x133D ], [ 0x1350, 0x1350 ], [ 0x135D, 0x1361 ],
    [ 0x1380, 0x1389 ], [ 0x138B, 0x138B ], [ 0x138E, 0x138E ], [ 0x1390, 0x13B5 ], [ 0x13B7, 0x13B7 ],
    [ 0x13D1, 0x13D1 ], [ 0x13D3, 0x13D3 ], [ 0x1400, 0x1434 ], [ 0x1447, 0x144A ], [ 0x145F, 0x1461 ],
    [ 0x1480, 0x14AF ], [ 0x14C4, 0x14C5 ], [ 0x14C7, 0x14C7 ], [ 0x1580, 0x15AE ], [ 0x15D8, 0x15DB ],
    [ 0x1600, 0x162F ], [ 0x1644, 0x1644 ], [ 0x1680, 0x16AA ], [ 0x16B8, 0x16B8 ], [ 0x1700, 0x171A ],
    [ 0x1740, 0x1746 ], [ 0x1800, 0x182B ], [ 0x18A0, 0x18DF ], [ 0x18FF, 0x1906 ], [ 0x1909, 0x1909 ],
    [ 0x190C, 0x1913 ], [ 0x1915, 0x1916 ], [ 0x1918, 0x192F ], [ 0x193F, 0x193F ], [ 0x1941, 0x1941 ],
    [ 0x19A0, 0x19A7 ], [ 0x19AA, 0x19D0 ], [ 0x19E1, 0x19E1 ], [ 0x19E3, 0x19E3 ], [ 0x1A00, 0x1A00 ],
    [ 0x1A0B, 0x1A32 ], [ 0x1A3A, 0x1A3A ], [ 0x1A50, 0x1A50 ], [ 0x1A5C, 0x1A89 ], [ 0x1A9D, 0x1A9D ],
    [ 0x1AB0, 0x1AF8 ], [ 0x1BC0, 0x1BE0 ], [ 0x1C00, 0x1C08 ], [ 0x1C0A, 0x1C2E ], [ 0x1C40, 0x1C40 ],
    [ 0x1C72, 0x1C8F ], [ 0x1D00, 0x1D06 ], [ 0x1D08, 0x1D09 ], [ 0x1D0B, 0x1D30 ], [ 0x1D46, 0x1D46 ],
    [ 0x1D60, 0x1D65 ], [ 0x1D67, 0x1D68 ], [ 0x1D6A, 0x1D89 ], [ 0x1D98, 0x1D98 ], [ 0x1EE0, 0x1EF2 ],
    [ 0x1F02, 0x1F02 ], [ 0x1F04, 0x1F10 ], [ 0x1F12, 0x1F33 ], [ 0x1FB0, 0x1FB0 ], [ 0x2000, 0x2399 ],
    [ 0x2400, 0x246E ], [ 0x2480, 0x2543 ], [ 0x2F90, 0x2FF0 ], [ 0x3000, 0x342F ], [ 0x3441, 0x3446 ],
    [ 0x3460, 0x43FA ], [ 0x4400, 0x4646 ], [ 0x6100, 0x611D ], [ 0x6800, 0x6A38 ], [ 0x6A40, 0x6A5E ],
    [ 0x6A70, 0x6ABE ], [ 0x6AD0, 0x6AED ], [ 0x6B00, 0x6B2F ], [ 0x6B40, 0x6B43 ], [ 0x6B63, 0x6B77 ],
    [ 0x6B7D, 0x6B8F ], [ 0x6D40, 0x6D6C ], [ 0x6E40, 0x6E7F ], [ 0x6F00, 0x6F4A ], [ 0x6F50, 0x6F50 ],
    [ 0x6F93, 0x6F9F ], [ 0x6FE0, 0x6FE1 ], [ 0x6FE3, 0x6FE3 ], [ 0x7000, 0x87F7 ], [ 0x8800, 0x8CD5 ],
    [ 0x8CFF, 0x8D08 ], [ 0xAFF0, 0xAFF3 ], [ 0xAFF5, 0xAFFB ], [ 0xAFFD, 0xAFFE ], [ 0xB000, 0xB122 ],
    [ 0xB132, 0xB132 ], [ 0xB150, 0xB152 ], [ 0xB155, 0xB155 ], [ 0xB164, 0xB167 ], [ 0xB170, 0xB2FB ],
    [ 0xBC00, 0xBC6A ], [ 0xBC70, 0xBC7C ], [ 0xBC80, 0xBC88 ], [ 0xBC90, 0xBC99 ], [ 0xD400, 0xD454 ],
    [ 0xD456, 0xD49C ], [ 0xD49E, 0xD49F ], [ 0xD4A2, 0xD4A2 ], [ 0xD4A5, 0xD4A6 ], [ 0xD4A9, 0xD4AC ],
    [ 0xD4AE, 0xD4B9 ], [ 0xD4BB, 0xD4BB ], [ 0xD4BD, 0xD4C3 ], [ 0xD4C5, 0xD505 ], [ 0xD507, 0xD50A ],
    [ 0xD50D, 0xD514 ], [ 0xD516, 0xD51C ], [ 0xD51E, 0xD539 ], [ 0xD53B, 0xD53E ], [ 0xD540, 0xD544 ],
    [ 0xD546, 0xD546 ], [ 0xD54A, 0xD550 ], [ 0xD552, 0xD6A5 ], [ 0xD6A8, 0xD6C0 ], [ 0xD6C2, 0xD6DA ],
    [ 0xD6DC, 0xD6FA ], [ 0xD6FC, 0xD714 ], [ 0xD716, 0xD734 ], [ 0xD736, 0xD74E ], [ 0xD750, 0xD76E ],
    [ 0xD770, 0xD788 ], [ 0xD78A, 0xD7A8 ], [ 0xD7AA, 0xD7C2 ], [ 0xD7C4, 0xD7CB ], [ 0xDF00, 0xDF1E ],
    [ 0xDF25, 0xDF2A ], [ 0xE030, 0xE06D ], [ 0xE100, 0xE12C ], [ 0xE137, 0xE13D ], [ 0xE14E, 0xE14E ],
    [ 0xE290, 0xE2AD ], [ 0xE2C0, 0xE2EB ], [ 0xE4D0, 0xE4EB ], [ 0xE5D0, 0xE5ED ], [ 0xE5F0, 0xE5F0 ],
    [ 0xE7E0, 0xE7E6 ], [ 0xE7E8, 0xE7EB ], [ 0xE7ED, 0xE7EE ], [ 0xE7F0, 0xE7FE ], [ 0xE800, 0xE8C4 ],
    [ 0xE900, 0xE943 ], [ 0xE94B, 0xE94B ], [ 0xEE00, 0xEE03 ], [ 0xEE05, 0xEE1F ], [ 0xEE21, 0xEE22 ],
    [ 0xEE24, 0xEE24 ], [ 0xEE27, 0xEE27 ], [ 0xEE29, 0xEE32 ], [ 0xEE34, 0xEE37 ], [ 0xEE39, 0xEE39 ],
    [ 0xEE3B, 0xEE3B ], [ 0xEE42, 0xEE42 ], [ 0xEE47, 0xEE47 ], [ 0xEE49, 0xEE49 ], [ 0xEE4B, 0xEE4B ],
    [ 0xEE4D, 0xEE4F ], [ 0xEE51, 0xEE52 ], [ 0xEE54, 0xEE54 ], [ 0xEE57, 0xEE57 ], [ 0xEE59, 0xEE59 ],
    [ 0xEE5B, 0xEE5B ], [ 0xEE5D, 0xEE5D ], [ 0xEE5F, 0xEE5F ], [ 0xEE61, 0xEE62 ], [ 0xEE64, 0xEE64 ],
    [ 0xEE67, 0xEE6A ], [ 0xEE6C, 0xEE72 ], [ 0xEE74, 0xEE77 ], [ 0xEE79, 0xEE7C ], [ 0xEE7E, 0xEE7E ],
    [ 0xEE80, 0xEE89 ], [ 0xEE8B, 0xEE9B ], [ 0xEEA1, 0xEEA3 ], [ 0xEEA5, 0xEEA9 ], [ 0xEEAB, 0xEEBB ],
  ];
  #[rustfmt::skip]
  static _IDENTIFIER_START3: [[u32;2];9] = [
    [ 0x20000, 0x2A6DF ], [ 0x2A700, 0x2B739 ], [ 0x2B740, 0x2B81D ], [ 0x2B820, 0x2CEA1 ], [ 0x2CEB0, 0x2EBE0 ],
    [ 0x2EBF0, 0x2EE5D ], [ 0x2F800, 0x2FA1D ], [ 0x30000, 0x3134A ], [ 0x31350, 0x323AF ]
  ];

  #[rustfmt::skip]
  static _IDENTIFIER_NEXT_DELTA1:[[u16;2];229] = [
    [ 0x0030, 0x0039 ], [ 0x0300, 0x036F ], [ 0x0483, 0x0487 ], [ 0x0591, 0x05BD ], [ 0x05BF, 0x05BF ],
    [ 0x05C1, 0x05C2 ], [ 0x05C4, 0x05C5 ], [ 0x05C7, 0x05C7 ], [ 0x0610, 0x061A ], [ 0x064B, 0x0669 ],
    [ 0x0670, 0x0670 ], [ 0x06D6, 0x06DC ], [ 0x06DF, 0x06E4 ], [ 0x06E7, 0x06E8 ], [ 0x06EA, 0x06ED ],
    [ 0x06F0, 0x06F9 ], [ 0x0711, 0x0711 ], [ 0x0730, 0x074A ], [ 0x07A6, 0x07B0 ], [ 0x07C0, 0x07C9 ],
    [ 0x07EB, 0x07F3 ], [ 0x07FD, 0x07FD ], [ 0x0816, 0x0819 ], [ 0x081B, 0x0823 ], [ 0x0825, 0x0827 ],
    [ 0x0829, 0x082D ], [ 0x0859, 0x085B ], [ 0x0897, 0x089F ], [ 0x08CA, 0x08E1 ], [ 0x08E3, 0x0903 ],
    [ 0x093A, 0x093C ], [ 0x093E, 0x094F ], [ 0x0951, 0x0957 ], [ 0x0962, 0x0963 ], [ 0x0966, 0x096F ],
    [ 0x0981, 0x0983 ], [ 0x09BC, 0x09BC ], [ 0x09BE, 0x09C4 ], [ 0x09C7, 0x09C8 ], [ 0x09CB, 0x09CD ],
    [ 0x09D7, 0x09D7 ], [ 0x09E2, 0x09E3 ], [ 0x09E6, 0x09EF ], [ 0x09FE, 0x09FE ], [ 0x0A01, 0x0A03 ],
    [ 0x0A3C, 0x0A3C ], [ 0x0A3E, 0x0A42 ], [ 0x0A47, 0x0A48 ], [ 0x0A4B, 0x0A4D ], [ 0x0A51, 0x0A51 ],
    [ 0x0A66, 0x0A71 ], [ 0x0A75, 0x0A75 ], [ 0x0A81, 0x0A83 ], [ 0x0ABC, 0x0ABC ], [ 0x0ABE, 0x0AC5 ],
    [ 0x0AC7, 0x0AC9 ], [ 0x0ACB, 0x0ACD ], [ 0x0AE2, 0x0AE3 ], [ 0x0AE6, 0x0AEF ], [ 0x0AFA, 0x0AFF ],
    [ 0x0B01, 0x0B03 ], [ 0x0B3C, 0x0B3C ], [ 0x0B3E, 0x0B44 ], [ 0x0B47, 0x0B48 ], [ 0x0B4B, 0x0B4D ],
    [ 0x0B55, 0x0B57 ], [ 0x0B62, 0x0B63 ], [ 0x0B66, 0x0B6F ], [ 0x0B82, 0x0B82 ], [ 0x0BBE, 0x0BC2 ],
    [ 0x0BC6, 0x0BC8 ], [ 0x0BCA, 0x0BCD ], [ 0x0BD7, 0x0BD7 ], [ 0x0BE6, 0x0BEF ], [ 0x0C00, 0x0C04 ],
    [ 0x0C3C, 0x0C3C ], [ 0x0C3E, 0x0C44 ], [ 0x0C46, 0x0C48 ], [ 0x0C4A, 0x0C4D ], [ 0x0C55, 0x0C56 ],
    [ 0x0C62, 0x0C63 ], [ 0x0C66, 0x0C6F ], [ 0x0C81, 0x0C83 ], [ 0x0CBC, 0x0CBC ], [ 0x0CBE, 0x0CC4 ],
    [ 0x0CC6, 0x0CC8 ], [ 0x0CCA, 0x0CCD ], [ 0x0CD5, 0x0CD6 ], [ 0x0CE2, 0x0CE3 ], [ 0x0CE6, 0x0CEF ],
    [ 0x0CF3, 0x0CF3 ], [ 0x0D00, 0x0D03 ], [ 0x0D3B, 0x0D3C ], [ 0x0D3E, 0x0D44 ], [ 0x0D46, 0x0D48 ],
    [ 0x0D4A, 0x0D4D ], [ 0x0D57, 0x0D57 ], [ 0x0D62, 0x0D63 ], [ 0x0D66, 0x0D6F ], [ 0x0D81, 0x0D83 ],
    [ 0x0DCA, 0x0DCA ], [ 0x0DCF, 0x0DD4 ], [ 0x0DD6, 0x0DD6 ], [ 0x0DD8, 0x0DDF ], [ 0x0DE6, 0x0DEF ],
    [ 0x0DF2, 0x0DF3 ], [ 0x0E31, 0x0E31 ], [ 0x0E34, 0x0E3A ], [ 0x0E47, 0x0E4E ], [ 0x0E50, 0x0E59 ],
    [ 0x0EB1, 0x0EB1 ], [ 0x0EB4, 0x0EBC ], [ 0x0EC8, 0x0ECE ], [ 0x0ED0, 0x0ED9 ], [ 0x0F18, 0x0F19 ],
    [ 0x0F20, 0x0F29 ], [ 0x0F35, 0x0F35 ], [ 0x0F37, 0x0F37 ], [ 0x0F39, 0x0F39 ], [ 0x0F3E, 0x0F3F ],
    [ 0x0F71, 0x0F84 ], [ 0x0F86, 0x0F87 ], [ 0x0F8D, 0x0F97 ], [ 0x0F99, 0x0FBC ], [ 0x0FC6, 0x0FC6 ],
    [ 0x102B, 0x103E ], [ 0x1040, 0x1049 ], [ 0x1056, 0x1059 ], [ 0x105E, 0x1060 ], [ 0x1062, 0x1064 ],
    [ 0x1067, 0x106D ], [ 0x1071, 0x1074 ], [ 0x1082, 0x108D ], [ 0x108F, 0x109D ], [ 0x135D, 0x135F ],
    [ 0x1712, 0x1715 ], [ 0x1732, 0x1734 ], [ 0x1752, 0x1753 ], [ 0x1772, 0x1773 ], [ 0x17B4, 0x17D3 ],
    [ 0x17DD, 0x17DD ], [ 0x17E0, 0x17E9 ], [ 0x180B, 0x180D ], [ 0x180F, 0x1819 ], [ 0x1885, 0x1886 ],
    [ 0x18A9, 0x18A9 ], [ 0x1920, 0x192B ], [ 0x1930, 0x193B ], [ 0x1946, 0x194F ], [ 0x19D0, 0x19D9 ],
    [ 0x1A17, 0x1A1B ], [ 0x1A55, 0x1A5E ], [ 0x1A60, 0x1A7C ], [ 0x1A7F, 0x1A89 ], [ 0x1A90, 0x1A99 ],
    [ 0x1AB0, 0x1ABD ], [ 0x1ABF, 0x1ACE ], [ 0x1B00, 0x1B04 ], [ 0x1B34, 0x1B44 ], [ 0x1B50, 0x1B59 ],
    [ 0x1B6B, 0x1B73 ], [ 0x1B80, 0x1B82 ], [ 0x1BA1, 0x1BAD ], [ 0x1BB0, 0x1BB9 ], [ 0x1BE6, 0x1BF3 ],
    [ 0x1C24, 0x1C37 ], [ 0x1C40, 0x1C49 ], [ 0x1C50, 0x1C59 ], [ 0x1CD0, 0x1CD2 ], [ 0x1CD4, 0x1CE8 ],
    [ 0x1CED, 0x1CED ], [ 0x1CF4, 0x1CF4 ], [ 0x1CF7, 0x1CF9 ], [ 0x1DC0, 0x1DFF ], [ 0x200C, 0x200D ],
    [ 0x203F, 0x2040 ], [ 0x2054, 0x2054 ], [ 0x20D0, 0x20DC ], [ 0x20E1, 0x20E1 ], [ 0x20E5, 0x20F0 ],
    [ 0x2CEF, 0x2CF1 ], [ 0x2D7F, 0x2D7F ], [ 0x2DE0, 0x2DFF ], [ 0x302A, 0x302F ], [ 0x3099, 0x309A ],
    [ 0xA620, 0xA629 ], [ 0xA66F, 0xA66F ], [ 0xA674, 0xA67D ], [ 0xA69E, 0xA69F ], [ 0xA6F0, 0xA6F1 ],
    [ 0xA802, 0xA802 ], [ 0xA806, 0xA806 ], [ 0xA80B, 0xA80B ], [ 0xA823, 0xA827 ], [ 0xA82C, 0xA82C ],
    [ 0xA880, 0xA881 ], [ 0xA8B4, 0xA8C5 ], [ 0xA8D0, 0xA8D9 ], [ 0xA8E0, 0xA8F1 ], [ 0xA8FF, 0xA909 ],
    [ 0xA926, 0xA92D ], [ 0xA947, 0xA953 ], [ 0xA980, 0xA983 ], [ 0xA9B3, 0xA9C0 ], [ 0xA9D0, 0xA9D9 ],
    [ 0xA9E5, 0xA9E5 ], [ 0xA9F0, 0xA9F9 ], [ 0xAA29, 0xAA36 ], [ 0xAA43, 0xAA43 ], [ 0xAA4C, 0xAA4D ],
    [ 0xAA50, 0xAA59 ], [ 0xAA7B, 0xAA7D ], [ 0xAAB0, 0xAAB0 ], [ 0xAAB2, 0xAAB4 ], [ 0xAAB7, 0xAAB8 ],
    [ 0xAABE, 0xAABF ], [ 0xAAC1, 0xAAC1 ], [ 0xAAEB, 0xAAEF ], [ 0xAAF5, 0xAAF6 ], [ 0xABE3, 0xABEA ],
    [ 0xABEC, 0xABED ], [ 0xABF0, 0xABF9 ], [ 0xFB1E, 0xFB1E ], [ 0xFE00, 0xFE0F ], [ 0xFE20, 0xFE2F ],
    [ 0xFE33, 0xFE34 ], [ 0xFE4D, 0xFE4F ], [ 0xFF10, 0xFF19 ], [ 0xFF3F, 0xFF3F ]
  ];
  #[rustfmt::skip]
  static _IDENTIFIER_NEXT_DELTA2:[[u16;2];158] = [
    [ 0x01FD, 0x01FD ], [ 0x02E0, 0x02E0 ], [ 0x0376, 0x037A ], [ 0x04A0, 0x04A9 ], [ 0x0A01, 0x0A03 ],
    [ 0x0A05, 0x0A06 ], [ 0x0A0C, 0x0A0F ], [ 0x0A38, 0x0A3A ], [ 0x0A3F, 0x0A3F ], [ 0x0AE5, 0x0AE6 ],
    [ 0x0D24, 0x0D27 ], [ 0x0D30, 0x0D39 ], [ 0x0D40, 0x0D49 ], [ 0x0D69, 0x0D6D ], [ 0x0EAB, 0x0EAC ],
    [ 0x0EFC, 0x0EFF ], [ 0x0F46, 0x0F50 ], [ 0x0F82, 0x0F85 ], [ 0x1000, 0x1002 ], [ 0x1038, 0x1046 ],
    [ 0x1066, 0x1070 ], [ 0x1073, 0x1074 ], [ 0x107F, 0x1082 ], [ 0x10B0, 0x10BA ], [ 0x10C2, 0x10C2 ],
    [ 0x10F0, 0x10F9 ], [ 0x1100, 0x1102 ], [ 0x1127, 0x1134 ], [ 0x1136, 0x113F ], [ 0x1145, 0x1146 ],
    [ 0x1173, 0x1173 ], [ 0x1180, 0x1182 ], [ 0x11B3, 0x11C0 ], [ 0x11C9, 0x11CC ], [ 0x11CE, 0x11D9 ],
    [ 0x122C, 0x1237 ], [ 0x123E, 0x123E ], [ 0x1241, 0x1241 ], [ 0x12DF, 0x12EA ], [ 0x12F0, 0x12F9 ],
    [ 0x1300, 0x1303 ], [ 0x133B, 0x133C ], [ 0x133E, 0x1344 ], [ 0x1347, 0x1348 ], [ 0x134B, 0x134D ],
    [ 0x1357, 0x1357 ], [ 0x1362, 0x1363 ], [ 0x1366, 0x136C ], [ 0x1370, 0x1374 ], [ 0x13B8, 0x13C0 ],
    [ 0x13C2, 0x13C2 ], [ 0x13C5, 0x13C5 ], [ 0x13C7, 0x13CA ], [ 0x13CC, 0x13D0 ], [ 0x13D2, 0x13D2 ],
    [ 0x13E1, 0x13E2 ], [ 0x1435, 0x1446 ], [ 0x1450, 0x1459 ], [ 0x145E, 0x145E ], [ 0x14B0, 0x14C3 ],
    [ 0x14D0, 0x14D9 ], [ 0x15AF, 0x15B5 ], [ 0x15B8, 0x15C0 ], [ 0x15DC, 0x15DD ], [ 0x1630, 0x1640 ],
    [ 0x1650, 0x1659 ], [ 0x16AB, 0x16B7 ], [ 0x16C0, 0x16C9 ], [ 0x16D0, 0x16E3 ], [ 0x171D, 0x172B ],
    [ 0x1730, 0x1739 ], [ 0x182C, 0x183A ], [ 0x18E0, 0x18E9 ], [ 0x1930, 0x1935 ], [ 0x1937, 0x1938 ],
    [ 0x193B, 0x193E ], [ 0x1940, 0x1940 ], [ 0x1942, 0x1943 ], [ 0x1950, 0x1959 ], [ 0x19D1, 0x19D7 ],
    [ 0x19DA, 0x19E0 ], [ 0x19E4, 0x19E4 ], [ 0x1A01, 0x1A0A ], [ 0x1A33, 0x1A39 ], [ 0x1A3B, 0x1A3E ],
    [ 0x1A47, 0x1A47 ], [ 0x1A51, 0x1A5B ], [ 0x1A8A, 0x1A99 ], [ 0x1BF0, 0x1BF9 ], [ 0x1C2F, 0x1C36 ],
    [ 0x1C38, 0x1C3F ], [ 0x1C50, 0x1C59 ], [ 0x1C92, 0x1CA7 ], [ 0x1CA9, 0x1CB6 ], [ 0x1D31, 0x1D36 ],
    [ 0x1D3A, 0x1D3A ], [ 0x1D3C, 0x1D3D ], [ 0x1D3F, 0x1D45 ], [ 0x1D47, 0x1D47 ], [ 0x1D50, 0x1D59 ],
    [ 0x1D8A, 0x1D8E ], [ 0x1D90, 0x1D91 ], [ 0x1D93, 0x1D97 ], [ 0x1DA0, 0x1DA9 ], [ 0x1EF3, 0x1EF6 ],
    [ 0x1F00, 0x1F01 ], [ 0x1F03, 0x1F03 ], [ 0x1F34, 0x1F3A ], [ 0x1F3E, 0x1F42 ], [ 0x1F50, 0x1F5A ],
    [ 0x3440, 0x3440 ], [ 0x3447, 0x3455 ], [ 0x611E, 0x6139 ], [ 0x6A60, 0x6A69 ], [ 0x6AC0, 0x6AC9 ],
    [ 0x6AF0, 0x6AF4 ], [ 0x6B30, 0x6B36 ], [ 0x6B50, 0x6B59 ], [ 0x6D70, 0x6D79 ], [ 0x6F4F, 0x6F4F ],
    [ 0x6F51, 0x6F87 ], [ 0x6F8F, 0x6F92 ], [ 0x6FE4, 0x6FE4 ], [ 0x6FF0, 0x6FF1 ], [ 0xBC9D, 0xBC9E ],
    [ 0xCCF0, 0xCCF9 ], [ 0xCF00, 0xCF2D ], [ 0xCF30, 0xCF46 ], [ 0xD165, 0xD169 ], [ 0xD16D, 0xD172 ],
    [ 0xD17B, 0xD182 ], [ 0xD185, 0xD18B ], [ 0xD1AA, 0xD1AD ], [ 0xD242, 0xD244 ], [ 0xD7CE, 0xD7FF ],
    [ 0xDA00, 0xDA36 ], [ 0xDA3B, 0xDA6C ], [ 0xDA75, 0xDA75 ], [ 0xDA84, 0xDA84 ], [ 0xDA9B, 0xDA9F ],
    [ 0xDAA1, 0xDAAF ], [ 0xE000, 0xE006 ], [ 0xE008, 0xE018 ], [ 0xE01B, 0xE021 ], [ 0xE023, 0xE024 ],
    [ 0xE026, 0xE02A ], [ 0xE08F, 0xE08F ], [ 0xE130, 0xE136 ], [ 0xE140, 0xE149 ], [ 0xE2AE, 0xE2AE ],
    [ 0xE2EC, 0xE2F9 ], [ 0xE4EC, 0xE4F9 ], [ 0xE5EE, 0xE5EF ], [ 0xE5F1, 0xE5FA ], [ 0xE8D0, 0xE8D6 ],
    [ 0xE944, 0xE94A ], [ 0xE950, 0xE959 ], [ 0xFBF0, 0xFBF9 ],
  ];
  #[rustfmt::skip]
  static _IDENTIFIER_NEXT_DELTA3:[[u32;2];1] = [ [ 0xE0100, 0xE01EF ] ];

  #[rustfmt::skip]
  static _GRAPH1:[[u16;2];343] = [
    [ 0x0020, 0x007E ], [ 0x00A0, 0x00AC ], [ 0x00AE, 0x0377 ], [ 0x037A, 0x037F ], [ 0x0384, 0x038A ],
    [ 0x038C, 0x038C ], [ 0x038E, 0x03A1 ], [ 0x03A3, 0x052F ], [ 0x0531, 0x0556 ], [ 0x0559, 0x058A ],
    [ 0x058D, 0x058F ], [ 0x0591, 0x05C7 ], [ 0x05D0, 0x05EA ], [ 0x05EF, 0x05F4 ], [ 0x0606, 0x061B ],
    [ 0x061D, 0x06DC ], [ 0x06DE, 0x070D ], [ 0x0710, 0x074A ], [ 0x074D, 0x07B1 ], [ 0x07C0, 0x07FA ],
    [ 0x07FD, 0x082D ], [ 0x0830, 0x083E ], [ 0x0840, 0x085B ], [ 0x085E, 0x085E ], [ 0x0860, 0x086A ],
    [ 0x0870, 0x088E ], [ 0x0897, 0x08E1 ], [ 0x08E3, 0x0983 ], [ 0x0985, 0x098C ], [ 0x098F, 0x0990 ],
    [ 0x0993, 0x09A8 ], [ 0x09AA, 0x09B0 ], [ 0x09B2, 0x09B2 ], [ 0x09B6, 0x09B9 ], [ 0x09BC, 0x09C4 ],
    [ 0x09C7, 0x09C8 ], [ 0x09CB, 0x09CE ], [ 0x09D7, 0x09D7 ], [ 0x09DC, 0x09DD ], [ 0x09DF, 0x09E3 ],
    [ 0x09E6, 0x09FE ], [ 0x0A01, 0x0A03 ], [ 0x0A05, 0x0A0A ], [ 0x0A0F, 0x0A10 ], [ 0x0A13, 0x0A28 ],
    [ 0x0A2A, 0x0A30 ], [ 0x0A32, 0x0A33 ], [ 0x0A35, 0x0A36 ], [ 0x0A38, 0x0A39 ], [ 0x0A3C, 0x0A3C ],
    [ 0x0A3E, 0x0A42 ], [ 0x0A47, 0x0A48 ], [ 0x0A4B, 0x0A4D ], [ 0x0A51, 0x0A51 ], [ 0x0A59, 0x0A5C ],
    [ 0x0A5E, 0x0A5E ], [ 0x0A66, 0x0A76 ], [ 0x0A81, 0x0A83 ], [ 0x0A85, 0x0A8D ], [ 0x0A8F, 0x0A91 ],
    [ 0x0A93, 0x0AA8 ], [ 0x0AAA, 0x0AB0 ], [ 0x0AB2, 0x0AB3 ], [ 0x0AB5, 0x0AB9 ], [ 0x0ABC, 0x0AC5 ],
    [ 0x0AC7, 0x0AC9 ], [ 0x0ACB, 0x0ACD ], [ 0x0AD0, 0x0AD0 ], [ 0x0AE0, 0x0AE3 ], [ 0x0AE6, 0x0AF1 ],
    [ 0x0AF9, 0x0AFF ], [ 0x0B01, 0x0B03 ], [ 0x0B05, 0x0B0C ], [ 0x0B0F, 0x0B10 ], [ 0x0B13, 0x0B28 ],
    [ 0x0B2A, 0x0B30 ], [ 0x0B32, 0x0B33 ], [ 0x0B35, 0x0B39 ], [ 0x0B3C, 0x0B44 ], [ 0x0B47, 0x0B48 ],
    [ 0x0B4B, 0x0B4D ], [ 0x0B55, 0x0B57 ], [ 0x0B5C, 0x0B5D ], [ 0x0B5F, 0x0B63 ], [ 0x0B66, 0x0B77 ],
    [ 0x0B82, 0x0B83 ], [ 0x0B85, 0x0B8A ], [ 0x0B8E, 0x0B90 ], [ 0x0B92, 0x0B95 ], [ 0x0B99, 0x0B9A ],
    [ 0x0B9C, 0x0B9C ], [ 0x0B9E, 0x0B9F ], [ 0x0BA3, 0x0BA4 ], [ 0x0BA8, 0x0BAA ], [ 0x0BAE, 0x0BB9 ],
    [ 0x0BBE, 0x0BC2 ], [ 0x0BC6, 0x0BC8 ], [ 0x0BCA, 0x0BCD ], [ 0x0BD0, 0x0BD0 ], [ 0x0BD7, 0x0BD7 ],
    [ 0x0BE6, 0x0BFA ], [ 0x0C00, 0x0C0C ], [ 0x0C0E, 0x0C10 ], [ 0x0C12, 0x0C28 ], [ 0x0C2A, 0x0C39 ],
    [ 0x0C3C, 0x0C44 ], [ 0x0C46, 0x0C48 ], [ 0x0C4A, 0x0C4D ], [ 0x0C55, 0x0C56 ], [ 0x0C58, 0x0C5A ],
    [ 0x0C5D, 0x0C5D ], [ 0x0C60, 0x0C63 ], [ 0x0C66, 0x0C6F ], [ 0x0C77, 0x0C8C ], [ 0x0C8E, 0x0C90 ],
    [ 0x0C92, 0x0CA8 ], [ 0x0CAA, 0x0CB3 ], [ 0x0CB5, 0x0CB9 ], [ 0x0CBC, 0x0CC4 ], [ 0x0CC6, 0x0CC8 ],
    [ 0x0CCA, 0x0CCD ], [ 0x0CD5, 0x0CD6 ], [ 0x0CDD, 0x0CDE ], [ 0x0CE0, 0x0CE3 ], [ 0x0CE6, 0x0CEF ],
    [ 0x0CF1, 0x0CF3 ], [ 0x0D00, 0x0D0C ], [ 0x0D0E, 0x0D10 ], [ 0x0D12, 0x0D44 ], [ 0x0D46, 0x0D48 ],
    [ 0x0D4A, 0x0D4F ], [ 0x0D54, 0x0D63 ], [ 0x0D66, 0x0D7F ], [ 0x0D81, 0x0D83 ], [ 0x0D85, 0x0D96 ],
    [ 0x0D9A, 0x0DB1 ], [ 0x0DB3, 0x0DBB ], [ 0x0DBD, 0x0DBD ], [ 0x0DC0, 0x0DC6 ], [ 0x0DCA, 0x0DCA ],
    [ 0x0DCF, 0x0DD4 ], [ 0x0DD6, 0x0DD6 ], [ 0x0DD8, 0x0DDF ], [ 0x0DE6, 0x0DEF ], [ 0x0DF2, 0x0DF4 ],
    [ 0x0E01, 0x0E3A ], [ 0x0E3F, 0x0E5B ], [ 0x0E81, 0x0E82 ], [ 0x0E84, 0x0E84 ], [ 0x0E86, 0x0E8A ],
    [ 0x0E8C, 0x0EA3 ], [ 0x0EA5, 0x0EA5 ], [ 0x0EA7, 0x0EBD ], [ 0x0EC0, 0x0EC4 ], [ 0x0EC6, 0x0EC6 ],
    [ 0x0EC8, 0x0ECE ], [ 0x0ED0, 0x0ED9 ], [ 0x0EDC, 0x0EDF ], [ 0x0F00, 0x0F47 ], [ 0x0F49, 0x0F6C ],
    [ 0x0F71, 0x0F97 ], [ 0x0F99, 0x0FBC ], [ 0x0FBE, 0x0FCC ], [ 0x0FCE, 0x0FDA ], [ 0x1000, 0x10C5 ],
    [ 0x10C7, 0x10C7 ], [ 0x10CD, 0x10CD ], [ 0x10D0, 0x1248 ], [ 0x124A, 0x124D ], [ 0x1250, 0x1256 ],
    [ 0x1258, 0x1258 ], [ 0x125A, 0x125D ], [ 0x1260, 0x1288 ], [ 0x128A, 0x128D ], [ 0x1290, 0x12B0 ],
    [ 0x12B2, 0x12B5 ], [ 0x12B8, 0x12BE ], [ 0x12C0, 0x12C0 ], [ 0x12C2, 0x12C5 ], [ 0x12C8, 0x12D6 ],
    [ 0x12D8, 0x1310 ], [ 0x1312, 0x1315 ], [ 0x1318, 0x135A ], [ 0x135D, 0x137C ], [ 0x1380, 0x1399 ],
    [ 0x13A0, 0x13F5 ], [ 0x13F8, 0x13FD ], [ 0x1400, 0x169C ], [ 0x16A0, 0x16F8 ], [ 0x1700, 0x1715 ],
    [ 0x171F, 0x1736 ], [ 0x1740, 0x1753 ], [ 0x1760, 0x176C ], [ 0x176E, 0x1770 ], [ 0x1772, 0x1773 ],
    [ 0x1780, 0x17DD ], [ 0x17E0, 0x17E9 ], [ 0x17F0, 0x17F9 ], [ 0x1800, 0x180D ], [ 0x180F, 0x1819 ],
    [ 0x1820, 0x1878 ], [ 0x1880, 0x18AA ], [ 0x18B0, 0x18F5 ], [ 0x1900, 0x191E ], [ 0x1920, 0x192B ],
    [ 0x1930, 0x193B ], [ 0x1940, 0x1940 ], [ 0x1944, 0x196D ], [ 0x1970, 0x1974 ], [ 0x1980, 0x19AB ],
    [ 0x19B0, 0x19C9 ], [ 0x19D0, 0x19DA ], [ 0x19DE, 0x1A1B ], [ 0x1A1E, 0x1A5E ], [ 0x1A60, 0x1A7C ],
    [ 0x1A7F, 0x1A89 ], [ 0x1A90, 0x1A99 ], [ 0x1AA0, 0x1AAD ], [ 0x1AB0, 0x1ACE ], [ 0x1B00, 0x1B4C ],
    [ 0x1B4E, 0x1BF3 ], [ 0x1BFC, 0x1C37 ], [ 0x1C3B, 0x1C49 ], [ 0x1C4D, 0x1C8A ], [ 0x1C90, 0x1CBA ],
    [ 0x1CBD, 0x1CC7 ], [ 0x1CD0, 0x1CFA ], [ 0x1D00, 0x1F15 ], [ 0x1F18, 0x1F1D ], [ 0x1F20, 0x1F45 ],
    [ 0x1F48, 0x1F4D ], [ 0x1F50, 0x1F57 ], [ 0x1F59, 0x1F59 ], [ 0x1F5B, 0x1F5B ], [ 0x1F5D, 0x1F5D ],
    [ 0x1F5F, 0x1F7D ], [ 0x1F80, 0x1FB4 ], [ 0x1FB6, 0x1FC4 ], [ 0x1FC6, 0x1FD3 ], [ 0x1FD6, 0x1FDB ],
    [ 0x1FDD, 0x1FEF ], [ 0x1FF2, 0x1FF4 ], [ 0x1FF6, 0x1FFE ], [ 0x2000, 0x200A ], [ 0x2010, 0x2029 ],
    [ 0x202F, 0x205F ], [ 0x2070, 0x2071 ], [ 0x2074, 0x208E ], [ 0x2090, 0x209C ], [ 0x20A0, 0x20C0 ],
    [ 0x20D0, 0x20F0 ], [ 0x2100, 0x218B ], [ 0x2190, 0x2429 ], [ 0x2440, 0x244A ], [ 0x2460, 0x2B73 ],
    [ 0x2B76, 0x2B95 ], [ 0x2B97, 0x2CF3 ], [ 0x2CF9, 0x2D25 ], [ 0x2D27, 0x2D27 ], [ 0x2D2D, 0x2D2D ],
    [ 0x2D30, 0x2D67 ], [ 0x2D6F, 0x2D70 ], [ 0x2D7F, 0x2D96 ], [ 0x2DA0, 0x2DA6 ], [ 0x2DA8, 0x2DAE ],
    [ 0x2DB0, 0x2DB6 ], [ 0x2DB8, 0x2DBE ], [ 0x2DC0, 0x2DC6 ], [ 0x2DC8, 0x2DCE ], [ 0x2DD0, 0x2DD6 ],
    [ 0x2DD8, 0x2DDE ], [ 0x2DE0, 0x2E5D ], [ 0x2E80, 0x2E99 ], [ 0x2E9B, 0x2EF3 ], [ 0x2F00, 0x2FD5 ],
    [ 0x2FF0, 0x303F ], [ 0x3041, 0x3096 ], [ 0x3099, 0x30FF ], [ 0x3105, 0x312F ], [ 0x3131, 0x318E ],
    [ 0x3190, 0x31E5 ], [ 0x31EF, 0x321E ], [ 0x3220, 0xA48C ], [ 0xA490, 0xA4C6 ], [ 0xA4D0, 0xA62B ],
    [ 0xA640, 0xA6F7 ], [ 0xA700, 0xA7CD ], [ 0xA7D0, 0xA7D1 ], [ 0xA7D3, 0xA7D3 ], [ 0xA7D5, 0xA7DC ],
    [ 0xA7F2, 0xA82C ], [ 0xA830, 0xA839 ], [ 0xA840, 0xA877 ], [ 0xA880, 0xA8C5 ], [ 0xA8CE, 0xA8D9 ],
    [ 0xA8E0, 0xA953 ], [ 0xA95F, 0xA97C ], [ 0xA980, 0xA9CD ], [ 0xA9CF, 0xA9D9 ], [ 0xA9DE, 0xA9FE ],
    [ 0xAA00, 0xAA36 ], [ 0xAA40, 0xAA4D ], [ 0xAA50, 0xAA59 ], [ 0xAA5C, 0xAAC2 ], [ 0xAADB, 0xAAF6 ],
    [ 0xAB01, 0xAB06 ], [ 0xAB09, 0xAB0E ], [ 0xAB11, 0xAB16 ], [ 0xAB20, 0xAB26 ], [ 0xAB28, 0xAB2E ],
    [ 0xAB30, 0xAB6B ], [ 0xAB70, 0xABED ], [ 0xABF0, 0xABF9 ], [ 0xAC00, 0xD7A3 ], [ 0xD7B0, 0xD7C6 ],
    [ 0xD7CB, 0xD7FB ], [ 0xF900, 0xFA6D ], [ 0xFA70, 0xFAD9 ], [ 0xFB00, 0xFB06 ], [ 0xFB13, 0xFB17 ],
    [ 0xFB1D, 0xFB36 ], [ 0xFB38, 0xFB3C ], [ 0xFB3E, 0xFB3E ], [ 0xFB40, 0xFB41 ], [ 0xFB43, 0xFB44 ],
    [ 0xFB46, 0xFBC2 ], [ 0xFBD3, 0xFD8F ], [ 0xFD92, 0xFDC7 ], [ 0xFDCF, 0xFDCF ], [ 0xFDF0, 0xFE19 ],
    [ 0xFE20, 0xFE52 ], [ 0xFE54, 0xFE66 ], [ 0xFE68, 0xFE6B ], [ 0xFE70, 0xFE74 ], [ 0xFE76, 0xFEFC ],
    [ 0xFF01, 0xFFBE ], [ 0xFFC2, 0xFFC7 ], [ 0xFFCA, 0xFFCF ], [ 0xFFD2, 0xFFD7 ], [ 0xFFDA, 0xFFDC ],
    [ 0xFFE0, 0xFFE6 ], [ 0xFFE8, 0xFFEE ], [ 0xFFFC, 0xFFFD ]
  ];
  #[rustfmt::skip]
  static _GRAPH2:[[u16;2];382] = [
    [ 0x0000, 0x000B ], [ 0x000D, 0x0026 ], [ 0x0028, 0x003A ], [ 0x003C, 0x003D ], [ 0x003F, 0x004D ],
    [ 0x0050, 0x005D ], [ 0x0080, 0x00FA ], [ 0x0100, 0x0102 ], [ 0x0107, 0x0133 ], [ 0x0137, 0x018E ],
    [ 0x0190, 0x019C ], [ 0x01A0, 0x01A0 ], [ 0x01D0, 0x01FD ], [ 0x0280, 0x029C ], [ 0x02A0, 0x02D0 ],
    [ 0x02E0, 0x02FB ], [ 0x0300, 0x0323 ], [ 0x032D, 0x034A ], [ 0x0350, 0x037A ], [ 0x0380, 0x039D ],
    [ 0x039F, 0x03C3 ], [ 0x03C8, 0x03D5 ], [ 0x0400, 0x049D ], [ 0x04A0, 0x04A9 ], [ 0x04B0, 0x04D3 ],
    [ 0x04D8, 0x04FB ], [ 0x0500, 0x0527 ], [ 0x0530, 0x0563 ], [ 0x056F, 0x057A ], [ 0x057C, 0x058A ],
    [ 0x058C, 0x0592 ], [ 0x0594, 0x0595 ], [ 0x0597, 0x05A1 ], [ 0x05A3, 0x05B1 ], [ 0x05B3, 0x05B9 ],
    [ 0x05BB, 0x05BC ], [ 0x05C0, 0x05F3 ], [ 0x0600, 0x0736 ], [ 0x0740, 0x0755 ], [ 0x0760, 0x0767 ],
    [ 0x0780, 0x0785 ], [ 0x0787, 0x07B0 ], [ 0x07B2, 0x07BA ], [ 0x0800, 0x0805 ], [ 0x0808, 0x0808 ],
    [ 0x080A, 0x0835 ], [ 0x0837, 0x0838 ], [ 0x083C, 0x083C ], [ 0x083F, 0x0855 ], [ 0x0857, 0x089E ],
    [ 0x08A7, 0x08AF ], [ 0x08E0, 0x08F2 ], [ 0x08F4, 0x08F5 ], [ 0x08FB, 0x091B ], [ 0x091F, 0x0939 ],
    [ 0x093F, 0x093F ], [ 0x0980, 0x09B7 ], [ 0x09BC, 0x09CF ], [ 0x09D2, 0x0A03 ], [ 0x0A05, 0x0A06 ],
    [ 0x0A0C, 0x0A13 ], [ 0x0A15, 0x0A17 ], [ 0x0A19, 0x0A35 ], [ 0x0A38, 0x0A3A ], [ 0x0A3F, 0x0A48 ],
    [ 0x0A50, 0x0A58 ], [ 0x0A60, 0x0A9F ], [ 0x0AC0, 0x0AE6 ], [ 0x0AEB, 0x0AF6 ], [ 0x0B00, 0x0B35 ],
    [ 0x0B39, 0x0B55 ], [ 0x0B58, 0x0B72 ], [ 0x0B78, 0x0B91 ], [ 0x0B99, 0x0B9C ], [ 0x0BA9, 0x0BAF ],
    [ 0x0C00, 0x0C48 ], [ 0x0C80, 0x0CB2 ], [ 0x0CC0, 0x0CF2 ], [ 0x0CFA, 0x0D27 ], [ 0x0D30, 0x0D39 ],
    [ 0x0D40, 0x0D65 ], [ 0x0D69, 0x0D85 ], [ 0x0D8E, 0x0D8F ], [ 0x0E60, 0x0E7E ], [ 0x0E80, 0x0EA9 ],
    [ 0x0EAB, 0x0EAD ], [ 0x0EB0, 0x0EB1 ], [ 0x0EC2, 0x0EC4 ], [ 0x0EFC, 0x0F27 ], [ 0x0F30, 0x0F59 ],
    [ 0x0F70, 0x0F89 ], [ 0x0FB0, 0x0FCB ], [ 0x0FE0, 0x0FF6 ], [ 0x1000, 0x104D ], [ 0x1052, 0x1075 ],
    [ 0x107F, 0x10BC ], [ 0x10BE, 0x10C2 ], [ 0x10D0, 0x10E8 ], [ 0x10F0, 0x10F9 ], [ 0x1100, 0x1134 ],
    [ 0x1136, 0x1147 ], [ 0x1150, 0x1176 ], [ 0x1180, 0x11DF ], [ 0x11E1, 0x11F4 ], [ 0x1200, 0x1211 ],
    [ 0x1213, 0x1241 ], [ 0x1280, 0x1286 ], [ 0x1288, 0x1288 ], [ 0x128A, 0x128D ], [ 0x128F, 0x129D ],
    [ 0x129F, 0x12A9 ], [ 0x12B0, 0x12EA ], [ 0x12F0, 0x12F9 ], [ 0x1300, 0x1303 ], [ 0x1305, 0x130C ],
    [ 0x130F, 0x1310 ], [ 0x1313, 0x1328 ], [ 0x132A, 0x1330 ], [ 0x1332, 0x1333 ], [ 0x1335, 0x1339 ],
    [ 0x133B, 0x1344 ], [ 0x1347, 0x1348 ], [ 0x134B, 0x134D ], [ 0x1350, 0x1350 ], [ 0x1357, 0x1357 ],
    [ 0x135D, 0x1363 ], [ 0x1366, 0x136C ], [ 0x1370, 0x1374 ], [ 0x1380, 0x1389 ], [ 0x138B, 0x138B ],
    [ 0x138E, 0x138E ], [ 0x1390, 0x13B5 ], [ 0x13B7, 0x13C0 ], [ 0x13C2, 0x13C2 ], [ 0x13C5, 0x13C5 ],
    [ 0x13C7, 0x13CA ], [ 0x13CC, 0x13D5 ], [ 0x13D7, 0x13D8 ], [ 0x13E1, 0x13E2 ], [ 0x1400, 0x145B ],
    [ 0x145D, 0x1461 ], [ 0x1480, 0x14C7 ], [ 0x14D0, 0x14D9 ], [ 0x1580, 0x15B5 ], [ 0x15B8, 0x15DD ],
    [ 0x1600, 0x1644 ], [ 0x1650, 0x1659 ], [ 0x1660, 0x166C ], [ 0x1680, 0x16B9 ], [ 0x16C0, 0x16C9 ],
    [ 0x16D0, 0x16E3 ], [ 0x1700, 0x171A ], [ 0x171D, 0x172B ], [ 0x1730, 0x1746 ], [ 0x1800, 0x183B ],
    [ 0x18A0, 0x18F2 ], [ 0x18FF, 0x1906 ], [ 0x1909, 0x1909 ], [ 0x190C, 0x1913 ], [ 0x1915, 0x1916 ],
    [ 0x1918, 0x1935 ], [ 0x1937, 0x1938 ], [ 0x193B, 0x1946 ], [ 0x1950, 0x1959 ], [ 0x19A0, 0x19A7 ],
    [ 0x19AA, 0x19D7 ], [ 0x19DA, 0x19E4 ], [ 0x1A00, 0x1A47 ], [ 0x1A50, 0x1AA2 ], [ 0x1AB0, 0x1AF8 ],
    [ 0x1B00, 0x1B09 ], [ 0x1BC0, 0x1BE1 ], [ 0x1BF0, 0x1BF9 ], [ 0x1C00, 0x1C08 ], [ 0x1C0A, 0x1C36 ],
    [ 0x1C38, 0x1C45 ], [ 0x1C50, 0x1C6C ], [ 0x1C70, 0x1C8F ], [ 0x1C92, 0x1CA7 ], [ 0x1CA9, 0x1CB6 ],
    [ 0x1D00, 0x1D06 ], [ 0x1D08, 0x1D09 ], [ 0x1D0B, 0x1D36 ], [ 0x1D3A, 0x1D3A ], [ 0x1D3C, 0x1D3D ],
    [ 0x1D3F, 0x1D47 ], [ 0x1D50, 0x1D59 ], [ 0x1D60, 0x1D65 ], [ 0x1D67, 0x1D68 ], [ 0x1D6A, 0x1D8E ],
    [ 0x1D90, 0x1D91 ], [ 0x1D93, 0x1D98 ], [ 0x1DA0, 0x1DA9 ], [ 0x1EE0, 0x1EF8 ], [ 0x1F00, 0x1F10 ],
    [ 0x1F12, 0x1F3A ], [ 0x1F3E, 0x1F5A ], [ 0x1FB0, 0x1FB0 ], [ 0x1FC0, 0x1FF1 ], [ 0x1FFF, 0x2399 ],
    [ 0x2400, 0x246E ], [ 0x2470, 0x2474 ], [ 0x2480, 0x2543 ], [ 0x2F90, 0x2FF2 ], [ 0x3000, 0x342F ],
    [ 0x3440, 0x3455 ], [ 0x3460, 0x43FA ], [ 0x4400, 0x4646 ], [ 0x6100, 0x6139 ], [ 0x6800, 0x6A38 ],
    [ 0x6A40, 0x6A5E ], [ 0x6A60, 0x6A69 ], [ 0x6A6E, 0x6ABE ], [ 0x6AC0, 0x6AC9 ], [ 0x6AD0, 0x6AED ],
    [ 0x6AF0, 0x6AF5 ], [ 0x6B00, 0x6B45 ], [ 0x6B50, 0x6B59 ], [ 0x6B5B, 0x6B61 ], [ 0x6B63, 0x6B77 ],
    [ 0x6B7D, 0x6B8F ], [ 0x6D40, 0x6D79 ], [ 0x6E40, 0x6E9A ], [ 0x6F00, 0x6F4A ], [ 0x6F4F, 0x6F87 ],
    [ 0x6F8F, 0x6F9F ], [ 0x6FE0, 0x6FE4 ], [ 0x6FF0, 0x6FF1 ], [ 0x7000, 0x87F7 ], [ 0x8800, 0x8CD5 ],
    [ 0x8CFF, 0x8D08 ], [ 0xAFF0, 0xAFF3 ], [ 0xAFF5, 0xAFFB ], [ 0xAFFD, 0xAFFE ], [ 0xB000, 0xB122 ],
    [ 0xB132, 0xB132 ], [ 0xB150, 0xB152 ], [ 0xB155, 0xB155 ], [ 0xB164, 0xB167 ], [ 0xB170, 0xB2FB ],
    [ 0xBC00, 0xBC6A ], [ 0xBC70, 0xBC7C ], [ 0xBC80, 0xBC88 ], [ 0xBC90, 0xBC99 ], [ 0xBC9C, 0xBC9F ],
    [ 0xCC00, 0xCCF9 ], [ 0xCD00, 0xCEB3 ], [ 0xCF00, 0xCF2D ], [ 0xCF30, 0xCF46 ], [ 0xCF50, 0xCFC3 ],
    [ 0xD000, 0xD0F5 ], [ 0xD100, 0xD126 ], [ 0xD129, 0xD172 ], [ 0xD17B, 0xD1EA ], [ 0xD200, 0xD245 ],
    [ 0xD2C0, 0xD2D3 ], [ 0xD2E0, 0xD2F3 ], [ 0xD300, 0xD356 ], [ 0xD360, 0xD378 ], [ 0xD400, 0xD454 ],
    [ 0xD456, 0xD49C ], [ 0xD49E, 0xD49F ], [ 0xD4A2, 0xD4A2 ], [ 0xD4A5, 0xD4A6 ], [ 0xD4A9, 0xD4AC ],
    [ 0xD4AE, 0xD4B9 ], [ 0xD4BB, 0xD4BB ], [ 0xD4BD, 0xD4C3 ], [ 0xD4C5, 0xD505 ], [ 0xD507, 0xD50A ],
    [ 0xD50D, 0xD514 ], [ 0xD516, 0xD51C ], [ 0xD51E, 0xD539 ], [ 0xD53B, 0xD53E ], [ 0xD540, 0xD544 ],
    [ 0xD546, 0xD546 ], [ 0xD54A, 0xD550 ], [ 0xD552, 0xD6A5 ], [ 0xD6A8, 0xD7CB ], [ 0xD7CE, 0xDA8B ],
    [ 0xDA9B, 0xDA9F ], [ 0xDAA1, 0xDAAF ], [ 0xDF00, 0xDF1E ], [ 0xDF25, 0xDF2A ], [ 0xE000, 0xE006 ],
    [ 0xE008, 0xE018 ], [ 0xE01B, 0xE021 ], [ 0xE023, 0xE024 ], [ 0xE026, 0xE02A ], [ 0xE030, 0xE06D ],
    [ 0xE08F, 0xE08F ], [ 0xE100, 0xE12C ], [ 0xE130, 0xE13D ], [ 0xE140, 0xE149 ], [ 0xE14E, 0xE14F ],
    [ 0xE290, 0xE2AE ], [ 0xE2C0, 0xE2F9 ], [ 0xE2FF, 0xE2FF ], [ 0xE4D0, 0xE4F9 ], [ 0xE5D0, 0xE5FA ],
    [ 0xE5FF, 0xE5FF ], [ 0xE7E0, 0xE7E6 ], [ 0xE7E8, 0xE7EB ], [ 0xE7ED, 0xE7EE ], [ 0xE7F0, 0xE7FE ],
    [ 0xE800, 0xE8C4 ], [ 0xE8C7, 0xE8D6 ], [ 0xE900, 0xE94B ], [ 0xE950, 0xE959 ], [ 0xE95E, 0xE95F ],
    [ 0xEC71, 0xECB4 ], [ 0xED01, 0xED3D ], [ 0xEE00, 0xEE03 ], [ 0xEE05, 0xEE1F ], [ 0xEE21, 0xEE22 ],
    [ 0xEE24, 0xEE24 ], [ 0xEE27, 0xEE27 ], [ 0xEE29, 0xEE32 ], [ 0xEE34, 0xEE37 ], [ 0xEE39, 0xEE39 ],
    [ 0xEE3B, 0xEE3B ], [ 0xEE42, 0xEE42 ], [ 0xEE47, 0xEE47 ], [ 0xEE49, 0xEE49 ], [ 0xEE4B, 0xEE4B ],
    [ 0xEE4D, 0xEE4F ], [ 0xEE51, 0xEE52 ], [ 0xEE54, 0xEE54 ], [ 0xEE57, 0xEE57 ], [ 0xEE59, 0xEE59 ],
    [ 0xEE5B, 0xEE5B ], [ 0xEE5D, 0xEE5D ], [ 0xEE5F, 0xEE5F ], [ 0xEE61, 0xEE62 ], [ 0xEE64, 0xEE64 ],
    [ 0xEE67, 0xEE6A ], [ 0xEE6C, 0xEE72 ], [ 0xEE74, 0xEE77 ], [ 0xEE79, 0xEE7C ], [ 0xEE7E, 0xEE7E ],
    [ 0xEE80, 0xEE89 ], [ 0xEE8B, 0xEE9B ], [ 0xEEA1, 0xEEA3 ], [ 0xEEA5, 0xEEA9 ], [ 0xEEAB, 0xEEBB ],
    [ 0xEEF0, 0xEEF1 ], [ 0xF000, 0xF02B ], [ 0xF030, 0xF093 ], [ 0xF0A0, 0xF0AE ], [ 0xF0B1, 0xF0BF ],
    [ 0xF0C1, 0xF0CF ], [ 0xF0D1, 0xF0F5 ], [ 0xF100, 0xF1AD ], [ 0xF1E6, 0xF202 ], [ 0xF210, 0xF23B ],
    [ 0xF240, 0xF248 ], [ 0xF250, 0xF251 ], [ 0xF260, 0xF265 ], [ 0xF300, 0xF6D7 ], [ 0xF6DC, 0xF6EC ],
    [ 0xF6F0, 0xF6FC ], [ 0xF700, 0xF776 ], [ 0xF77B, 0xF7D9 ], [ 0xF7E0, 0xF7EB ], [ 0xF7F0, 0xF7F0 ],
    [ 0xF800, 0xF80B ], [ 0xF810, 0xF847 ], [ 0xF850, 0xF859 ], [ 0xF860, 0xF887 ], [ 0xF890, 0xF8AD ],
    [ 0xF8B0, 0xF8BB ], [ 0xF8C0, 0xF8C1 ], [ 0xF900, 0xFA53 ], [ 0xFA60, 0xFA6D ], [ 0xFA70, 0xFA7C ],
    [ 0xFA80, 0xFA89 ], [ 0xFA8F, 0xFAC6 ], [ 0xFACE, 0xFADC ], [ 0xFADF, 0xFAE9 ], [ 0xFAF0, 0xFAF8 ],
    [ 0xFB00, 0xFB92 ], [ 0xFB94, 0xFBF9 ],
  ];
  #[rustfmt::skip]
  static _GRAPH3:[[u32;2];10] = [ [ 0x20000, 0x2A6DF ], [ 0x2A700, 0x2B739 ], [ 0x2B740, 0x2B81D ],
                                  [ 0x2B820, 0x2CEA1 ], [ 0x2CEB0, 0x2EBE0 ], [ 0x2EBF0, 0x2EE5D ],
                                  [ 0x2F800, 0x2FA1D ], [ 0x30000, 0x3134A ], [ 0x31350, 0x323AF ],
                                  [ 0xE0100, 0xE01EF ] ];

  fn _lookup_table16<const N: usize>(c: u16, table: &'static [[u16; 2]; N]) -> bool {
    let mut l: usize = 0;
    let mut r: usize = table.len();
    let mut m;
    while l < r {
      m = (l + r) >> 1;
      if c <= table[m][1] {
        r = m;
      } else {
        l = m + 1;
      }
    }
    l != table.len() && c >= table[l][0]
  }
  fn _lookup_table32<const N: usize>(c: u32, table: &'static [[u32; 2]; N]) -> bool {
    table.iter().any(|[l, r]| c >= *l && c <= *r)
  }

  pub fn _is_identifier_start(u: char) -> bool {
    if u > '\u{1FFFF}' {
      _lookup_table32(u as u32, &_IDENTIFIER_START3)
    } else if u > '\u{FFFF}' {
      _lookup_table16(u as u16, &_IDENTIFIER_START2)
    } else {
      _lookup_table16(u as u16, &_IDENTIFIER_START1)
    }
  }
  pub fn _is_identifier_next(u: char) -> bool {
    _is_identifier_start(u)
      || if u > '\u{1FFFF}' {
        _lookup_table32(u as u32, &_IDENTIFIER_NEXT_DELTA3)
      } else if u > '\u{FFFF}' {
        _lookup_table16(u as u16, &_IDENTIFIER_NEXT_DELTA2)
      } else {
        _lookup_table16(u as u16, &_IDENTIFIER_NEXT_DELTA1)
      }
  }
  pub fn _is_graph(u: char) -> bool {
    if u > '\u{1FFFF}' {
      _lookup_table32(u as u32, &_GRAPH3)
    } else if u > '\u{FFFF}' {
      _lookup_table16(u as u16, &_GRAPH2)
    } else {
      _lookup_table16(u as u16, &_GRAPH1)
    }
  }

  pub fn _escape(u: char, fit_json5: bool) -> Option<char> {
    match u {
      '"' => Some('"'),
      '\\' => Some('\\'),
      '/' => Some('/'),
      'b' => Some('\u{8}'),
      'f' => Some('\u{C}'),
      'n' => Some('\n'),
      'r' => Some('\r'),
      't' => Some('\t'),
      '\'' => fit_json5.then_some('\''),
      'v' => fit_json5.then_some('\u{B}'),
      '0' => fit_json5.then_some('\0'),
      _ => None,
    }
  }

  pub fn _to_digit(c: char) -> u8 {
    unsafe {
      match c {
        '0'..='9' => c as u8 - b'0',
        'a'..='f' => c as u8 - b'a' + 10,
        'A'..='F' => c as u8 - b'A' + 10,
        _ => unreachable_unchecked(),
      }
    }
  }
}
use char_check::*;

impl JsonStreamParser {
  pub fn new(option: JsonOption) -> Self {
    JsonStreamParser {
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

  fn _next_location(state: _LocationState) -> _LocationState {
    unsafe {
      match state {
        _LocationState::RootStart => _LocationState::RootEnd,
        _LocationState::KeyFirstStart | _LocationState::KeyStart => _LocationState::KeyEnd,
        _LocationState::ValueStart => _LocationState::ValueEnd,
        _LocationState::ElementFirstStart | _LocationState::ElementStart => {
          _LocationState::ElementEnd
        }
        _ => unreachable_unchecked(),
      }
    }
  }
  fn _transform_location(location: _LocationState) -> JsonLocation {
    match location {
      _LocationState::RootStart | _LocationState::RootEnd => JsonLocation::Root,
      _LocationState::KeyFirstStart | _LocationState::KeyStart | _LocationState::KeyEnd => {
        JsonLocation::Key
      }
      _LocationState::ValueStart | _LocationState::ValueEnd => JsonLocation::Value,
      _LocationState::ElementFirstStart
      | _LocationState::ElementStart
      | _LocationState::ElementEnd => JsonLocation::Element,
      _ => panic!("invalid location state"),
    }
  }

  fn _handle_comma(&mut self) -> Result<(JsonTokenInfo, JsonLocation), JsonStreamParserError> {
    match self.location {
      _LocationState::ValueEnd => {
        self.location = _LocationState::KeyStart;
        Ok((JsonTokenInfo::ObjectNext, JsonLocation::Object))
      }
      _LocationState::ElementEnd => {
        self.location = _LocationState::ElementStart;
        Ok((JsonTokenInfo::ArrayNext, JsonLocation::Array))
      }
      _LocationState::ElementFirstStart => Err("extra commas not allowed in empty array"),
      _LocationState::ValueStart => Err("unpexted empty value"),
      _ => Err("unexpected comma"),
    }
  }
  fn _handle_array_end(&mut self) -> Result<(JsonTokenInfo, JsonLocation), JsonStreamParserError> {
    match self.location {
      _LocationState::ElementFirstStart | _LocationState::ElementEnd => {
        unsafe {
          self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
        }
        self.state = _ValueState::Empty;
        Ok((JsonTokenInfo::ArrayEnd, Self::_transform_location(self.location)))
      }
      _LocationState::ElementStart => {
        if self.option.accept_trailing_comma_in_array {
          unsafe {
            self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
          }
          self.state = _ValueState::Empty;
          Ok((JsonTokenInfo::ArrayEnd, Self::_transform_location(self.location)))
        } else {
          Err("extra commas not allowed in array")
        }
      }
      _ => Err("bad closing square bracket"),
    }
  }
  fn _handle_object_end(&mut self) -> Result<(JsonTokenInfo, JsonLocation), JsonStreamParserError> {
    match self.location {
      _LocationState::KeyFirstStart | _LocationState::ValueEnd => {
        unsafe {
          self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
        }
        self.state = _ValueState::Empty;
        Ok((JsonTokenInfo::ObjectEnd, Self::_transform_location(self.location)))
      }
      _LocationState::KeyStart => {
        if self.option.accept_trailing_comma_in_object {
          unsafe {
            self.location = Self::_next_location(self.stack.pop().unwrap_unchecked());
          }
          self.state = _ValueState::Empty;
          Ok((JsonTokenInfo::ObjectEnd, Self::_transform_location(self.location)))
        } else {
          Err("extra commas not allowed in object")
        }
      }
      _ => Err("bad closing curly brace"),
    }
  }
  fn _handle_eof(&mut self) -> Result<JsonTokenInfo, JsonStreamParserError> {
    match self.location {
      _LocationState::RootStart | _LocationState::RootEnd => {
        self.location = _LocationState::Eof;
        Ok(JsonTokenInfo::Eof)
      }
      _LocationState::KeyFirstStart
      | _LocationState::KeyStart
      | _LocationState::KeyEnd
      | _LocationState::ValueStart
      | _LocationState::ValueEnd => Err("unexpected EOF while parsing object"),
      _LocationState::ElementFirstStart
      | _LocationState::ElementStart
      | _LocationState::ElementEnd => Err("unexpected EOF while parsing array"),
      _LocationState::Eof => Err("unexpected EOF"),
    }
  }
  fn _handle_slash(&mut self) -> Result<(JsonTokenInfo, JsonLocation), JsonStreamParserError> {
    if self.option.accept_single_line_comment || self.option.accpet_multi_line_comment {
      self.state = _ValueState::CommentMayStart;
      Ok((JsonTokenInfo::CommentSingleLineMayStart, Self::_transform_location(self.location)))
    } else {
      Err("comment not allowed")
    }
  }
  fn _handle_number_separator(
    &mut self,
    c: char,
  ) -> Result<(JsonTokenInfo, JsonLocation), JsonStreamParserError> {
    self.state = _ValueState::Empty;
    let old_location = self.location;
    self.location = Self::_next_location(self.location);
    match c {
      '\0' => Self::_handle_eof(self).map(|info| (info, Self::_transform_location(old_location))),
      '}' => Self::_handle_object_end(self),
      ']' => Self::_handle_array_end(self),
      ',' => Self::_handle_comma(self),
      '/' => Self::_handle_slash(self),
      _ => Err("unexpected character after number"),
    }
  }

  fn _step_empty(&mut self, c: char) -> Result<JsonToken, JsonStreamParserError> {
    let olocation = Self::_transform_location(self.location);

    if _is_whitespace(c, self.option.accept_json5_whitespace) {
      return Ok(JsonToken { c, info: JsonTokenInfo::Whitespace, location: olocation });
    }
    if c == '\0' {
      return self._handle_eof().map(|info| JsonToken { c, info: info, location: olocation });
    }
    if c == '/' {
      return self._handle_slash().map(|info| JsonToken { c, info: info.0, location: info.1 });
    }
    if matches!(self.location, _LocationState::RootEnd) {
      return Err("non-whitespace character after JSON");
    }

    if c == '"' || (c == '\'' && self.option.accept_single_quote) {
      self.state = _ValueState::String(c == '\'');
      return Ok(JsonToken { c, info: JsonTokenInfo::StringStart, location: olocation });
    }
    if c == '\'' {
      return Err("single quote not allowed");
    }
    if matches!(self.location, _LocationState::KeyFirstStart | _LocationState::KeyStart) {
      if self.option.accept_identifier_key {
        if _is_identifier_start(c) {
          self.state = _ValueState::Identifier;
          return Ok(JsonToken {
            c,
            info: JsonTokenInfo::IdentifierNormal,
            location: JsonLocation::Key,
          });
        } else if c == '\\' {
          self.state = _ValueState::IdentifierEscape(false, 0, 0);
          return Ok(JsonToken {
            c,
            info: JsonTokenInfo::IdentifierEscape(None, 0),
            location: JsonLocation::Key,
          });
        }
      }
      if matches!(self.location, _LocationState::KeyFirstStart) && c == ',' {
        return Err("extra commas not allowed in empty object");
      }
      if c != '}' {
        return Err("property name must be a string");
      }
    }

    if c == ':' {
      if matches!(self.location, _LocationState::KeyEnd) {
        self.location = _LocationState::ValueStart;
        self.state = _ValueState::Empty;
        return Ok(JsonToken { c, info: JsonTokenInfo::ObjectValueStart, location: olocation });
      }
      return Err(if matches!(self.location, _LocationState::ValueStart) {
        "repeated colon"
      } else if matches!(self.location, _LocationState::ElementEnd) {
        "unexpected colon in array"
      } else {
        "unexpected colon"
      });
    }
    if matches!(self.location, _LocationState::KeyEnd) {
      return Err("missing colon between key and value");
    }

    match c {
      '[' => {
        self.stack.push(self.location);
        self.location = _LocationState::ElementFirstStart;
        self.state = _ValueState::Empty;
        Ok(JsonToken { c, info: JsonTokenInfo::ArrayStart, location: olocation })
      }
      ']' => self._handle_array_end().map(|(info, location)| JsonToken { c, info, location }),

      '{' => {
        self.stack.push(self.location);
        self.location = _LocationState::KeyFirstStart;
        self.state = _ValueState::Empty;
        Ok(JsonToken { c, info: JsonTokenInfo::ObjectStart, location: olocation })
      }
      '}' => self._handle_object_end().map(|(info, location)| JsonToken { c, info, location }),

      ',' => self._handle_comma().map(|(info, location)| JsonToken { c, info, location }),

      '+' | '-' => {
        if c == '+' && self.option.accept_positive_sign {
          Err("positive sign not allowed")
        } else {
          self.state = _ValueState::Number(_ValueNumberState::Sign);
          Ok(JsonToken { c, info: JsonTokenInfo::NumberIntegerSign, location: olocation })
        }
      }

      '0'..='9' => {
        self.state = _ValueState::Number(if c == '0' {
          _ValueNumberState::Zero
        } else {
          _ValueNumberState::Digit
        });
        Ok(JsonToken { c, info: JsonTokenInfo::NumberIntegerDigit, location: olocation })
      }
      '.' => {
        if self.option.accept_empty_integer {
          self.state = _ValueState::NumberFraction(false);
          Ok(JsonToken { c, info: JsonTokenInfo::NumberFractionStart, location: olocation })
        } else {
          Err("unexpected '.' before number")
        }
      }

      'N' => {
        if self.option.accept_nan {
          self.state = _ValueState::NumberNan(0);
          Ok(JsonToken { c, info: JsonTokenInfo::NumberNan(false, 0), location: olocation })
        } else {
          Err("unexpected character")
        }
      }
      'I' => {
        if self.option.accept_infinity {
          self.state = _ValueState::NumberInfinity(0);
          Ok(JsonToken { c, info: JsonTokenInfo::NumberInfinity(false, 0), location: olocation })
        } else {
          Err("unexpected character")
        }
      }

      'n' => {
        self.state = _ValueState::Null(1);
        Ok(JsonToken { c, info: JsonTokenInfo::Null(false, 0), location: olocation })
      }
      't' => {
        self.state = _ValueState::True(1);
        Ok(JsonToken { c, info: JsonTokenInfo::True(false, 0), location: olocation })
      }
      'f' => {
        self.state = _ValueState::False(1);
        Ok(JsonToken { c, info: JsonTokenInfo::False(false, 0), location: olocation })
      }

      'u' => Err("\"undefined\" is not a valid JSON value"),
      _ => Err("unexpected character"),
    }
  }
  fn _step_string(
    &mut self,
    c: char,
    single_quote: bool,
  ) -> Result<JsonToken, JsonStreamParserError> {
    if c == if single_quote { '\'' } else { '"' } {
      self.location = Self::_next_location(self.location);
      self.state = _ValueState::Empty;
      Ok(JsonToken {
        c,
        info: JsonTokenInfo::StringEnd,
        location: Self::_transform_location(self.location),
      })
    } else if c == '\\' {
      self.state = _ValueState::StringEscape(single_quote);
      Ok(JsonToken {
        c,
        info: JsonTokenInfo::StringEscapeStart,
        location: Self::_transform_location(self.location),
      })
    } else if c == '\0' {
      Err("unexpected EOF while parsing string")
    } else if _is_control(c) {
      Err("unexpected control character in string")
    } else {
      Ok(JsonToken {
        c,
        info: JsonTokenInfo::StringNormal,
        location: Self::_transform_location(self.location),
      })
    }
  }
  fn _step(&mut self, c: char) -> Result<JsonToken, JsonStreamParserError> {
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
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::Null(
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
          Err("unexpected character while parsing \"null\"")
        }
      }
      _ValueState::True(ref mut idx) => {
        let old_idx = *idx;
        if c == _TRUE[*idx as usize] {
          *idx += 1;
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::True(
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
          Err("unexpected character while parsing \"true\"")
        }
      }
      _ValueState::False(ref mut idx) => {
        let old_idx = *idx;
        if c == _FALSE[*idx as usize] {
          *idx += 1;
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::False(
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
          Err("unexpected character while parsing \"false\"")
        }
      }
      _ValueState::NumberInfinity(ref mut idx) => {
        let old_idx = *idx;
        if c == _INFINITY[*idx as usize] {
          *idx += 1;
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::NumberInfinity(
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
          Err("unexpected character while parsing \"Infinity\"")
        }
      }
      _ValueState::NumberNan(ref mut idx) => {
        let old_idx = *idx;
        if c == _NAN[*idx as usize] {
          *idx += 1;
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::NumberNan(
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
          Err("unexpected character while parsing \"NaN\"")
        }
      }

      _ValueState::StringMultilineCr(bl) => {
        if c == '\n' {
          self.state = _ValueState::String(bl);
          Ok(JsonToken { c, info: JsonTokenInfo::StringNextLine, location: olocation })
        } else {
          self._step_string(c, bl)
        }
      }
      _ValueState::String(bl) => self._step_string(c, bl),
      _ValueState::StringEscape(bl) => {
        if c == 'u' {
          self.state = _ValueState::StringUnicode(bl, 0, 0);
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::StringEscapeUnicodeStart,
            location: Self::_transform_location(self.location),
          })
        } else {
          match _escape(c, self.option.accpet_json5_string_escape) {
            Some(escaped_char) => {
              self.state = _ValueState::String(bl);
              Ok(JsonToken {
                c,
                info: JsonTokenInfo::StringEscape(escaped_char),
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
                Ok(JsonToken {
                  c,
                  info: JsonTokenInfo::StringNextLine,
                  location: Self::_transform_location(self.location),
                })
              } else if self.option.accpet_json5_string_escape && c == 'x' {
                self.state = _ValueState::StringEscapeHex(bl, 0, 0);
                Ok(JsonToken {
                  c,
                  info: JsonTokenInfo::StringEscapeHexStart,
                  location: Self::_transform_location(self.location),
                })
              } else {
                Err("bad escape character in string")
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
            Ok(JsonToken {
              c,
              info: JsonTokenInfo::StringEscapeUnicode(char::from_u32(num as u32), 4),
              location: olocation,
            })
          } else {
            Ok(JsonToken {
              c,
              info: JsonTokenInfo::StringEscapeUnicode(None, *idx - 1),
              location: olocation,
            })
          }
        } else {
          Err("bad Unicode escape character")
        }
      }
      _ValueState::StringEscapeHex(bl, idx, num) => {
        if _is_hex(c) {
          if idx == 1 {
            let num = num << 4 | _to_digit(c);
            self.state = _ValueState::String(bl);

            Ok(JsonToken {
              c,
              info: unsafe {
                JsonTokenInfo::StringEscapeHex(Some(char::from_u32_unchecked(num as u32)), 4)
              },
              location: olocation,
            })
          } else {
            self.state = _ValueState::StringEscapeHex(bl, 1, _to_digit(c));
            Ok(JsonToken { c, info: JsonTokenInfo::StringEscapeHex(None, 0), location: olocation })
          }
        } else {
          Err("bad Unicode escape character")
        }
      }

      _ValueState::Number(ref mut state) => {
        if c == '0' {
          if matches!(*state, _ValueNumberState::Zero) {
            Err("leading zero not allowed")
          } else {
            if matches!(*state, _ValueNumberState::Sign) {
              *state = _ValueNumberState::Zero;
            }
            Ok(JsonToken { c, info: JsonTokenInfo::NumberIntegerDigit, location: olocation })
          }
        } else if matches!(c, '1'..='9') {
          if matches!(*state, _ValueNumberState::Zero) {
            Err("leading zero not allowed")
          } else {
            if matches!(*state, _ValueNumberState::Sign) {
              *state = _ValueNumberState::Digit;
            }
            Ok(JsonToken { c, info: JsonTokenInfo::NumberIntegerDigit, location: olocation })
          }
        } else if c == '.' {
          if matches!(*state, _ValueNumberState::Sign) && !self.option.accept_empty_integer {
            Err("unexpected '.' before number")
          } else {
            self.state = _ValueState::NumberFraction(false);
            Ok(JsonToken { c, info: JsonTokenInfo::NumberFractionStart, location: olocation })
          }
        } else if matches!(*state, _ValueNumberState::Sign) {
          if self.option.accept_infinity && c == 'I' {
            self.state = _ValueState::NumberInfinity(1);
            Ok(JsonToken { c, info: JsonTokenInfo::NumberInfinity(false, 0), location: olocation })
          } else if self.option.accept_nan && c == 'N' {
            self.state = _ValueState::NumberNan(1);
            Ok(JsonToken { c, info: JsonTokenInfo::NumberNan(false, 0), location: olocation })
          } else {
            Err("the integer part cannot be empty")
          }
        } else {
          if matches!(*state, _ValueNumberState::Zero) {
            match c {
              'x' | 'X' => {
                if self.option.accept_hexadecimal_integer {
                  self.state = _ValueState::NumberHex(false);
                  return Ok(JsonToken {
                    c,
                    info: JsonTokenInfo::NumberHexStart,
                    location: olocation,
                  });
                }
              }
              'o' | 'O' => {
                if self.option.accept_octal_integer {
                  self.state = _ValueState::NumberOct(false);
                  return Ok(JsonToken {
                    c,
                    info: JsonTokenInfo::NumberOctStart,
                    location: olocation,
                  });
                }
              }
              'b' | 'B' => {
                if self.option.accept_binary_integer {
                  self.state = _ValueState::NumberBin(false);
                  return Ok(JsonToken {
                    c,
                    info: JsonTokenInfo::NumberBinStart,
                    location: olocation,
                  });
                }
              }
              _ => {}
            };
          }

          if c == 'e' || c == 'E' {
            self.state = _ValueState::NumberExponent(_ValueExponentState::Desire);
            Ok(JsonToken { c, info: JsonTokenInfo::NumberExponentStart, location: olocation })
          } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
            self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
          } else {
            Err("unexpected character while the integer part of number")
          }
        }
      }
      _ValueState::NumberFraction(ref mut state) => {
        if matches!(c, '0'..='9') {
          *state = true;
          Ok(JsonToken { c, info: JsonTokenInfo::NumberFractionDigit, location: olocation })
        } else if !*state && !self.option.accept_empty_fraction {
          Err("the fraction part cannot be empty")
        } else if c == 'e' || c == 'E' {
          self.state = _ValueState::NumberExponent(_ValueExponentState::Desire);
          Ok(JsonToken { c, info: JsonTokenInfo::NumberExponentStart, location: olocation })
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
        } else {
          Err("unexpected character while the fraction part of number")
        }
      }
      _ValueState::NumberExponent(ref mut state) => {
        if c == '+' || c == '-' {
          match *state {
            _ValueExponentState::Desire => {
              *state = _ValueExponentState::Sign;
              Ok(JsonToken { c, info: JsonTokenInfo::NumberExponentSign, location: olocation })
            }
            _ValueExponentState::Sign => Err("repeated sign in exponent part"),
            _ValueExponentState::Digit => Err("unexepcted sign in exponent part"),
          }
        } else if matches!(c, '0'..='9') {
          *state = _ValueExponentState::Digit;
          Ok(JsonToken { c, info: JsonTokenInfo::NumberExponentDigit, location: olocation })
        } else if matches!(*state, _ValueExponentState::Desire | _ValueExponentState::Sign) {
          Err("the exponent part cannot be empty")
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
        } else {
          Err("unexpected character while parsing the exponent part of number")
        }
      }

      _ValueState::NumberHex(ref mut state) => {
        if _is_hex(c) {
          *state = true;
          Ok(JsonToken { c, info: JsonTokenInfo::NumberHex, location: olocation })
        } else if c == '.' {
          Err("fraction not allowed in hexadecimal number")
        } else if !*state {
          Err("the hexadecimal integer part cannot be empty")
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
        } else {
          Err("unexpected character while the hexadecimal integer part of number")
        }
      }
      _ValueState::NumberOct(ref mut state) => {
        if matches!(c, '0'..='7') {
          *state = true;
          Ok(JsonToken { c, info: JsonTokenInfo::NumberOct, location: olocation })
        } else if c == '.' {
          Err("fraction not allowed in octal number")
        } else if !*state {
          Err("the octal integer part cannot be empty")
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
        } else {
          Err("unexpected character while the octal integer part of number")
        }
      }
      _ValueState::NumberBin(ref mut state) => {
        if _is_hex(c) {
          *state = true;
          Ok(JsonToken { c, info: JsonTokenInfo::NumberBin, location: olocation })
        } else if c == '.' {
          Err("fraction not allowed in binary number")
        } else if !*state {
          Err("the binary integer part cannot be empty")
        } else if _is_number_separator(c, self.option.accept_json5_whitespace) {
          self._handle_number_separator(c).map(|(info, location)| JsonToken { c, info, location })
        } else {
          Err("unexpected character while the binary integer part of number")
        }
      }

      _ValueState::CommentMayStart => {
        if self.option.accept_single_line_comment && c == '/' {
          self.state = _ValueState::CommentSingleLine;
          Ok(JsonToken { c, info: JsonTokenInfo::CommentSingleLine, location: olocation })
        } else if self.option.accpet_multi_line_comment && c == '*' {
          self.state = _ValueState::CommentMultiLine;
          Ok(JsonToken { c, info: JsonTokenInfo::CommentMultiLine, location: olocation })
        } else {
          Err("slash is not used for comment")
        }
      }
      _ValueState::CommentSingleLine => {
        if _is_next_line(c) {
          self.state = _ValueState::Empty;
        }
        Ok(JsonToken { c, info: JsonTokenInfo::CommentSingleLine, location: olocation })
      }
      _ValueState::CommentMultiLine => {
        if c == '*' {
          self.state = _ValueState::CommentMultiLineMayEnd;
        }
        Ok(JsonToken { c, info: JsonTokenInfo::CommentMultiLine, location: olocation })
      }
      _ValueState::CommentMultiLineMayEnd => {
        if c == '/' {
          self.state = _ValueState::Empty;
          Ok(JsonToken {
            c,
            info: JsonTokenInfo::CommentMultiLineEnd,
            location: Self::_transform_location(self.location),
          })
        } else if c == '*' {
          Ok(JsonToken { c, info: JsonTokenInfo::CommentMultiLine, location: olocation })
        } else {
          self.state = _ValueState::CommentMultiLine;
          Ok(JsonToken { c, info: JsonTokenInfo::CommentMultiLine, location: olocation })
        }
      }

      _ValueState::Identifier => {
        if c == ':' {
          self.location = _LocationState::ValueStart;
          self.state = _ValueState::Empty;
          Ok(JsonToken { c, info: JsonTokenInfo::ObjectValueStart, location: JsonLocation::Object })
        } else if _is_whitespace(c, self.option.accept_json5_whitespace) {
          self.location = _LocationState::KeyEnd;
          self.state = _ValueState::Empty;
          Ok(JsonToken { c, info: JsonTokenInfo::Whitespace, location: JsonLocation::Key })
        } else if _is_identifier_next(c) {
          Ok(JsonToken { c, info: JsonTokenInfo::IdentifierNormal, location: JsonLocation::Key })
        } else {
          Err("unexpected character in identifier")
        }
      }
      _ValueState::IdentifierEscape(ref mut prefix, ref mut idx, ref mut num) => {
        if !*prefix {
          if c == 'u' {
            *prefix = true;
            Ok(JsonToken {
              c,
              info: JsonTokenInfo::IdentifierEscapeStart(true, 1),
              location: JsonLocation::Key,
            })
          } else {
            Err("expect 'u' after '\\' in identifier")
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
              Ok(JsonToken {
                c,
                info: JsonTokenInfo::IdentifierEscape(char::from_u32(num as u32), 4),
                location: JsonLocation::Key,
              })
            } else {
              Ok(JsonToken {
                c,
                info: JsonTokenInfo::IdentifierEscape(None, *idx - 1),
                location: JsonLocation::Key,
              })
            }
          } else {
            Err("expected hexadecimal number after \"\\u\" in identifier")
          }
        }
      }
    }
  }

  pub fn feed_one(&mut self, c: char) -> Result<JsonToken, JsonStreamParserError> {
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
  pub fn feed(&mut self, s: &str) -> Result<Vec<JsonToken>, (usize, JsonStreamParserError)> {
    let mut tokens = Vec::new();
    let mut cnt = 0;
    for c in s.chars() {
      match self.feed_one(c) {
        Ok(token) => tokens.push(token),
        Err(e) => return Err((cnt, e)),
      }
      cnt += 1;
    }
    Ok(tokens)
  }
  pub fn end(&mut self) -> Result<JsonToken, JsonStreamParserError> {
    self.feed_one('\0')
  }

  pub fn get_position(&self) -> JsonParserPosition {
    self.position
  }
  pub fn get_line(&self) -> JsonParserPosition {
    self.line
  }
  pub fn get_column(&self) -> JsonParserPosition {
    self.column
  }
}

pub fn json_stream_parse(
  s: &str,
  option: JsonOption,
) -> Result<Vec<JsonToken>, (usize, JsonStreamParserError)> {
  let mut parser = JsonStreamParser::new(option);

  let mut tokens = Vec::new();
  let mut cnt = 0;
  for c in s.chars() {
    match parser.feed_one(c) {
      Ok(token) => tokens.push(token),
      Err(e) => return Err((cnt, e)),
    }
    cnt += 1;
  }
  match parser.end() {
    Ok(token) => tokens.push(token),
    Err(e) => return Err((cnt, e)),
  }
  Ok(tokens)
}
