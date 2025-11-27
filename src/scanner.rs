use core::fmt;
use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut}
};

use crate::{output::*, types::Primitive};

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

#[derive(Debug, Clone, Copy)]
pub enum NumeralSystem {
    Binary,
    Decimal,
    Hexadecimal
}

#[derive(Debug, Clone)]
pub enum NumericLiteral {
    AsciiChar(char),
    Boolean(bool),
    PositiveInteger(u64, NumeralSystem),
    NegativeInteger(i64, NumeralSystem),
    Float(f64)
}

impl NumericLiteral {
    pub fn containing_type(&self) -> Primitive {
        match self {
            NumericLiteral::AsciiChar(_) => Primitive::Char,
            NumericLiteral::Boolean(_) => Primitive::Bool,
            NumericLiteral::PositiveInteger(_, _) => Primitive::U64,
            NumericLiteral::NegativeInteger(_, _) => Primitive::I64,
            NumericLiteral::Float(_) => Primitive::F64
        }
    }
}

impl Display for NumericLiteral {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NumericLiteral::AsciiChar(character) => match character {
                character if character.is_alphanumeric() => write!(formatter, "{0:?}", character),
                character => write!(formatter, "{0}", {
                    let output_string = character.escape_default().to_string().replace(['{', '}'], "");

                    match output_string.contains("u") {
                        true => format!("0x{0:02X}", *character as u8),
                        false => format!("\'{0}\'", output_string)
                    }
                })
            },

            NumericLiteral::Boolean(boolean) => write!(formatter, "{0}", boolean),
            NumericLiteral::Float(float) => write!(formatter, "{0}", float),

            NumericLiteral::PositiveInteger(value, numeral_system) => match numeral_system {
                NumeralSystem::Binary => write!(formatter, "0b{0:02b}", value),
                NumeralSystem::Decimal => write!(formatter, "{0}", value),
                NumeralSystem::Hexadecimal => write!(formatter, "0x{0:02X}", value)
            },

            NumericLiteral::NegativeInteger(value, numeral_system) => match numeral_system {
                NumeralSystem::Binary => write!(formatter, "-0b{0:02b}", value.abs()),
                NumeralSystem::Decimal => write!(formatter, "{0}", value),
                NumeralSystem::Hexadecimal => write!(formatter, "-0x{0:02X}", value.abs())
            }
        }
    }
}

impl PartialEq for NumericLiteral {
    /// Evaluate the raw numeric value of the literals, casting types as needed
    fn eq(&self, other: &NumericLiteral) -> bool {
        match self {
            NumericLiteral::AsciiChar(own_value) => match other {
                NumericLiteral::AsciiChar(other_value) => own_value == other_value,
                NumericLiteral::Boolean(other_value) => *own_value as u8 == *other_value as u8,
                NumericLiteral::PositiveInteger(other_value, _) if *other_value <= u8::MAX as u64 => *own_value as u8 == *other_value as u8,
                NumericLiteral::Float(other_value) if other_value.fract() == 0.0 && *other_value >= 0.0 => *own_value as u8 == *other_value as u8,
                // Remaining values cannot be used for comparison
                _ => false
            },

            NumericLiteral::Boolean(own_value) => match other {
                NumericLiteral::AsciiChar(other_value) => *own_value as u8 == *other_value as u8,
                NumericLiteral::Boolean(other_value) => own_value == other_value,
                NumericLiteral::PositiveInteger(other_value, _) => *own_value as u64 == *other_value,
                NumericLiteral::Float(other_value) if other_value.fract() == 0.0 && *other_value >= 0.0 => *own_value as u64 == *other_value as u64,
                // Remaining values cannot be used for comparison
                _ => false
            },

            NumericLiteral::PositiveInteger(own_value, _) => match other {
                NumericLiteral::AsciiChar(other_value) if *own_value <= u8::MAX as u64 => *own_value as u8 == *other_value as u8,
                NumericLiteral::Boolean(other_value) => *own_value == *other_value as u64,
                NumericLiteral::PositiveInteger(other_value, _) => *own_value == *other_value,
                NumericLiteral::Float(other_value) if other_value.fract() == 0.0 && *other_value >= 0.0 && *other_value <= u64::MAX as f64 => *own_value == *other_value as u64,
                // Remaining values cannot be used for comparison
                _ => false
            },

            NumericLiteral::NegativeInteger(own_value, _) => match other {
                NumericLiteral::NegativeInteger(other_value, _) => *own_value == *other_value,
                NumericLiteral::Float(other_value) if other_value.fract() == 0.0 && *other_value <= 0.0 && *other_value >= i64::MIN as f64 => *own_value == *other_value as i64,
                // Remaining values cannot be used for comparison
                _ => false
            },

            NumericLiteral::Float(own_value) => match other {
                NumericLiteral::AsciiChar(other_value) if own_value.fract() == 0.0 && *own_value >= 0.0 && *own_value <= u8::MAX as f64 => *own_value as u8 == *other_value as u8,
                NumericLiteral::Float(other_value) => *own_value == *other_value,
                NumericLiteral::Boolean(other_value) if own_value.fract() == 0.0 && *own_value >= 0.0 && *own_value <= u64::MAX as f64 => *own_value as u64 == *other_value as u64,
                NumericLiteral::PositiveInteger(other_value, _) if own_value.fract() == 0.0 && *own_value >= 0.0 && *own_value <= u64::MAX as f64 => *own_value as u64 == *other_value,
                NumericLiteral::NegativeInteger(other_value, _) if own_value.fract() == 0.0 && *own_value <= 0.0 && *own_value >= i64::MIN as f64 => *own_value as i64 == *other_value,
                // Remaining values cannot be used for comparison
                _ => false
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
    Message,
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

pub struct Scanner<ScannerIterator: Iterator<Item = char>> {
    input:  ScannerIterator,
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

impl<ScannerIterator: Iterator<Item = char>> Scanner<ScannerIterator> {
    pub fn new(input: ScannerIterator) -> Self {
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
            "deprecate" /* Alias for reserve */ => Some(Token::Reserve),
            "enum" => Some(Token::Enum),
            "extend" => Some(Token::Extend),
            "false" => Some(Token::NumericLiteral(NumericLiteral::Boolean(false))),
            "include" => Some(Token::Include),
            "message" => Some(Token::Message),
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

    pub fn extract_number(string: &mut String, from: Position, to: Position) -> Result<NumericLiteral, ScanningError> {
        if string.is_empty() {
            error!("Tried parsing an empty literal numeric value!");
            return Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)));
        }

        // Get whether number is negative
        let is_negative: bool = string.chars().nth(0).unwrap() == '-';

        // Get number type
        let number_type: NumberType = match string {
            // Float - First, as hexadecimal floats are a thing apparently...
            _ if string.contains('.') => NumberType::Float,

            // Binary
            _ if string.contains("0b") => {
                let index = string.find("0b").unwrap();
                if !(string.remove(index) == '0' && string.remove(index) == 'b') {
                    error!("Something went wrong in parsing binary literal!");
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)));
                }

                NumberType::Binary
            },
            _ if string.contains("0B") => {
                let index = string.find("0B").unwrap();
                if !(string.remove(index) == '0' && string.remove(index) == 'B') {
                    error!("Something went wrong in parsing binary literal!");
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)));
                }

