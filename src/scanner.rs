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
        Spanned {
            item,
            from: s1.from,
            to: s2.to
        }
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

use std::ops::{Deref, DerefMut};

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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Bitfield,
    Colon,
    Comment(String),
    DecimalLiteral(i64),
    Define,
    Enum,
    Equals,
    Extend,
    FloatLiteral(f64),
    Identifier(String),
    HexLiteral(u64),
    Include,
    LeftBrace,
    LeftBracket,
    Redefine,
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
    UnexpectedEndOfFileWhileParsing {
        token_kind:     &'static str,
        start_position: Position
    }
}

type ScanningResult = Result<ScanningProduct, ScanningError>;

pub struct Scanner<I: Iterator<Item = char>> {
    input:  I,
    line:   u32,
    offset: u32,
    peeked: Option<char>
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
            Some(c) => Some(c),
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
            "include" => Some(Token::Include),
            "redefine" => Some(Token::Redefine),
            "struct" => Some(Token::Struct),
            _ => None
        }
    }

    pub fn position(&self) -> Position {
        Position {
            line:   self.line,
            offset: Some(self.offset)
        }
    }

    pub fn scan_token(&mut self) -> ScanningResult {
        let from = self.position();
        let c = match self.advance() {
            Some(c) => c,
            None => {
                return Ok(ScanningProduct::Finished);
            }
        };
        let peeked = self.peek();

        let to = self.position();
        let tok = |t| Ok(ScanningProduct::Token(Spanned::new(t, from, to)));

        match c {
            '/' => match peeked {
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
                                        return Ok(ScanningProduct::Token(Spanned::new(
                                            Token::Comment(comment),
                                            from,
                                            self.position()
                                        )));
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
                Some(c) => Err(ScanningError::UnexpectedCharacter(Spanned::new(
                    c,
                    self.position(),
                    self.position()
                ))),
                None => Err(ScanningError::UnexpectedEndOfFile)
            },
            '=' => tok(Token::Equals),
            ':' => tok(Token::Colon),
            ';' => tok(Token::SemiColon),
            '{' => tok(Token::LeftBrace),
            '}' => tok(Token::RightBrace),
            '[' => tok(Token::LeftBracket),
            ']' => tok(Token::RightBracket),
            '\n' => {
                self.line += 1;
                self.offset = 0;
                Ok(ScanningProduct::Skip)
            },
            '"' => self.scan_string_literal(),
            c if c.is_whitespace() => Ok(ScanningProduct::Skip),
            c if c.is_numeric() => self.scan_numerics(c),
            c if c.is_alphanumeric() || c == '_' => self.scan_identifier(c),
            c => return Err(ScanningError::UnexpectedCharacter(Spanned::new(c, from, self.position())))
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
                    return Ok(ScanningProduct::Token(Spanned::new(
                        Token::StringLiteral(string),
                        from,
                        self.position()
                    )));
                },
                c => string.push(c)
            }
        }
    }

    pub fn scan_identifier(&mut self, begin: char) -> ScanningResult {
        let mut from = self.position();
        from.offset = from.offset.map(|v| v - 1);

        let mut ident = String::new();
        ident.push(begin);

        loop {
            match self.peek() {
                Some(c) if c.is_alphanumeric() || c == '_' => ident.push(self.advance().unwrap()),
                _ => {
                    break;
                }
            }
        }

        let to = self.position();

        Ok(match self.keyword(&ident) {
            Some(k) => ScanningProduct::Token(Spanned::new(k, from, to)),
            None => ScanningProduct::Token(Spanned::new(Token::Identifier(ident), from, to))
        })
    }

    pub fn scan_numerics(&mut self, begin: char) -> ScanningResult {
        let mut from = self.position();
        from.offset = from.offset.map(|v| v - 1);

        let mut text = String::new();
        text.push(begin);

        while self.peek().unwrap().is_numeric() {
            text.push(self.advance().unwrap());
        }

        match self.peek().unwrap() {
            '.' => {
                text.push(self.advance().unwrap());
                while self.peek().unwrap().is_numeric() {
                    text.push(self.advance().unwrap());
                }
                let to = self.position();

                match text.parse::<f64>() {
                    Ok(f) => Ok(ScanningProduct::Token(Spanned::new(Token::FloatLiteral(f), from, to))),
                    Err(_) => Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                }
            },

            'x' | 'X' => {
                text.push(self.advance().unwrap());
                while self.peek().unwrap().is_alphanumeric() {
                    text.push(self.advance().unwrap());
                }
                let to = self.position();

                match u64::from_str_radix(&text.strip_prefix("0x").unwrap(), 16) {
                    Ok(hex) => Ok(ScanningProduct::Token(Spanned::new(Token::HexLiteral(hex), from, to))),
                    Err(_) => Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                }
            },

            _ => {
                let to = self.position();
                match i64::from_str_radix(&text, 10) {
                    Ok(i) => Ok(ScanningProduct::Token(Spanned::new(Token::DecimalLiteral(i), from, to))),
                    Err(_) => Err(ScanningError::InvalidLiteral(Spanned::new((), from, to)))
                }
            }
        }
    }
}
