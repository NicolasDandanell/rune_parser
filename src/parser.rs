use crate::types::*;

use crate::scanner::*;
use std::iter::{Iterator, Peekable};

type ItemType = Spanned<Token>;

#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(ItemType),
    UnexpectedEndOfInput,
    ScanningError(ScanningError),
}

impl From<ScanningError> for ParsingError {
    fn from(e: ScanningError) -> ParsingError {
        ParsingError::ScanningError(e)
    }
}

type ParsingResult<T> = Result<T, ParsingError>;

pub trait TokenSource: std::clone::Clone {
    fn next(&mut self) -> Option<ItemType>;
    fn peek(&mut self) -> Option<&ItemType>;

    fn expect_next(&mut self) -> ParsingResult<ItemType> {
        match self.next() {
            None => Err(ParsingError::UnexpectedEndOfInput),
            Some(t) => Ok(t),
        }
    }

    fn expect_token(&mut self, token: Token) -> ParsingResult<ItemType> {
        match self.expect_next()? {
            t if *t == token => Ok(t),
            t => Err(ParsingError::UnexpectedToken(t)),
        }
    }

    fn expect_identifier(&mut self) -> ParsingResult<Spanned<String>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Identifier(s) => Ok(Spanned::new(s, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }

    fn expect_string_literal(&mut self) -> ParsingResult<Spanned<String>> {
        let token = self.expect_next()?;
        match token.item {
            Token::StringLiteral(s) => Ok(Spanned::new(s, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }

    fn expect_type(&mut self) -> ParsingResult<Spanned<FieldType>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Identifier(s) => Ok(Spanned::new(
                match s.as_str() {
                    "bool" => FieldType::Boolean,
                    "u8" => FieldType::UByte,
                    "i8" => FieldType::Byte,
                    "u16" => FieldType::UShort,
                    "i16" => FieldType::Short,
                    "u32" => FieldType::UInt,
                    "i32" => FieldType::Int,
                    "u64" => FieldType::ULong,
                    "i64" => FieldType::Long,
                    "f32" => FieldType::Float,
                    "f64" => FieldType::Double,
                    _ => FieldType::UserDefined(s),
                },
                token.from,
                token.to,
            )),
            Token::LeftBracket => {
                let inner_type = self.expect_type()?;
                self.expect_token(Token::SemiColon)?;
                let count_token = self.expect_next()?;
                let count = match &count_token.item {
                    // Simple integer value will generate a simple number
                    Token::IntegerLiteral(i) => ArraySize::NumericValue(*i as usize),
                    // String will generate a user definition, which will be populated with a value in post processing
                    Token::Identifier(s)  => ArraySize::UserDefinition(DefineDefinition { identifier: s.clone(), value: DefineValue::NoValue, comment: None } ),
                    _ => return Err(ParsingError::UnexpectedToken(count_token)),
                };

                let rb = self.expect_token(Token::RightBracket)?;

                Ok(Spanned::new(
                    FieldType::Array(Box::new(inner_type.item), count),
                    token.from,
                    rb.to,
                ))
            }
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }

    fn maybe_expect(&mut self, token: Token) -> Option<ItemType> {
        match TokenSource::peek(self)? {
            t if t.item == token => Some(self.expect_next().unwrap()),
            _ => None,
        }
    }

    fn maybe_expect_comment(&mut self) -> Option<Spanned<String>> {
        if let Spanned {
            from: _,
            to: _,
            item: Token::Comment(_),
        } = TokenSource::peek(self)?
        {
            let Spanned {
                from,
                to,
                item: Token::Comment(s),
            } = self.expect_next().unwrap()
            else {
                unreachable!()
            };
            return Some(Spanned::new(s, from, to));
        }

        None
    }
}

impl<T> TokenSource for Peekable<T>
where
    T: Iterator<Item = ItemType> + Clone,
{
    fn next(&mut self) -> Option<ItemType> {
        std::iter::Iterator::next(self)
    }

    fn peek(&mut self) -> Option<&ItemType> {
        self.peek()
    }
}

pub fn parse_tokens(tokens: &mut impl TokenSource) -> ParsingResult<Definitions> {
    let mut definitions = Definitions::new();
    let mut last_comment: Option<String> = None;

    'parsing: loop {
        let token = tokens.peek();
        if token.is_none() {
            break 'parsing;
        }
        let token = token.unwrap();

        match &token.item {
            Token::Bitfield => {
                // Get comment if any
                let comment = last_comment.take();

                // Type and identifier
                tokens.expect_token(Token::Bitfield)?;
                let ident = tokens.expect_identifier()?;

                // Backing type
                tokens.expect_token(Token::Colon)?;
                let backing_type = tokens.expect_type()?;

                // Get member fields
                tokens.expect_token(Token::LeftBrace)?;
                let mut members = Vec::new();

                loop {
                    // Comment if any
                    let comment = tokens.maybe_expect_comment();

                    // Identifier
                    let field_ident = tokens.expect_identifier()?;

                    // Bit size
                    tokens.expect_token(Token::Colon)?;
                    let bit_size_token = tokens.expect_next()?;
                    let bit_size = match bit_size_token.item {
                        Token::IntegerLiteral(i) => i as usize,
                        _ => return Err(ParsingError::UnexpectedToken(bit_size_token))
                    };

                    // Bit field slot
                    tokens.expect_token(Token::Equals)?;
                    let field_slot_token = tokens.expect_next()?;
                    let field_slot = match field_slot_token.item {
                        Token::IntegerLiteral(i) => i as usize,
                        _ => return Err(ParsingError::UnexpectedToken(field_slot_token))
                    };

                    members.push(BitfieldMember {
                        ident: field_ident.item.clone(),
                        bit_size: bit_size,
                        bit_slot: field_slot,
                        comment: comment.map(|s| s.item)
                    });

                    if tokens.maybe_expect(Token::SemiColon).is_none() {
                        tokens.expect_token(Token::RightBrace)?;
                        break;
                    }
                    if tokens.maybe_expect(Token::RightBrace).is_some() {
                        break;
                    }
                }

                definitions.bitfields.push(BitfieldDefinition {
                    name:         ident.item.clone(),
                    backing_type: backing_type.item,
                    members:      members,
                    comment:      comment
                })
            },

            Token::Struct => {
                let comment = last_comment.take();
                tokens.expect_token(Token::Struct)?;
                let ident = tokens.expect_identifier()?;

                tokens.expect_token(Token::LeftBrace)?;

                let mut members = Vec::new();
                loop {
                    let comment = tokens.maybe_expect_comment();
                    let field_ident = tokens.expect_identifier()?;
                    tokens.expect_token(Token::Colon)?;
                    let tk = tokens.expect_type()?;

                    tokens.expect_token(Token::Equals)?;

                    let field_slot_token = tokens.expect_next()?;
                    let field_slot: FieldSlot = match &field_slot_token.item {
                        Token::IntegerLiteral(i) => {
                            // Check if value is positive and within the legal values (0 to and including 31)
                            match *i {
                                // Legal values
                                0..31 => FieldSlot::NamedSlot(*i as usize),
                                // Higher than legal values
                                31..  => panic!("Field index cannot have a value higher than 30!"),
                                // Negative values
                                ..0   => panic!("Field indexes cannot have negative values!")
                            }
                        },
                        Token::Identifier(s) if s == "VerificationField" => {
                            FieldSlot::VerificationField
                        }
                        _ => return Err(ParsingError::UnexpectedToken(field_slot_token)),
                    };

                    members.push(StructMember {
                        ident: field_ident.item.clone(),
                        field_type: tk.item.clone(),
                        field_slot,
                        comment: comment.map(|s| s.item),
                        user_definition_link: UserDefinitionLink::NoLink
                    });

                    if tokens.maybe_expect(Token::SemiColon).is_none() {
                        tokens.expect_token(Token::RightBrace)?;
                        break;
                    }
                    if tokens.maybe_expect(Token::RightBrace).is_some() {
                        break;
                    }
                }

                definitions.structs.push(StructDefinition {
                    name: ident.item,
                    members,
                    comment,
                })
            }

            Token::Enum => {
                let comment = last_comment.take();
                tokens.expect_token(Token::Enum)?;
                let ident = tokens.expect_identifier()?;

                tokens.expect_token(Token::Colon)?;

                let backing_type = tokens.expect_type()?;

                tokens.expect_token(Token::LeftBrace)?;

                let mut members = Vec::new();
                loop {
                    let comment = tokens.maybe_expect_comment();
                    let field_ident = tokens.expect_identifier()?;

                    tokens.expect_token(Token::Equals)?;

                    let enum_value_token = tokens.expect_next()?;
                    let enum_value = match &enum_value_token.item {
                        Token::IntegerLiteral(i) => EnumValue::IntegerLiteral(*i),
                        Token::FloatLiteral(f)   => EnumValue::FloatLiteral(*f),
                        _ => return Err(ParsingError::UnexpectedToken(enum_value_token)),
                    };

                    members.push(EnumMember {
                        ident: field_ident.item.clone(),
                        value: enum_value,

                        comment: comment.map(|s| s.item),
                    });

                    if tokens.maybe_expect(Token::SemiColon).is_none() {
                        tokens.expect_token(Token::RightBrace)?;
                        break;
                    }
                    if tokens.maybe_expect(Token::RightBrace).is_some() {
                        break;
                    }
                }

                definitions.enums.push(EnumDefinition {
                    name: ident.item,
                    backing_type: backing_type.item,
                    members,
                    comment,
                })
            }

            Token::Comment(s) => {
                last_comment = Some(s.clone());
                tokens.expect_next()?;
            }

            Token::Include => {
                tokens.expect_next()?;
                let string: String = tokens.expect_string_literal()?.item.strip_suffix(".rune").expect("File included was now a .rune file").to_string();
                tokens.expect_token(Token::SemiColon)?;
                definitions.includes.push(
                    IncludeDefinition {
                        file: string
                    }
                );
            }

            Token::Define => {
                let comment = last_comment.take();

                tokens.expect_next()?;

                // Get definition name
                let definition_name = tokens.expect_identifier()?;

                let define_value_token = tokens.expect_next()?;
                let define_value: DefineValue = match &define_value_token.item {
                    Token::IntegerLiteral(i) => DefineValue::IntegerLiteral(*i),
                    Token::FloatLiteral(f)   => DefineValue::FloatLiteral(*f),
                    _ => return Err(ParsingError::UnexpectedToken(define_value_token)),
                };

                tokens.expect_token(Token::SemiColon)?;

                // Save, as implementing Composite value will require more debugging
                /* match define_value {
                    DefineValue::IntegerLiteral(integer) => {
                        println!("Got definition with identifier \"{0}\" and integer value \"{1}\"", definition_name.item, integer)
                    },
                    DefineValue::FloatLiteral(float)     => {
                        println!("Got definition with identifier \"{0}\" and float value \"{1}\"", definition_name.item, float)
                    },
                    _ => panic!("Composite define values not implemented yet!")
                }; */

                definitions.defines.push(
                    DefineDefinition {
                        identifier: definition_name.item,
                        value:      define_value,
                        comment:    comment
                    }
                );
            }
            _ => return Err(ParsingError::UnexpectedToken(token.clone())),
        }
    }

    Ok(definitions)
}