                NumberType::Binary
            },

            // Hexadecimal
            _ if string.contains("0x") => {
                let index = string.find("0x").unwrap();
                if !(string.remove(index) == '0' && string.remove(index) == 'x') {
                    error!("Something went wrong in parsing hexadecimal literal!");
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)));
                }

                NumberType::Hexadecimal
            },
            _ if string.contains("0X") => {
                let index = string.find("0X").unwrap();
                if !(string.remove(index) == '0' && string.remove(index) == 'X') {
                    error!("Something went wrong in parsing hexadecimal literal!");
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)));
                }

                NumberType::Hexadecimal
            },
            _ => NumberType::Decimal
        };

        match number_type {
            NumberType::Float => match string.parse::<f64>() {
                Err(error) => {
                    error!("Could not parse numeric value! Got error {0}", error);
                    Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                },
                Ok(value) => Ok(NumericLiteral::Float(value))
            },

            NumberType::Binary => {
                let numeral_system: NumeralSystem = NumeralSystem::Binary;

                match is_negative {
                    true => match i64::from_str_radix(string, 2) {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::NegativeInteger(value, numeral_system))
                    },
                    false => match u64::from_str_radix(string, 2) {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::PositiveInteger(value, numeral_system))
                    }
                }
            },
            NumberType::Decimal => {
                let numeral_system: NumeralSystem = NumeralSystem::Decimal;

                match is_negative {
                    true => match string.parse::<i64>() {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::NegativeInteger(value, numeral_system))
                    },
                    false => match string.parse::<u64>() {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::PositiveInteger(value, numeral_system))
                    }
                }
            },
            NumberType::Hexadecimal => {
                let numeral_system: NumeralSystem = NumeralSystem::Hexadecimal;

                match is_negative {
                    true => match i64::from_str_radix(string, 16) {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::NegativeInteger(value, numeral_system))
                    },
                    false => match u64::from_str_radix(string, 16) {
                        Err(error) => {
                            error!("Could not parse numeric value! Got error {0}", error);
                            Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                        },
                        Ok(value) => Ok(NumericLiteral::PositiveInteger(value, numeral_system))
                    }
                }
            }
        }
    }

    pub fn scan_char(&mut self) -> ScanningResult {
        let starting_from = self.position();
        let mut from = starting_from;
        from.offset = from.offset.map(|v| v - 1);

        // Advance past the ' that caused this function to be called
        self.advance();

        let mut text: String = String::new();

        let mut had_escape: bool = false;
        let mut previous_was_escape: bool = false;

        let mut index: usize = 0;

        while self.peek().is_some() {
            match self.peek().unwrap() {
                // Check for escape sequence, and set the boolean accordingly
                '\\' if !previous_was_escape => {
                    if index != 0 {
                        error!("Invalid mid-character escape sequence in character literal declaration");
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    }

                    had_escape = true;
                    previous_was_escape = true;
                    text.push(self.advance().unwrap());
                },

                // If previous was not escape, and we encounter a ', then the char definition is over
                '\'' if !previous_was_escape => {
                    self.advance();
                    break;
                },

                // Add ascii characters normally
                character if character.is_ascii() && (index == 0 || had_escape) => {
                    previous_was_escape = false;
                    text.push(self.advance().unwrap())
                },

                // This case will trigger if multiple valid ascii characters are contained in the declaration, such as 'ab'
                character if character.is_ascii() => {
                    text.push(self.advance().unwrap());
                    error!("Multiple ascii characters {0} found in character literal declaration", text);
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                },

                // Any non-ascii characters should cause the parsing to return an error
                value => {
                    error!("Unexpected character {0} found in character literal declaration", value);
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                }
            }
            index += 1;
        }

        // Parse text string into a valid ascii value
        // ———————————————————————————————————————————

        let resulting_char: char = match text.len() {
            0 => {
                error!("Empty character found in character literal declaration");
                return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
            },
            1 => text.chars().nth(0).unwrap(),
            // We already checked that the escape character was the first one when > 1 characters
            2 => match text.chars().nth(1).unwrap() {
                character if character.is_numeric() => u8::from_str_radix(&character.to_string(), 10).unwrap() as char,
                'a' => 0x07 as char,
                'b' => 0x08 as char,
                'e' => 0x1B as char,
                'f' => 0x0C as char,
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => 0x0B as char,
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                _ => {
                    error!("Invalid escape sequence {0} found", text);
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                }
            },
            3.. => match text.chars().nth(1).unwrap() {
                character if character.is_numeric() => match u8::from_str_radix(&text[1..], 10) {
                    Ok(value) => value as char,
                    Err(_) => {
                        error!("Invalid escape sequence {0} found", text);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    }
                },
                'u' => match u8::from_str_radix(&text[2..], 10) {
                    Ok(value) => value as char,
                    Err(_) => {
                        error!("Invalid escape sequence {0} found", text);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    }
                },
                'x' => match u8::from_str_radix(&text[2..], 16) {
                    Ok(value) => value as char,
                    Err(_) => {
                        error!("Invalid escape sequence {0} found", text);
                        return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                    }
                },
                _ => {
                    error!("Invalid escape sequence {0} found", text);
                    return Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())));
                }
            }
        };

        let character: NumericLiteral = NumericLiteral::AsciiChar(resulting_char);

        Ok(ScanningProduct::Token(Spanned::new(Token::NumericLiteral(character), from, self.position())))
    }

    pub fn scan_numerics(&mut self) -> ScanningResult {
        let starting_from = self.position();
        let mut from = starting_from;
        from.offset = from.offset.map(|v| v - 1);

        let mut text: String = String::new();

        // Parse a whole number. Ranges should be handled elsewhere.
        while self.peek().is_some() {
            match self.peek().unwrap() {
                '-' | '_' | '.' | ' ' => text.push(self.advance().unwrap()),
                character if character.is_alphanumeric() => text.push(self.advance().unwrap()),

                // End of number
                _ => break
            }
        }

        // Check range
        match text.contains("..") {
            true => {
                let strings: Vec<&str> = text.split("..").collect();
                match strings.len() {
                    2 => {
                        let start: NumericLiteral = Self::extract_number(&mut String::from(strings[0].trim()), from, self.position())?;
                        let end: NumericLiteral = Self::extract_number(&mut String::from(strings[1].trim()), from, self.position())?;
                        Ok(ScanningProduct::Token(Spanned::new(Token::NumericRange(start, end), from, self.position())))
                    },
                    _ => {
                        error!("Invalid range declaration");
                        Err(ScanningError::InvalidLiteral(Spanned::new((), from, self.position())))
                    }
                }
            },
            false => {
                let number: NumericLiteral = Self::extract_number(&mut String::from(text.trim()), from, self.position())?;
                Ok(ScanningProduct::Token(Spanned::new(Token::NumericLiteral(number), from, self.position())))
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
            '\'' => self.scan_char(),

            character if character.is_numeric() || character == '-' => self.scan_numerics(),
            character if character.is_alphanumeric() || character == '_' => self.scan_identifier(),
            character if character.is_whitespace() => {
                self.advance();
                Ok(ScanningProduct::Skip)
            },
            character => {
                self.advance();
                Err(ScanningError::UnexpectedCharacter(Spanned::new(character, from, self.position())))
            }
        }
    }
}
