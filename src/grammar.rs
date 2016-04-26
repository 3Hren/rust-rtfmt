use serde_json;
pub use serde_json::Value;

pub use self::grammar::{expression, ParseError};

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


const OPENED_BRACE: char = '{';
const CLOSED_BRACE: char = '}';

peg! grammar(r#"
use super::Key;
use super::Token;
use super::OPENED_BRACE;
use super::CLOSED_BRACE;

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
pub enum Error<'a> {
    KeyNotFound(&'a Vec<Key>),
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

    pub fn consume(&self, val: &Value) -> Result<String, Error> {
        let mut res = String::new();

        for token in &self.tokens {
            match *token {
                Token::Literal(ref literal) => res.push_str(&literal[..]),
                Token::Placeholder(ref format) => {
                    let mut cur = val;
                    for key in format {
                        match find(&cur, key) {
                            Some(val) => {
                                cur = val;
                            }
                            None => {
                                return Err(Error::KeyNotFound(format));
                            }
                        }
                    }

                    res.push_str(&serde_json::to_string(&cur).unwrap()[..]);
                }
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::{Generator, Key, Token};

    #[test]
    fn literal_ast() {
        let generator = Generator::new("hello").unwrap();

        assert_eq!(vec![Token::Literal("hello".into())], generator.tokens);
    }

    #[test]
    fn placeholder_ast() {
        let generator = Generator::new("{hello}").unwrap();

        let expected = vec![Token::Placeholder(vec![Key::Name("hello".into())])];
        assert_eq!(expected, generator.tokens);
    }
}
