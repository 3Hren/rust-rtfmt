#![feature(plugin)]
#![feature(question_mark)]
#![plugin(peg_syntax_ext)]

//# format_string := <text> [ format <text> ] *
// format := '{' [ argument ] [ ':' format_spec ] '}'
// argument := integer | identifier
//
// format_spec := [[fill]align][sign]['#'][0][width]['.' precision][type]
// fill := character
// align := '<' | '^' | '>'
// sign := '+' | '-'
// width := count
// precision := count | '*'
// type := identifier | ''
// count := parameter | integer
// parameter := integer '$'

extern crate serde_json;

pub use serde_json::Value;

use grammar::{expression, ParseError};

const OPENED_BRACE: char = '{';
const CLOSED_BRACE: char = '}';

peg! grammar(r#"
use Key;
use Token;
use OPENED_BRACE;
use CLOSED_BRACE;

#[pub]
expression -> Vec<Token>
    = (format / text)+
text -> Token
    = result:text_char+ { Token::Literal(result.into_iter().collect()) }
text_char -> char
    = "{{" { OPENED_BRACE }
    / "}}" { CLOSED_BRACE }
    / [^{}] { match_str.chars().next().unwrap() }
format -> Token
    = "{" keys:(name ++ ".") "}" { Token::Placeholder(keys) }
name -> Key
    = [0-9]+ { Key::Id(match_str.parse().unwrap()) }
    / [a-zA-Z][a-zA-Z0-9]* { Key::Name(match_str.into()) }
"#);

// TODO: Format spec.
// TODO: Error cases.
// TODO: Remove all unwraps().
// TODO: Refactoring and symbol visibility.
// TODO: Benchmarking.

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Id(usize),
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(String),
    Placeholder(Vec<Key>),
}

pub struct Generator {
    tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum Error {
    __Unknown,
}

fn find<'r>(value: &'r Value, key: &Key) -> Option<&'r Value> {
    match *key {
        Key::Id(id) => {
            if let &Value::Array(ref value) = value {
                value.get(id)
            } else {
                None
            }
        }
        Key::Name(ref name) => value.find(name),
    }
}

impl Generator {
    pub fn new(pattern: &str) -> Result<Generator, ParseError> {
        let result = Generator {
            tokens: expression(pattern)?,
        };

        Ok(result)
    }

    pub fn consume(&self, value: &Value) -> Result<String, Error> {
        let mut result = String::new();

        for token in &self.tokens {
            match *token {
                Token::Literal(ref literal) => result.push_str(&literal[..]),
                Token::Placeholder(ref format) => {
                    let mut current = value;
                    for key in format {
                        match find(&current, key) {
                            Some(value) => { current = value; }
                            None => unimplemented!(),
                        }
                    }

                    result.push_str(&serde_json::to_string(&current).unwrap()[..]);
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::{Generator, Key, Token, Value};

    #[test]
    fn literal() {
        assert_eq!("hello", &Generator::new("hello").unwrap().consume(&Value::Null).unwrap());
    }

    #[test]
    fn literal_ast() {
        let generator = Generator::new("hello").unwrap();

        assert_eq!(vec![Token::Literal("hello".into())], generator.tokens);
    }

    #[test]
    fn placeholder() {
        let mut map = BTreeMap::new();
        map.insert("hello".into(), Value::I64(42));

        let value = Value::Object(map);

        assert_eq!("42", &Generator::new("{hello}").unwrap().consume(&value).unwrap());
    }

    #[test]
    fn placeholder_ast() {
        let generator = Generator::new("{hello}").unwrap();

        let expected = vec![Token::Placeholder(vec![Key::Name("hello".into())])];
        assert_eq!(expected, generator.tokens);
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
}
