use std::ops::{Deref, DerefMut};

use crate::output::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line:   u32,
    pub offset: Option<u32>
}

#[derive(Clone)]
pub struct Spanned<T> {
    pub item: T,
    pub from: Position,
    pub to:   Position
}

impl<T> std::fmt::Debug for Spanned<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Span [{}:{}, {}:{}], Item: {:?}",
            self.from.line,
            self.from.offset.unwrap_or_default(),
            self.to.line,
            self.to.offset.unwrap_or_default(),
            self.item
        ))
    }
}

impl<T: Copy> Copy for Spanned<T> {}

impl<T> Spanned<T> {
    pub fn new(item: T, from: Position, to: Position) -> Spanned<T> {
        Spanned { item, from, to }
    }

    pub fn empty() -> Spanned<()> {
        Spanned {
            item: (),
            from: Position { line: 0, offset: None },
            to:   Position { line: 0, offset: None }
        }
    }

    pub fn encompass<A, B>(item: T, s1: Spanned<A>, s2: Spanned<B>) -> Spanned<T> {
        Spanned { item, from: s1.from, to: s2.to }
    }

    pub fn map<U, F>(&self, f: F) -> Spanned<U>
    where
        F: FnOnce(&T) -> U
    {
        Spanned {
            from: self.from,
            to:   self.to,
            item: f(&self.item)
        }
    }

