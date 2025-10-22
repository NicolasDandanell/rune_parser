use std::iter::{Iterator, Peekable};

use crate::{output::*, scanner::*, types::*};

type ItemType = Spanned<Token>;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(ItemType),
    UnexpectedEndOfInput,
    ScanningError(ScanningError),
    InvalidIndex(isize),
    LogicError
}

impl From<ScanningError> for ParsingError {
    fn from(error: ScanningError) -> ParsingError {
        ParsingError::ScanningError(error)
    }
}

type ParsingResult<T> = Result<T, ParsingError>;

pub trait TokenSource: std::clone::Clone {
    fn next(&mut self) -> Option<ItemType>;
    fn peek(&mut self) -> Option<&ItemType>;

    fn expect_next(&mut self) -> ParsingResult<ItemType> {
        match self.next() {
            None => Err(ParsingError::UnexpectedEndOfInput),
            Some(token) => Ok(token)
        }
    }

    fn expect_token(&mut self, expected_token: Token) -> ParsingResult<ItemType> {
        match self.expect_next()? {
            token if *token == expected_token => Ok(token),
            token => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_identifier(&mut self) -> ParsingResult<Spanned<String>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Identifier(string) => Ok(Spanned::new(string, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_string_literal(&mut self) -> ParsingResult<Spanned<String>> {
        let token = self.expect_next()?;
        match token.item {
            Token::StringLiteral(string) => Ok(Spanned::new(string, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_bitfield_size(&mut self) -> ParsingResult<Spanned<BitSize>> {
        let token = self.expect_next()?;

        match &token.item {
            // Parse identifier, with char first, then convert rest to decimal number
            Token::Identifier(string) => {
                let signed: bool = match string.chars().nth(0).unwrap() {
                    'u' | 'U' => false,
                    'i' | 'I' => true,
                    _ => return Err(ParsingError::UnexpectedToken(token))
                };

                let size: usize = match string[1..].parse() {
                    Err(_) => return Err(ParsingError::UnexpectedToken(token)),
                    Ok(number) => number
                };

                let bitfield_size: BitSize = match signed {
                    false => BitSize::Unsigned(size),
                    true => BitSize::Signed(size)
                };

                Ok(Spanned::new(bitfield_size, token.from, token.to))
            },
            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_type(&mut self) -> ParsingResult<Spanned<FieldType>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Identifier(string) => Ok(Spanned::new(
                match string.as_str() {
                    "bool" => FieldType::Boolean,
                    "u8" => FieldType::UByte,
                    "i8" => FieldType::Byte,
                    "char" => FieldType::Char,
                    "u16" => FieldType::UShort,
                    "i16" => FieldType::Short,
                    "u32" => FieldType::UInt,
                    "i32" => FieldType::Int,
                    "u64" => FieldType::ULong,
                    "i64" => FieldType::Long,
                    "f32" => FieldType::Float,
                    "f64" => FieldType::Double,
                    _ => FieldType::UserDefined(string)
                },
                token.from,
                token.to
            )),

            Token::LeftBracket => {
                let inner_type = self.expect_type()?;
                self.expect_token(Token::SemiColon)?;
                let count_token = self.expect_next()?;
                let count = match &count_token.item {
                    // Simple integer or hex value will generate a simple number
                    Token::NumericLiteral(value) => match value {
                        NumericLiteral::Decimal(decimal) => ArraySize::DecimalValue(*decimal as usize),
                        NumericLiteral::Hexadecimal(hexadecimal) => ArraySize::HexValue(*hexadecimal as usize),
                        _ => return Err(ParsingError::UnexpectedToken(count_token))
                    },

                    // String will generate a user definition, which will be populated with a value in post processing
                    Token::Identifier(string) => ArraySize::UserDefinition(DefineDefinition {
                        identifier:   string.clone(),
                        value:        DefineValue::NoValue,
                        comment:      None,
                        redefinition: None
                    }),
                    _ => return Err(ParsingError::UnexpectedToken(count_token))
                };

                let right_bracket = self.expect_token(Token::RightBracket)?;

                Ok(Spanned::new(FieldType::Array(Box::new(inner_type.item), count), token.from, right_bracket.to))
            },

            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn maybe_expect(&mut self, expected_token: Token) -> Option<ItemType> {
        match TokenSource::peek(self)? {
            token if token.item == expected_token => Some(self.expect_next().unwrap()),
            _ => None
        }
    }

    fn maybe_expect_comment(&mut self) -> Option<Spanned<String>> {
        if let Spanned {
            from: _,
            to: _,
            item: Token::Comment(_)
        } = TokenSource::peek(self)?
        {
            let Spanned {
                from,
                to,
                item: Token::Comment(string)
            } = self.expect_next().unwrap()
            else {
                unreachable!()
            };
            return Some(Spanned::new(string, from, to));
        }

        None
    }
}

impl<T> TokenSource for Peekable<T>
where
    T: Iterator<Item = ItemType> + Clone
{
    fn next(&mut self) -> Option<ItemType> {
        std::iter::Iterator::next(self)
    }

    fn peek(&mut self) -> Option<&ItemType> {
        self.peek()
    }
}

fn parse_bitfield(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<BitfieldDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Type and identifier
    tokens.expect_token(Token::Bitfield)?;
    let identifier = tokens.expect_identifier()?;

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
        let field_identifier = tokens.expect_identifier()?;

        // Bit size
        tokens.expect_token(Token::Colon)?;
        let bit_size_token: Spanned<BitSize> = tokens.expect_bitfield_size()?;
        let bit_size: BitSize = bit_size_token.item;

        // Bit field slot
        tokens.expect_token(Token::Equals)?;
        let field_slot_token = tokens.expect_next()?;

        let field_slot = match &field_slot_token.item {
            Token::NumericLiteral(value) => match value {
                NumericLiteral::Decimal(decimal) => match decimal {
                    ..0 => {
                        error!("Bit slot cannot have negative number {0}", decimal);
                        return Err(ParsingError::InvalidIndex(*decimal));
                    },
                    0.. => *decimal as usize
                },
                NumericLiteral::Hexadecimal(hexadecimal) => *hexadecimal as usize,
                _ => return Err(ParsingError::UnexpectedToken(field_slot_token))
            },
            _ => return Err(ParsingError::UnexpectedToken(field_slot_token))
        };

        members.push(BitfieldMember {
            identifier: field_identifier.item.clone(),
            bit_size:   bit_size,
            bit_slot:   field_slot,
            comment:    comment.map(|s| s.item)
        });

        if tokens.maybe_expect(Token::SemiColon).is_none() {
            tokens.expect_token(Token::RightBrace)?;
            break;
        }
        if tokens.maybe_expect(Token::RightBrace).is_some() {
            break;
        }
    }

    return Ok(BitfieldDefinition {
        name:            identifier.item.clone(),
        backing_type:    backing_type.item,
        members:         members,
        comment:         comment,
        orphan_comments: Vec::new()
    });
}

fn parse_define(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<DefineDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get define token
    tokens.expect_next()?;

    // Get definition name
    let definition_name = tokens.expect_identifier()?;

    let define_value_token = tokens.expect_next()?;
    let define_value: DefineValue = match define_value_token.item {
        Token::NumericLiteral(value) => DefineValue::NumericLiteral(value),
        _ => return Err(ParsingError::UnexpectedToken(define_value_token))
    };

    tokens.expect_token(Token::SemiColon)?;

    // Save, as implementing Composite value will require more debugging
    /* match define_value {
        DefineValue::IntegerLiteral(integer) => {
            info!("Got definition with identifier \"{0}\" and integer value \"{1}\"", definition_name.item, integer)
        },
        DefineValue::FloatLiteral(float)     => {
            info!("Got definition with identifier \"{0}\" and float value \"{1}\"", definition_name.item, float)
        },
        _ => error!("Composite define values not implemented yet!")
    }; */

    Ok(DefineDefinition {
        identifier:   definition_name.item,
        value:        define_value,
        comment:      comment,
        redefinition: None
    })
}

fn parse_enum(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<EnumDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get enum token
    tokens.expect_token(Token::Enum)?;

    // Get identifier
    let identifier = tokens.expect_identifier()?;

    tokens.expect_token(Token::Colon)?;

    let backing_type = tokens.expect_type()?;

    tokens.expect_token(Token::LeftBrace)?;

    let mut members: Vec<EnumMember> = Vec::new();
    let mut orphan_comments: Vec<StandaloneCommentDefinition> = Vec::new();

    loop {
        let comment = tokens.maybe_expect_comment();

        match &comment {
            // Check for orphan comment
            Some(comment) => {
                match tokens.peek() {
                    None => {
                        error!("Sudden end of file in the middle of an enum!");
                        return Err(ParsingError::UnexpectedEndOfInput);
                    },
                    Some(next) => match &next.item {
                        // Create orphan comment from 'comment'
                        Token::Comment(_) => {
                            orphan_comments.push(StandaloneCommentDefinition {
                                comment:  comment.item.to_string(),
                                position: match members.len() {
                                    0 => CommentPosition::Start,
                                    _ => CommentPosition::Middle(members.len())
                                }
                            });

                            continue;
                        },

                        // Create orphan comment from 'comment'
                        Token::RightBrace => {
                            orphan_comments.push(StandaloneCommentDefinition {
                                comment:  comment.item.to_string(),
                                position: CommentPosition::End
                            });

                            tokens.expect_token(Token::RightBrace)?;
                            break;
                        },

                        // Parse next item entry normally
                        _ => ()
                    }
                }
            },
            None => ()
        };

        let field_ident = tokens.expect_identifier()?;

        tokens.expect_token(Token::Equals)?;

        let enum_value_token = tokens.expect_next()?;
        let enum_value = match enum_value_token.item {
            Token::NumericLiteral(value) => value,
            _ => return Err(ParsingError::UnexpectedToken(enum_value_token))
        };

        members.push(EnumMember {
            identifier: field_ident.item.clone(),
            value:      enum_value,
            comment:    comment.map(|s| s.item)
        });

        if tokens.maybe_expect(Token::SemiColon).is_none() {
            tokens.expect_token(Token::RightBrace)?;
            break;
        }
        if tokens.maybe_expect(Token::RightBrace).is_some() {
            break;
        }
    }

    Ok(EnumDefinition {
        name: identifier.item,
        backing_type: backing_type.item,
        orphan_comments: orphan_comments,
        members,
        comment
    })
}

fn parse_extension(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<ExtensionDefinition, ParsingError> {
    // Get extend token
    tokens.expect_token(Token::Extend)?;

    // Peek next token to see if it's a struct or enum
    let next_token = match tokens.peek() {
        Some(token) => token,
        None => return Err(ParsingError::UnexpectedEndOfInput)
    };

    match &next_token.item {
        Token::Bitfield => match parse_bitfield(tokens, last_comment) {
            Ok(definition) => Ok(ExtensionDefinition::Bitfield(definition)),
            Err(error) => return Err(error)
        },
        Token::Enum => match parse_enum(tokens, last_comment) {
            Ok(definition) => Ok(ExtensionDefinition::Enum(definition)),
            Err(error) => return Err(error)
        },
        Token::Struct => match parse_struct(tokens, last_comment) {
            Ok(definition) => Ok(ExtensionDefinition::Struct(definition)),
            Err(error) => return Err(error)
        },
        _ => return Err(ParsingError::UnexpectedToken(next_token.clone()))
    }
}

fn parse_include(tokens: &mut impl TokenSource, _: &mut Option<String>) -> Result<IncludeDefinition, ParsingError> {
    tokens.expect_next()?;

    let string: String = tokens.expect_string_literal()?.item.strip_suffix(".rune").expect("File included was now a .rune file").to_string();

    tokens.expect_token(Token::SemiColon)?;

    return Ok(IncludeDefinition { file: string });
}

fn parse_redefine(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<RedefineDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get redefine token
    tokens.expect_next()?;

    // Get definition name
    let definition_name = tokens.expect_identifier()?;

    let redefine_value_token = tokens.expect_next()?;
    let redefine_value: DefineValue = match redefine_value_token.item {
        Token::NumericLiteral(value) => DefineValue::NumericLiteral(value),
        _ => return Err(ParsingError::UnexpectedToken(redefine_value_token))
    };

    tokens.expect_token(Token::SemiColon)?;

    Ok(RedefineDefinition {
        identifier: definition_name.item,
        value:      redefine_value,
        comment:    comment
    })
}

fn parse_struct(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<StructDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get struct token
    tokens.expect_token(Token::Struct)?;

    // Get identifier
    let identifier = tokens.expect_identifier()?;

    tokens.expect_token(Token::LeftBrace)?;

    let mut members = Vec::new();
    let mut orphan_comments: Vec<StandaloneCommentDefinition> = Vec::new();

    loop {
        let comment = tokens.maybe_expect_comment();

        match &comment {
            // Check for orphan comment
            Some(comment) => {
                match tokens.peek() {
                    None => {
                        error!("Sudden end of file in the middle of an enum!");
                        return Err(ParsingError::UnexpectedEndOfInput);
                    },
                    Some(next) => match &next.item {
                        // Create orphan comment from 'comment'
                        Token::Comment(_) => {
                            orphan_comments.push(StandaloneCommentDefinition {
                                comment:  comment.item.to_string(),
                                position: match members.len() {
                                    0 => CommentPosition::Start,
                                    _ => CommentPosition::Middle(members.len())
                                }
                            });

                            continue;
                        },

                        // Create orphan comment from 'comment'
                        Token::RightBrace => {
                            orphan_comments.push(StandaloneCommentDefinition {
                                comment:  comment.item.to_string(),
                                position: CommentPosition::End
                            });

                            tokens.expect_token(Token::RightBrace)?;
                            break;
                        },

                        // Parse next item entry normally
                        _ => ()
                    }
                }
            },
            None => ()
        };

        let field_ident = tokens.expect_identifier()?;

        tokens.expect_token(Token::Colon)?;
        let tk = tokens.expect_type()?;

        tokens.expect_token(Token::Equals)?;

        let field_slot_token = tokens.expect_next()?;
        let field_slot: FieldSlot = match &field_slot_token.item {
            Token::NumericLiteral(value) => match value {
                NumericLiteral::Decimal(decimal) => match decimal {
                    // Legal values
                    0..32 => FieldSlot::Numeric(*decimal as usize),
                    // Higher than legal values
                    32.. => {
                        error!("Field index cannot have a value higher than 31!");
                        return Err(ParsingError::InvalidIndex(*decimal));
                    },
                    // Negative values
                    ..0 => {
                        error!("Field indexes cannot have negative values!");
                        return Err(ParsingError::InvalidIndex(*decimal));
                    }
                },
                NumericLiteral::Hexadecimal(hexadecimal) => match hexadecimal {
                    // Legal values
                    0..32 => FieldSlot::Numeric(*hexadecimal as usize),
                    // Higher than legal values
                    32.. => {
                        error!("Field index cannot have a value higher than 31!");
                        return Err(ParsingError::InvalidIndex(*hexadecimal as isize));
                    }
                },
                _ => return Err(ParsingError::UnexpectedToken(field_slot_token))
            },

            Token::Identifier(s) if s == "verifier" => FieldSlot::Verifier,
            _ => return Err(ParsingError::UnexpectedToken(field_slot_token))
        };

        members.push(StructMember {
            identifier: field_ident.item.clone(),
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

    Ok(StructDefinition {
        name: identifier.item,
        members,
        orphan_comments,
        comment
    })
}

pub fn parse_tokens(tokens: &mut impl TokenSource) -> ParsingResult<Definitions> {
    let mut definitions = Definitions::new();
    let mut last_comment: Option<String> = None;

    let mut last_was_comment: bool = false;

    'parsing: loop {
        let token = match tokens.peek() {
            None => break 'parsing,
            Some(token) => token
        };

        match &token.item {
            Token::Comment(_) => (),
            _ => last_was_comment = false
        };

        match &token.item {
            Token::Bitfield => match parse_bitfield(tokens, &mut last_comment) {
                Ok(definition) => definitions.bitfields.push(definition),
                Err(error) => return Err(error)
            },

            Token::Comment(s) => {
                if last_was_comment {
                    // Turn the last comment into a standalone comment
                    definitions.standalone_comments.push(StandaloneCommentDefinition {
                        comment:  match last_comment {
                            None => {
                                error!("Something went wrong in comment parsing logic");
                                return Err(ParsingError::LogicError);
                            },
                            Some(string) => string
                        },
                        position: CommentPosition::Start
                    });
                }

                last_comment = Some(s.clone());

                last_was_comment = true;

                tokens.expect_next()?;
            },

            Token::Define => match parse_define(tokens, &mut last_comment) {
                Ok(definition) => definitions.defines.push(definition),
                Err(error) => return Err(error)
            },

            Token::Enum => match parse_enum(tokens, &mut last_comment) {
                Ok(definition) => definitions.enums.push(definition),
                Err(error) => return Err(error)
            },

            Token::Extend => match parse_extension(tokens, &mut last_comment) {
                Ok(definition) => definitions.extensions.add_entry(definition),
                Err(error) => return Err(error)
            },

            Token::Include => match parse_include(tokens, &mut last_comment) {
                Ok(definition) => definitions.includes.push(definition),
                Err(error) => return Err(error)
            },

            Token::Redefine => match parse_redefine(tokens, &mut last_comment) {
                Ok(definition) => definitions.redefines.push(definition),
                Err(error) => return Err(error)
            },

            Token::Struct => match parse_struct(tokens, &mut last_comment) {
                Ok(definition) => definitions.structs.push(definition),
                Err(error) => return Err(error)
            },

            _ => return Err(ParsingError::UnexpectedToken(token.clone()))
        }
    }

    Ok(definitions)
}
