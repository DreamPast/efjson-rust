#[derive(Default, Clone, Debug)]
pub struct ParserOption {
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
  pub accept_json5_string_escape: bool,

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
  pub accept_multi_line_comment: bool,
}
impl ParserOption {
  pub fn new_jsonc() -> Self {
    ParserOption {
      // << comment >>
      accept_single_line_comment: true,
      accept_multi_line_comment: true,
      ..Default::default()
    }
  }
  pub fn new_json5() -> Self {
    ParserOption {
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
      accept_json5_string_escape: true,
      // << number >>
      accept_positive_sign: true,
      accept_empty_fraction: true,
      accept_empty_integer: true,
      accept_nan: true,
      accept_infinity: true,
      accept_hexadecimal_integer: true,
      // << comment >>
      accept_single_line_comment: true,
      accept_multi_line_comment: true,
      ..Default::default()
    }
  }
  pub fn new_full() -> Self {
    return ParserOption {
      accept_octal_integer: true,
      accept_binary_integer: true,
      ..Self::new_json5()
    };
  }
}
impl Into<u32> for ParserOption {
  fn into(self) -> u32 {
    let mut ret = 0;
    if self.accept_json5_whitespace {
      ret |= 0x000001;
    }
    if self.accept_trailing_comma_in_array {
      ret |= 0x000002;
    }
    if self.accept_trailing_comma_in_object {
      ret |= 0x000004;
    }
    if self.accept_identifier_key {
      ret |= 0x000008;
    }
    if self.accept_single_quote {
      ret |= 0x000010;
    }
    if self.accept_multiline_string {
      ret |= 0x000020;
    }
    if self.accept_json5_string_escape {
      ret |= 0x000040;
    }
    if self.accept_positive_sign {
      ret |= 0x000080;
    }
    if self.accept_empty_fraction {
      ret |= 0x000100;
    }
    if self.accept_empty_integer {
      ret |= 0x000200;
    }
    if self.accept_nan {
      ret |= 0x000400;
    }
    if self.accept_infinity {
      ret |= 0x000800;
    }
    if self.accept_hexadecimal_integer {
      ret |= 0x001000;
    }
    if self.accept_octal_integer {
      ret |= 0x002000;
    }
    if self.accept_binary_integer {
      ret |= 0x004000;
    }
    if self.accept_single_line_comment {
      ret |= 0x008000;
    }
    if self.accept_multi_line_comment {
      ret |= 0x010000;
    }
    ret
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

impl Into<JsonValue> for () {
  fn into(self) -> JsonValue {
    JsonValue::NULL
  }
}
impl Into<JsonValue> for f64 {
  fn into(self) -> JsonValue {
    JsonValue::NUMBER(self)
  }
}
impl Into<JsonValue> for bool {
  fn into(self) -> JsonValue {
    JsonValue::BOOL(self)
  }
}
impl Into<JsonValue> for String {
  fn into(self) -> JsonValue {
    JsonValue::STRING(self)
  }
}
impl Into<JsonValue> for JsonArray {
  fn into(self) -> JsonValue {
    JsonValue::ARRAY(self)
  }
}
impl Into<JsonValue> for JsonObject {
  fn into(self) -> JsonValue {
    JsonValue::OBJECT(self)
  }
}
