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


const OPENED_BRACE: &'static str = "{";
const CLOSED_BRACE: &'static str = "}";

peg! grammar(r#"
use super::{Key, Token, OPENED_BRACE, CLOSED_BRACE};

#[pub]
expression -> Vec<Token<'input>>
    = (format / text)+
text -> Token<'input>
    = "{{" { Token::Literal(OPENED_BRACE) }
    / "}}" { Token::Literal(CLOSED_BRACE) }
    / [^{}]+ { Token::Literal(match_str) }
format -> Token<'input>
    = "{" keys:(name ++ ".") "}" {
        Token::Placeholder(&match_str[1..match_str.len() - 1], keys)
    }
name -> Key<'input>
    = [0-9]+ { Key::Id(match_str.parse().expect("expect number")) }
    / [a-zA-Z][a-zA-Z0-9]* { Key::Name(match_str) }
"#);

// TODO: Format spec.

#[derive(Debug, Clone, PartialEq)]
pub enum Key<'a> {
    Id(usize),
    Name(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Literal(&'a str),
    Placeholder(&'a str, Vec<Key<'a>>),
}

pub struct Generator<'a> {
    pattern: &'a str,
    tokens: Vec<Token<'a>>,
}

#[derive(Debug)]
pub enum Error<'a> {
    KeyNotFound(&'a str),
    Serialization(serde_json::Error),
}

impl<'a> From<serde_json::Error> for Error<'a> {
    fn from(err: serde_json::Error) -> Error<'a> {
        Error::Serialization(err)
    }
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

impl<'a> Generator<'a> {
    pub fn new(pattern: &'a str) -> Result<Generator<'a>, ParseError> {
        let result = Generator {
            pattern: pattern,
            tokens: expression(pattern)?,
        };

        Ok(result)
    }

    pub fn pattern(&self) -> &str {
        self.pattern
    }

    pub fn consume(&self, val: &Value) -> Result<String, Error> {
        let mut res = String::new();

        for token in &self.tokens {
            match *token {
                Token::Literal(ref literal) => res.push_str(&literal[..]),
                Token::Placeholder(ref name, ref keys) => {
                    let mut cur = val;
                    for key in keys {
                        match find(&cur, key) {
                            Some(val) => {
                                cur = val;
                            }
                            None => {
                                return Err(Error::KeyNotFound(name));
                            }
                        }
                    }

                    res.push_str(&serde_json::to_string(&cur)?[..]);
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

        assert_eq!(vec![Token::Literal("hello")], generator.tokens);
    }

    #[test]
    fn placeholder_ast() {
        let generator = Generator::new("{hello}").unwrap();

        let expected = vec![Token::Placeholder("hello", vec![Key::Name("hello")])];
        assert_eq!(expected, generator.tokens);
    }
}
