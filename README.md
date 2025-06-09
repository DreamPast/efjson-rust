# A Streaming and Event-driven JSON Parser

[English](./README.md) [简体中文](./README.zh.md)

Other programming languages:

- [**Typescript**](https://github.com/DreamPast/efjson)
- [C/C++](https://github.com/DreamPast/efjson-cpp)

## Features

- no extra dependencies
- supports JSON5 and JSONC
- stream parser requires minimal memory when no events are triggered

## Example

### Stream Parsing

```rust
use efjson_rust::{JsonOption, json_stream_parse};
fn main() {
  const SRC: &'static str = r#"{
  "null": null,
  "true": true,
  "false": false,

  "string": "string",
  "string_with_escape": "string with \"escape\"",
  "string_with_unicode_escape": "string with \uD83D\uDE00",
  "string_with_unicode": "string with 😊",

  "integer": 1234,
  "negative": -1234,
  "number": 1234.5678,
  "number_with_exponent": 1.234e2,

  "array": [
    "this is the first element",
    {
      "object": "a nesting object"
    }
  ],
  "object": {
    "1st": [],
    "2st": {}
  }
}"#;
  println!("{}", SRC);
  for item in json_stream_parse(SRC, JsonOption::default()).unwrap() {
    println!("{:?}", item);
  }
}
```

## References

JSON Specification: [RFC 4627 on JSON](https://www.ietf.org/rfc/rfc4627.txt)

JSON State Diagram: [JSON](https://www.json.org/)

JSON5 Specification: [The JSON5 Data Interchange Format](https://spec.json5.org/)

JSON Pointer: [JavaScript Object Notation (JSON) Pointer](https://datatracker.ietf.org/doc/html/rfc6901)

Relative JSON Pointers: [Relative JSON Pointers](https://datatracker.ietf.org/doc/html/draft-bhutton-relative-json-pointer-00)
