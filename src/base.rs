bitflags::bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
 pub struct ParserOption: u32 {
    /* << white space >> */
    /** whether to accept whitespace in JSON5 */
    const JSON5_WHITESPACE         = 0x000001;
    /* << array >> */
    /** whether to accept a single trailing comma in array (for example, `[1,]`) */
    const TRAILING_COMMA_IN_ARRAY  = 0x000002;
    /* << object >> */
    /** whether to accept a single trailing comma in object (for example, `{"a":1,}`) */
    const TRAILING_COMMA_IN_OBJECT = 0x000004;
    /** whether to accept identifier key in object (for example, `{a:1}`) */
    const IDENTIFIER_KEY           = 0x000008;
    /* << string >> */
    /** whether to accept single quote in string (for example, `'a'`) */
    const SINGLE_QUOTE             = 0x000010;
    /** whether to accept multi-line string (for example, `"a\\\nb"`) */
    const MULTILINE_STRING         = 0x000020;
    /** whether to accept JSON5 string escape (for example, `"\\x01"`, `\\v`, `\\0`) */
    const JSON5_STRING_ESCAPE      = 0x000040;
    /* << number >> */
    /** whether to accept positive sign in number (for example, `+1`, `+0`) */
    const POSITIVE_SIGN            = 0x000080;
    /** whether to accept empty fraction in number (for example, `1.`, `0.`) */
    const EMPTY_FRACTION           = 0x000100;
    /** whether to accept empty integer in number (for example, `.1`, `.0`) */
    const EMPTY_INTEGER            = 0x000200;
    /** whether to accept NaN */
    const NAN                      = 0x000400;
    /** whether to accept Infinity */
    const INFINITY                 = 0x000800;
    /** whether to accept hexadecimal integer (for example, `0x1`, `0x0`) */
    const HEXADECIMAL_INTEGER      = 0x001000;
    /** whether to accept octal integer (for example, `0o1`, `0o0`) */
    const OCTAL_INTEGER            = 0x002000;
    /** whether to accept binary integer (for example, `0b1`, `0b0`) */
    const BINARY_INTEGER           = 0x004000;
    /* << comment >> */
    /** whether to accept single line comment (for example, `// a comment`) */
    const SINGLE_LINE_COMMENT      = 0x008000;
    /** whether to accept multi-line comment */
    const MULTI_LINE_COMMENT       = 0x010000;
    /* << other >> */
    /** whether to allow empty json value */
    const ALLOW_EMPTY_VALUE        = 0x020000;
  }
}
impl Default for ParserOption {
  fn default() -> Self {
    Self::empty()
  }
}
impl ParserOption {
  pub fn make_jsonc() -> Self {
    Self::SINGLE_LINE_COMMENT | Self::MULTI_LINE_COMMENT
  }
  pub fn make_json5() -> Self {
    Self::SINGLE_LINE_COMMENT
      | Self::MULTI_LINE_COMMENT
      | Self::JSON5_WHITESPACE
      | Self::TRAILING_COMMA_IN_ARRAY
      | Self::TRAILING_COMMA_IN_OBJECT
      | Self::IDENTIFIER_KEY
      | Self::SINGLE_QUOTE
      | Self::MULTILINE_STRING
      | Self::JSON5_STRING_ESCAPE
      | Self::POSITIVE_SIGN
      | Self::EMPTY_FRACTION
      | Self::EMPTY_INTEGER
      | Self::NAN
      | Self::INFINITY
      | Self::HEXADECIMAL_INTEGER
  }
}

#[derive(Clone, Debug)]
pub enum JsonValue {
  NULL,
  BOOL(bool),
  NUMBER(f64),
  STRING(String),
  ARRAY(JsonArray),
  OBJECT(JsonObject),
}
pub type JsonArray = Vec<JsonValue>;
pub type JsonObject = std::collections::HashMap<String, JsonValue>;

impl From<()> for JsonValue {
  fn from(_: ()) -> JsonValue {
    JsonValue::NULL
  }
}
impl From<f64> for JsonValue {
  fn from(val: f64) -> Self {
    JsonValue::NUMBER(val)
  }
}
impl From<bool> for JsonValue {
  fn from(val: bool) -> Self {
    JsonValue::BOOL(val)
  }
}
impl From<String> for JsonValue {
  fn from(val: String) -> Self {
    JsonValue::STRING(val)
  }
}
impl From<JsonArray> for JsonValue {
  fn from(val: JsonArray) -> Self {
    JsonValue::ARRAY(val)
  }
}
impl From<JsonObject> for JsonValue {
  fn from(val: JsonObject) -> Self {
    JsonValue::OBJECT(val)
  }
}
