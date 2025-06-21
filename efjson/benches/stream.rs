use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use efjson::{ParserOption, stream_parser::StreamParser};
use rand::random_range;

fn bench_no_save(option: ParserOption, s: &str) {
  let mut parser = StreamParser::new(option);
  for c in s.chars() {
    let _ = parser.feed_one(c);
  }
  let _ = parser.feed_one('\0');
  let _ = parser;
}

fn gen_array() -> String {
  let mut s1 = String::from("[");
  s1.push_str(&"100,".repeat(1000));
  s1.pop();
  s1.push_str("],");

  let mut s = String::from("[");
  s.push_str(&s1.repeat(1000));
  s.pop();
  s.push_str("]");

  s
}
fn gen_object() -> String {
  let table: &[u8; 62] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut s1 = String::from("{");
  for i in 0..500 {
    s1.push_str(&format!(
      r#""{}":"{}","#,
      table[i % table.len()] as char,
      table[i % table.len()] as char
    ));
  }
  s1.pop();
  s1.push_str("}");

  let mut s = String::from("{");
  for i in 0..2000 {
    s.push_str(&format!(r#""{}":{},"#, table[i % table.len()] as char, s1));
  }
  s.pop();
  s.push_str("}");

  s
}
fn gen_number() -> String {
  let mut s = String::from("-1");
  for i in 1..(1000000 as usize) {
    s.push(((i % 10) as u8 + b'1') as char);
  }
  s.push('.');
  for i in 0..(1000000 as usize) {
    s.push(((i % 10) as u8 + b'1') as char);
  }
  s.push('e');
  for i in 0..(1000000 as usize) {
    s.push(((i % 10) as u8 + b'1') as char);
  }

  s
}
fn gen_string() -> String {
  let mut s = String::from("\"");
  for _ in 0..1000000 {
    let u = match random_range(0..16) {
      0 => random_range(0x10000..0x110000),
      1..4 => random_range(0x80..0xD800),
      _ => random_range(0x20..0x80),
    };

    if u > 0xFFFF {
      s.push_str(&format!("\\u{{{:04x}}}\\u{{{:04x}}}", (u >> 10) + 0xD800, (u & 0x3FF) + 0xDC00));
    } else if u > 0x80 {
      s.push_str(&format!("\\u{{{:04x}}}", u));
    } else {
      s.push(u as u8 as char);
    }
  }
  s.push('"');
  s
}
fn gen_recursive_array() -> String {
  let mut s = "[".repeat(2000000);
  s.push('1');
  s.push_str(&"]".repeat(2000000));
  s
}

fn bench_stream(c: &mut Criterion) {
  {
    let s = gen_array();
    c.bench_function("array", |b| {
      b.iter(|| StreamParser::parse(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_object();
    c.bench_function("object", |b| {
      b.iter(|| StreamParser::parse(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_number();
    c.bench_function("number", |b| {
      b.iter(|| StreamParser::parse(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_string();
    c.bench_function("string", |b| {
      b.iter(|| StreamParser::parse(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_recursive_array();
    c.bench_function("recursive_array", |b| {
      b.iter(|| StreamParser::parse(ParserOption::all(), &s));
    });
  }
}
fn bench_stream_no_save(c: &mut Criterion) {
  {
    let s = gen_array();
    c.bench_function("[no_save]array", |b| {
      b.iter(|| bench_no_save(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_object();
    c.bench_function("[no_save]object", |b| {
      b.iter(|| bench_no_save(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_number();
    c.bench_function("[no_save]number", |b| {
      b.iter(|| bench_no_save(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_string();
    c.bench_function("[no_save]string", |b| {
      b.iter(|| bench_no_save(ParserOption::all(), &s));
    });
  }
  {
    let s = gen_recursive_array();
    c.bench_function("[no_save]recursive_array", |b| {
      b.iter(|| bench_no_save(ParserOption::all(), &s));
    });
  }
}

criterion_group! {
  name=benches;
  config=Criterion::default().sample_size(10).measurement_time(Duration::from_secs(10));
  targets=bench_stream,bench_stream_no_save
}
criterion_main!(benches);
