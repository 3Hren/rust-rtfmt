#![feature(test)]

extern crate rtfmt;
extern crate test;

use std::collections::BTreeMap;
use test::Bencher;

use rtfmt::{Generator, Value};

#[bench]
fn complex(b: &mut Bencher) {
    let g = Generator::new("le message: {id}!").unwrap();

    let mut map = BTreeMap::new();
    map.insert("id".into(), Value::I64(42));

    let value = Value::Object(map);

    b.iter(|| {
        let result = g.consume(&value).unwrap();
        test::black_box(result);
    });
}
