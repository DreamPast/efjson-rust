# efjson: 一个流式和事件解析的 JSON 解析器

[English](./README.md) [简体中文](./README.zh.md)

其它编程语言:

- [**Typescript**](https://github.com/DreamPast/efjson)
- [C/C++](https://github.com/DreamPast/efjson-cpp)

## 特性

- 无额外依赖
- 支持 JSON5 和 JSONC
- 在无事件的情况下，流解析器只需要极少的内存

## 例子

### 流式解析

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

## 引用

JSON 规范：[RFC 4627 on Json](https://www.ietf.org/rfc/rfc4627.txt)

JSON 状态图：[JSON](https://www.json.org/)

JSON5 规范：[The JSON5 Data Interchange Format](https://spec.json5.org/)

JSON 指针: [JavaScript Object Notation (JSON) Pointer](https://datatracker.ietf.org/doc/html/rfc6901)

相对 JSON 指针: [Relative JSON Pointers](https://datatracker.ietf.org/doc/html/draft-bhutton-relative-json-pointer-00)