    pub fn just_span(&self) -> Spanned<()> {
        self.map(|_| ())
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

#[derive(Debug, Clone)]
pub enum NumericLiteral {
    PositiveBinary(u64),
    NegativeBinary(i64),
    Boolean(bool),
    PositiveDecimal(u64),
    NegativeDecimal(i64),
    PositiveHexadecimal(u64),
    NegativeHexadecimal(i64),
    Float(f64)
}

impl PartialEq for NumericLiteral {
    /// Evaluate the raw numeric value of the literals, casting types as needed
    fn eq(&self, other: &NumericLiteral) -> bool {
        match self {
            NumericLiteral::Boolean(own_value) => match other {
                NumericLiteral::Boolean(other_value) => own_value == other_value,
                NumericLiteral::PositiveBinary(other_value) | NumericLiteral::PositiveDecimal(other_value) | NumericLiteral::PositiveHexadecimal(other_value) => *own_value as u64 == *other_value,
                NumericLiteral::Float(other_value) => match other_value.fract() == 0.0 {
                    false => false,
                    true => *own_value as i64 == *other_value as i64
                },
                // Other types store only negative values
                _ => false
            },

            NumericLiteral::PositiveBinary(own_value) | NumericLiteral::PositiveDecimal(own_value) | NumericLiteral::PositiveHexadecimal(own_value) => match other {
                NumericLiteral::Boolean(other_value) => *own_value == *other_value as u64,
                NumericLiteral::PositiveBinary(other_value) | NumericLiteral::PositiveDecimal(other_value) | NumericLiteral::PositiveHexadecimal(other_value) => *own_value == *other_value,
                NumericLiteral::Float(other_value) => match other_value.fract() == 0.0 {
                    false => false,
                    true => match *other_value < 0.0 {
                        true => false,
                        false => *own_value == *other_value as u64
                    }
                },
                // Other types store only negative values
                _ => false
            },

            NumericLiteral::NegativeBinary(own_value) | NumericLiteral::NegativeDecimal(own_value) | NumericLiteral::NegativeHexadecimal(own_value) => match other {
                NumericLiteral::NegativeBinary(other_value) | NumericLiteral::NegativeDecimal(other_value) | NumericLiteral::NegativeHexadecimal(other_value) => *own_value == *other_value,
                NumericLiteral::Float(other_value) => match other_value.fract() == 0.0 {
                    false => false,
                    true => *own_value == *other_value as i64
                },
                // Other types store only positive values
                _ => false
            },

            NumericLiteral::Float(own_value) => match other {
                NumericLiteral::Float(other_value) => *own_value == *other_value,

                // Boolean
                NumericLiteral::Boolean(other_value) => match own_value.fract() == 0.0 {
                    false => false,
                    true => *own_value as i64 == *other_value as i64
                },

                // Positives
                NumericLiteral::PositiveBinary(other_value) | NumericLiteral::PositiveDecimal(other_value) | NumericLiteral::PositiveHexadecimal(other_value) => match own_value.fract() == 0.0 {
                    false => false,
                    true => match *own_value >= 0.0 && *own_value <= u64::MAX as f64 {
                        true => *own_value as u64 == *other_value,
                        false => false
                    }
                },

                // Negatives
                NumericLiteral::NegativeBinary(other_value) | NumericLiteral::NegativeDecimal(other_value) | NumericLiteral::NegativeHexadecimal(other_value) => match own_value.fract() == 0.0 {
                    false => false,
                    true => match *own_value <= 0.0 && *own_value >= i64::MIN as f64 {
                        true => *own_value as i64 == *other_value,
                        false => false
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Bitfield,
    Comma,
    Colon,
    Comment(String),
    Define,
    Enum,
    Equals,
    Extend,
    Identifier(String),
    Include,
    LeftBrace,
    LeftBracket,
    NumericLiteral(NumericLiteral),
    NumericRange(NumericLiteral, NumericLiteral),
    Redefine,
    Reserve,
    RightBrace,
    RightBracket,
    SemiColon,
    StringLiteral(String),
    Struct,
    Verifier
}

#[derive(Debug, Clone)]
pub enum ScanningProduct {
    Skip,
    Finished,
    Token(Spanned<Token>)
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum ScanningError {
    UnexpectedCharacter(Spanned<char>),
    InvalidLiteral(Spanned<()>),
    UnexpectedEndOfFile,
    UnexpectedEndOfFileWhileParsing { token_kind: &'static str, start_position: Position }
}

type ScanningResult = Result<ScanningProduct, ScanningError>;

pub struct Scanner<I: Iterator<Item = char>> {
    input:  I,
    line:   u32,
    offset: u32,
    peeked: Option<char>
}

#[derive(Debug, Clone, PartialEq)]
// Helper for scanning numbers
enum NumberType {
    Binary,
    Decimal,
    Float,
    Hexadecimal
}

impl<I: Iterator<Item = char>> Scanner<I> {
    pub fn new(input: I) -> Self {
        Scanner {
            input,
            line: 1,
            offset: 0,
            peeked: None
        }
    }

    pub fn scan_all(mut self) -> Result<Vec<Spanned<Token>>, ScanningError> {
        let mut output = Vec::new();

        loop {
            match self.scan_token()? {
                ScanningProduct::Skip => (),
                ScanningProduct::Finished => return Ok(output),
                ScanningProduct::Token(token) => {
                    output.push(token);
                }
            }
        }
    }

    pub fn advance(&mut self) -> Option<char> {
        self.offset += 1;
        match self.peeked {
            None => self.input.next(),
            Some(c) => {
                self.peeked = None;
                Some(c)
            }
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        match self.peeked {
            Some(character) => Some(character),
            None => {
                self.peeked = self.input.next();
                self.peeked
            }
        }
    }

    pub fn keyword(&self, what: &str) -> Option<Token> {
        match what.to_owned().to_lowercase().as_str() {
            "bitfield" => Some(Token::Bitfield),
            "define" => Some(Token::Define),
            "enum" => Some(Token::Enum),
            "extend" => Some(Token::Extend),
            "false" => Some(Token::NumericLiteral(NumericLiteral::Boolean(false))),
            "include" => Some(Token::Include),
            "redefine" => Some(Token::Redefine),
            "reserve" => Some(Token::Reserve),
            "struct" => Some(Token::Struct),
            "true" => Some(Token::NumericLiteral(NumericLiteral::Boolean(true))),
            "verifier" => Some(Token::Verifier),
            _ => None
        }
    }

    pub fn position(&self) -> Position {
        Position {
            line:   self.line,
            offset: Some(self.offset)
        }
    }

    pub fn scan_identifier(&mut self) -> ScanningResult {
        let from = self.position();

        let mut identifier = String::new();

        loop {
            match self.peek() {
                Some(character) if character.is_alphanumeric() || character == '_' => identifier.push(self.advance().unwrap()),
                _ => {
                    break;
                }
            }
        }

        let to = self.position();

        Ok(match self.keyword(&identifier) {
            Some(k) => ScanningProduct::Token(Spanned::new(k, from, to)),
            None => ScanningProduct::Token(Spanned::new(Token::Identifier(identifier), from, to))
        })
    }

    pub fn scan_numerics(&mut self) -> ScanningResult {
        let starting_from = self.position();
        let mut from = starting_from;
        from.offset = from.offset.map(|v| v - 1);

        let mut text: String = String::new();

        println!("Start scan numerics. First item is: {0}", self.peek().unwrap());

        // Check for negative
        let is_negative: bool = match self.peek() {
            Some(character) => match character {
                // Consume '-' if number is negative
                '-' => {
                    self.advance();
                    true
                },
                _ => false
            },
            None => return Err(ScanningError::UnexpectedEndOfFile)
        };

        let mut number_type: NumberType = NumberType::Decimal;
        let mut scientific_notation: bool = false;

        // Seems like we have to handle ranges here...
        let mut range_detected: bool = false;

        // Parse a whole number. Ranges should be handled elsewhere.
        while self.peek().is_some() {
            match self.peek().unwrap() {
                // Binary number
                'b' | 'B' if (text.len() == 1) && (number_type == NumberType::Decimal) => {
                    number_type = NumberType::Binary;
                    self.advance().unwrap();
                },

                // Floating point number or range start
                '.' if number_type == NumberType::Decimal => {
                    // Advance, as we need to see if this is a float or a range
                    self.advance();

                    match self.peek() {
                        Some(character) => match character {
                            // Is range
                            '.' => {
                                // Consume second '.'
                                self.advance();
                                range_detected = true;
                                break;
                            },

                            // Not a range, but a float
                            _ => {
                                text.push('.');
                                number_type = NumberType::Float;
                            }
                        },
                        None => return Err(ScanningError::UnexpectedEndOfFile)
                    }
                },

                '.' => {
                    // Consume the '.'
                    self.advance();
                    if self.peek().is_some() {
                        if self.peek().unwrap() == '.' {
                            // Definitely a range here
                            range_detected = true;
                            // Consume second '.'
                            self.advance();
                            break;
                        }
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    }
                    return Err(ScanningError::UnexpectedEndOfFile);
                },

                // Scientific notation
                'e' if (number_type == NumberType::Float) || (number_type == NumberType::Decimal) => {
                    number_type = NumberType::Float;
                    scientific_notation = true;
                    text.push(self.advance().unwrap());
                },
                '-' if scientific_notation == true => {
                    text.push(self.advance().unwrap());
                },

                // Hexadecimal numbers
                'x' | 'X' if (text.len() == 1) && (number_type == NumberType::Decimal) => {
                    number_type = NumberType::Hexadecimal;
                    self.advance().unwrap();
                },

                // Rust style divider
                '_' if number_type != NumberType::Float => {
                    // Ignore divider
                    self.advance().unwrap();
                },

                character if character.is_alphanumeric() => {
                    text.push(self.advance().unwrap());
                },

                // End of number
                _ => break
            }
        }

        let number_value: NumericLiteral = match is_negative {
            true => match number_type {
                NumberType::Binary => match i64::from_str_radix(&text, 2) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::NegativeBinary(value * -1)
                },
                NumberType::Decimal => match i64::from_str_radix(&text, 10) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::NegativeDecimal(value * -1)
                },
                NumberType::Hexadecimal => match i64::from_str_radix(&text, 16) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::NegativeHexadecimal(value * -1)
                },
                NumberType::Float => match text.parse::<f64>() {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::Float(value * -1.0)
                }
            },
            false => match number_type {
                NumberType::Binary => match u64::from_str_radix(&text, 2) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::PositiveBinary(value)
                },
                NumberType::Decimal => match u64::from_str_radix(&text, 10) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::PositiveDecimal(value)
                },
                NumberType::Hexadecimal => match u64::from_str_radix(&text, 16) {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => NumericLiteral::PositiveHexadecimal(value)
                },
                NumberType::Float => match text.parse::<f64>() {
                    Err(error) => {
                        error!("Could not parse numeric value! Got error {0}", error);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    },
                    Ok(value) => return Ok(ScanningProduct::Token(Spanned::new(Token::NumericLiteral(NumericLiteral::Float(value)), from, self.position())))
                }
            }
        };

        match range_detected {
            false => Ok(ScanningProduct::Token(Spanned::new(Token::NumericLiteral(number_value), from, self.position()))),
            true => {
                let range_end: NumericLiteral = match self.scan_numerics()? {
                    ScanningProduct::Finished | ScanningProduct::Skip => return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position()))),
                    ScanningProduct::Token(token) => match token.item {
                        Token::NumericLiteral(literal) => literal,
                        _ => return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())))
                    }
                };
                Ok(ScanningProduct::Token(Spanned::new(Token::NumericRange(number_value, range_end), from, self.position())))
            }
        }
    }

    pub fn scan_string_literal(&mut self) -> ScanningResult {
        let mut from = self.position();
        // we want to include the opening quot.
        from.offset = from.offset.map(|v| v - 1);

        let mut string = String::new();

        loop {
            match self.advance().ok_or(ScanningError::UnexpectedEndOfFileWhileParsing {
                token_kind:     "string_literal",
                start_position: from
            })? {
                '"' => {
                    return Ok(ScanningProduct::Token(Spanned::new(Token::StringLiteral(string), from, self.position())));
                },
                character => string.push(character)
            }
        }
    }

    pub fn scan_token(&mut self) -> ScanningResult {
        let from = self.position();

        let character = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(ScanningProduct::Finished);
            }
        };

        let to = self.position();
        let token = |t| Ok(ScanningProduct::Token(Spanned::new(t, from, to)));

        match character {
            '/' => {
                // Advance to the '/'
                self.advance();

                // Peek the next character
                match self.peek() {
                    Some('/') => {
                        self.advance();

                        let mut comment = String::new();

                        loop {
                            match self.advance().ok_or(ScanningError::UnexpectedEndOfFileWhileParsing {
                                token_kind:     "comment",
                                start_position: from
                            })? {
                                '\n' => {
                                    let to = self.position();
                                    self.offset = 0;
                                    self.line += 1;

                                    return Ok(ScanningProduct::Token(Spanned::new(Token::Comment(comment), from, to)));
                                },
                                c => comment.push(c)
                            }
                        }
                    },
                    Some('*') => {
                        self.advance();

                        let from = self.position();
                        let mut comment = String::new();

                        loop {
                            match self.advance().ok_or(ScanningError::UnexpectedEndOfFileWhileParsing {
                                token_kind:     "comment",
                                start_position: from
                            })? {
                                '*' => {
                                    match self.peek().ok_or(ScanningError::UnexpectedEndOfFileWhileParsing {
                                        token_kind:     "comment",
                                        start_position: from
                                    })? {
                                        '/' => {
                                            self.advance();
                                            return Ok(ScanningProduct::Token(Spanned::new(Token::Comment(comment), from, self.position())));
                                        },
                                        _ => {
                                            comment.push('*');
                                            continue;
                                        }
                                    }
                                },
                                '\n' => {
                                    self.offset = 0;
                                    self.line += 1;
                                    comment.push('\n');
                                },
                                c => comment.push(c)
                            }
                        }
                    },
                    Some(c) => Err(ScanningError::UnexpectedCharacter(Spanned::new(c, self.position(), self.position()))),
                    None => Err(ScanningError::UnexpectedEndOfFile)
                }
            },

            '=' => {
                self.advance();
                token(Token::Equals)
            },

            ',' => {
                self.advance();
                token(Token::Comma)
            },

            ':' => {
                self.advance();
                token(Token::Colon)
            },
            ';' => {
                self.advance();
                token(Token::SemiColon)
            },
            '{' => {
                self.advance();
                token(Token::LeftBrace)
            },
            '}' => {
                self.advance();
                token(Token::RightBrace)
            },
            '[' => {
                self.advance();
                token(Token::LeftBracket)
            },
            ']' => {
                self.advance();
                token(Token::RightBracket)
            },
            '"' => {
                self.advance();
                self.scan_string_literal()
            },
            '\n' => {
                self.advance();
                self.line += 1;
                self.offset = 0;
                Ok(ScanningProduct::Skip)
            },

            character if character.is_numeric() || character == '-' => self.scan_numerics(),
            character if character.is_alphanumeric() || character == '_' => self.scan_identifier(),
            character if character.is_whitespace() => {
                self.advance();
                Ok(ScanningProduct::Skip)
            },
            character => {
                self.advance();
                return Err(ScanningError::UnexpectedCharacter(Spanned::new(character, from, self.position())));
            }
        }
    }
}
