# efjson: 一个流式和事件解析的 JSON 解析器

[English](./README.md) [简体中文](./README.zh.md)

其它编程语言:

- [Typescript](https://github.com/DreamPast/efjson)
- [C/C++](https://github.com/DreamPast/efjson-cpp)

## 特性

- 支持 JSON5 和 JSONC
- 在无事件的情况下，流解析器只需要极少的内存
- 提供反序列化功能，而且这也是流式的

## 例子

### 流式解析

```rust
use efjson::{stream_parser::StreamParser, ParserOption};

const SRC: &'static str = r#"{
"N":null,"T":true,"F":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;
fn test_stream() {
  let tokens = StreamParser::parse(ParserOption::default(), SRC).unwrap();
  for item in tokens {
    println!("{:?}", item);
  }
}

fn main() {
  test_stream();
}
```

### 反序列化

```rust
use std::collections::HashMap;

use efjson::{
  JsonValue, ParserOption,
  deserialize::{JsonRawString, JsonRawToken, deserialize},
};

fn test_deserialize() {
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[1,.1,1.,1.234e3]"));
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[0x1234,0o1234,0b1011]"));
  println!("{:?}", deserialize::<Vec<f64>>(ParserOption::all(), "[Infinity,-Infinity,NaN]"));
  println!("{:?}", deserialize::<Vec<i64>>(ParserOption::all(), "[-0x1234,0o1234,+0b1011]"));
  println!(
    "{:?}\t{:?}",
    deserialize::<Vec<bool>>(ParserOption::all(), "[true,false]"),
    deserialize::<()>(ParserOption::all(), "null")
  );
  println!(
    "{:?}",
    deserialize::<[i32; 4]>(ParserOption::all(), "[1,2,3,4]")
  );
  println!(
    "{:?}",
    deserialize::<HashMap<String, Option<i32>>>(ParserOption::all(), "{'a':1,'b':null,}")
  );

  {
    println!("[");
    for token in
      deserialize::<JsonRawToken>(ParserOption::all(), r#"{'a':12,b:[13,14]}"#).unwrap().tokens
    {
      println!("    {:?}", token);
    }
    println!("]");
  }

  println!(
    "{:?}\t",
    vec![
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"[]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",]"#),
      deserialize::<(String, i32)>(ParserOption::all(), r#"["a",12,13]"#)
    ]
  );

  println!(
    "{:?}",
    deserialize::<HashMap<String, JsonRawString>>(
      ParserOption::all(),
      r#"{'a':1.2e3,'b':null,'c':"str",'d':[13,14],'e':{a:12,},}"#
    )
    .unwrap()
    .iter()
    .map(|(k, v)| (k, &v.json))
    .collect::<HashMap<_, _>>()
  );
  println!(
    "{:?}",
    deserialize::<Vec<JsonRawString>>(
      ParserOption::all(),
      r#"[1.2e3,null,"str",[13,14,],{a:12,},]"#
    )
    .unwrap()
    .iter_mut()
    .map(|x| &x.json)
    .collect::<Vec<_>>()
  );
  println!(
    "{:?}",
    deserialize::<JsonValue>(ParserOption::all(), r#"[1.2e3,null,"str",[13,14,],{"a":12,},]"#)
      .unwrap()
  );
}

fn main() {
  test_deserialize();
}
```

并且你也可以使用派生（`derive`）:

```rust
use std::collections::HashMap;

use efjson::{deserialize::deserialize, Deserializable, ParserOption};

const SRC: &'static str = r#"{
"n":null,"t":true,"f":false,
"str":"str,\"esc\",\uD83D\uDE00,😊",
"num":-1.2e3,"arr":["A",{"obj":"B"}]
}"#;

#[derive(Debug, Deserializable)]
#[allow(dead_code)]
struct Struct {
  n: (),
  t: bool,
  f: bool,
  str: String,
  num: f64,
  arr: (String, HashMap<String, String>),
}

fn main() {
  let r: Result<Struct, _> = deserialize(ParserOption::all(), SRC);
  println!("{:#?}", r);
}
```

## 引用

JSON 规范：[RFC 4627 on Json](https://www.ietf.org/rfc/rfc4627.txt)

JSON 状态图：[JSON](https://www.json.org/)

JSON5 规范：[The JSON5 Data Interchange Format](https://spec.json5.org/)

JSON 指针: [JavaScript Object Notation (JSON) Pointer](https://datatracker.ietf.org/doc/html/rfc6901)

相对 JSON 指针: [Relative JSON Pointers](https://datatracker.ietf.org/doc/html/draft-bhutton-relative-json-pointer-00)
