#![feature(plugin)]
#![feature(question_mark)]
#![plugin(peg_syntax_ext)]

extern crate serde_json;

mod grammar;

pub use grammar::{Generator, ParseError, Value};
