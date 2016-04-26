#![feature(test)]

extern crate rtfmt;
extern crate test;

use std::collections::BTreeMap;

use rtfmt::{Generator, Value};

#[test]
fn literal() {
    assert_eq!("hello", &Generator::new("hello").unwrap().consume(&Value::Null).unwrap());
}

#[test]
fn placeholder() {
    let mut map = BTreeMap::new();
    map.insert("hello".into(), Value::I64(42));

    let value = Value::Object(map);

    assert_eq!("42", &Generator::new("{hello}").unwrap().consume(&value).unwrap());
}

#[test]
fn literal_squash() {
    let g = Generator::new("{{surrounded}}").unwrap();

    assert_eq!("{surrounded}", g.consume(&Value::Null).unwrap());
}

#[test]
fn literal_squash_beatle() {
    assert_eq!("}{", &Generator::new("}}{{").unwrap().consume(&Value::Null).unwrap());
}

#[test]
fn literal_with_placeholder() {
    let g = Generator::new("le message: {id}!").unwrap();

    let mut map = BTreeMap::new();
    map.insert("id".into(), Value::I64(42));

    let value = Value::Object(map);

    assert_eq!("le message: 42!", g.consume(&value).unwrap());
}

#[test]
fn literal_with_nested_placeholder() {
    let g = Generator::new("le message: {key.nested}!").unwrap();

    let mut map = BTreeMap::new();
    let mut key = BTreeMap::new();
    key.insert("nested".into(), Value::I64(42));
    map.insert("key".into(), Value::Object(key));

    let value = Value::Object(map);

    assert_eq!("le message: 42!", g.consume(&value).unwrap());
}

#[test]
fn literal_with_nested_placeholder_with_array() {
    let g = Generator::new("le message: {0.1}!").unwrap();

    let mut nested = Vec::new();
    nested.push(Value::I64(42));
    nested.push(Value::String("value".into()));

    let mut array = Vec::new();
    array.push(Value::Array(nested));

    let value = Value::Array(array);

    assert_eq!("le message: \"value\"!", g.consume(&value).unwrap());
}
