use std::iter::{Iterator, Peekable};

use crate::{output::*, scanner::*, types::*};

type ItemType = Spanned<Token>;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(ItemType),
    UnexpectedEndOfInput,
    ScanningError(ScanningError),
    InvalidBitIndex(NumericLiteral),
    InvalidIndex(NumericLiteral),
    InvalidBitfieldBackingType(FieldType),
    InvalidEnumBackingType(FieldType),
    InvalidEnumValue(NumericLiteral),
    LogicError
}

impl NumericLiteral {
    pub fn to_string(&self) -> String {
        match self {
            NumericLiteral::Boolean(boolean) => boolean.to_string(),
            NumericLiteral::PositiveBinary(binary) => format!("0b{0:02b}", binary),
            NumericLiteral::NegativeBinary(binary) => format!("-0b{0:02b}", binary),
            NumericLiteral::Float(float) => float.to_string(),
            NumericLiteral::PositiveDecimal(integer) => integer.to_string(),
            NumericLiteral::NegativeDecimal(integer) => integer.to_string(),
            NumericLiteral::PositiveHexadecimal(hex) => format!("0x{0:02X}", hex),
            NumericLiteral::NegativeHexadecimal(hex) => format!("-0x{0:02X}", hex)
        }
    }

    pub fn to_field_index(&self) -> Result<u64, ParsingError> {
        match self {
            NumericLiteral::Boolean(_) => {
                error!("Boolean values are not valid as field indexes");
                return Err(ParsingError::InvalidIndex(self.clone()));
            },
            NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => match value {
                // Legal values
                0..FieldIndex::LIMIT => Ok(*value),
                // Higher than legal values
                FieldIndex::LIMIT.. => {
                    error!("Field index cannot have a value higher than 31!");
                    return Err(ParsingError::InvalidIndex(self.clone()));
                }
            },
            // Floating points can be used if they represent an integer value. I have no clue why one would do that though...
            NumericLiteral::Float(float) => match float.fract() == 0.0 {
                false => {
                    error!("Field indexes must have integer values!");
                    return Err(ParsingError::InvalidIndex(self.clone()));
                },
                true => match *float < 0.0 {
                    true => {
                        error!("Field indexes cannot have negative values!");
                        return Err(ParsingError::InvalidIndex(self.clone()));
                    },
                    false => {
                        match *float as u64 {
                            // Legal values
                            0..FieldIndex::LIMIT => Ok(*float as u64),
                            // Higher than legal values
                            FieldIndex::LIMIT.. => {
                                error!("Field index cannot have a value higher than 31!");
                                return Err(ParsingError::InvalidIndex(self.clone()));
                            }
                        }
                    }
                }
            },
            _ => {
                error!("Field indexes cannot have negative values!");
                return Err(ParsingError::InvalidIndex(self.clone()));
            }
        }
    }

    fn to_bit_index(&self) -> Result<u64, ParsingError> {
        match self {
            NumericLiteral::Boolean(_) => {
                error!("Boolean values are not valid as bitfield indexes");
                return Err(ParsingError::InvalidIndex(self.clone()));
            },
            NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => match value {
                // Legal values
                0..BitSize::LIMIT => Ok(*value),
                // Higher than legal values
                BitSize::LIMIT.. => {
                    error!("Bitfield index cannot have a value higher than 63!");
                    return Err(ParsingError::InvalidIndex(self.clone()));
                }
            },
            // Floating points can be used if they represent an integer value. I have no clue why one would do that though...
            NumericLiteral::Float(float) => match float.fract() == 0.0 {
                false => {
                    error!("Bitfield indexes must have integer values!");
                    return Err(ParsingError::InvalidIndex(self.clone()));
                },
                true => {
                    match *float < 0.0 {
                        true => {
                            error!("Bitfield indexes cannot have negative values!");
                            return Err(ParsingError::InvalidIndex(self.clone()));
                        },
                        false => match *float as u64 {
                            // Legal values
                            0..BitSize::LIMIT => Ok(*float as u64),
                            // Higher than legal values
                            BitSize::LIMIT.. => {
                                error!("Bitfield index cannot have a value higher than 63!");
                                return Err(ParsingError::InvalidIndex(self.clone()));
                            }
                        }
                    }
                }
            },
            _ => {
                error!("Bitfield indexes cannot have negative values!");
                return Err(ParsingError::InvalidBitIndex(self.clone()));
            }
        }
    }
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

                let size: u64 = match string[1..].parse() {
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

    fn expect_identifier(&mut self) -> ParsingResult<Spanned<String>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Identifier(string) => Ok(Spanned::new(string, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_next(&mut self) -> ParsingResult<ItemType> {
        match self.next() {
            None => Err(ParsingError::UnexpectedEndOfInput),
            Some(token) => Ok(token)
        }
    }

    fn expect_reserve(&mut self) -> ParsingResult<Spanned<Token>> {
        let token = self.expect_next()?;
        match token.item {
            Token::Reserve => Ok(token),
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

    fn expect_numeric_literal(&mut self) -> ParsingResult<Spanned<NumericLiteral>> {
        let token = self.expect_next()?;
        match token.item {
            Token::NumericLiteral(literal) => Ok(Spanned::new(literal, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token))
        }
    }

    fn expect_token(&mut self, expected_token: Token) -> ParsingResult<ItemType> {
        match self.expect_next()? {
            token if *token == expected_token => Ok(token),
            token => Err(ParsingError::UnexpectedToken(token))
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
                        NumericLiteral::PositiveBinary(binary) => ArraySize::Binary(*binary),
                        NumericLiteral::PositiveDecimal(decimal) => ArraySize::Decimal(*decimal),
                        NumericLiteral::PositiveHexadecimal(hexadecimal) => ArraySize::Hexadecimal(*hexadecimal),
                        _ => return Err(ParsingError::UnexpectedToken(count_token))
                    },

                    // String will generate a user definition, which will be populated with a value in post processing
                    Token::Identifier(string) => ArraySize::UserDefinition(DefineDefinition {
                        name:         string.clone(),
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

fn check_for_orphan_comment(tokens: &mut impl TokenSource, index: usize, comment: &Option<Spanned<String>>) -> Option<StandaloneCommentDefinition> {
    // Peek next token
    let peeked_token = match tokens.peek() {
        Some(token) => token.clone(),
        None => return None
    };

    match comment {
        // Create orphan comment from previous 'comment'
        Some(comment) => match peeked_token.item {
            Token::Comment(_) => Some(StandaloneCommentDefinition {
                comment: comment.item.to_string(),
                index
            }),
            Token::RightBrace => Some(StandaloneCommentDefinition {
                comment: comment.item.to_string(),
                index
            }),
            _ => None
        },
        None => None
    }
}

fn parse_bitfield(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<BitfieldDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Type and identifier
    tokens.expect_token(Token::Bitfield)?;
    let name = tokens.expect_identifier()?.item;

    // Backing type
    tokens.expect_token(Token::Colon)?;
    let backing_type = tokens.expect_type()?.item;

    // Validate backing type
    if !backing_type.can_back_bitfield() {
        error!("{0} is not a valid backing type for a bitfield!", backing_type.to_string());
        return Err(ParsingError::InvalidBitfieldBackingType(backing_type));
    }

    // Get member fields
    tokens.expect_token(Token::LeftBrace)?;
    let mut members = Vec::new();
    let mut orphan_comments: Vec<StandaloneCommentDefinition> = Vec::new();
    let mut reserved_indexes: Vec<u64> = Vec::new();

    loop {
        // Get comment if any
        let comment = tokens.maybe_expect_comment();

        // Peek next token
        let peeked_token = match tokens.peek() {
            Some(token) => token.clone(),
            None => {
                error!("Sudden end of file in the middle of a struct!");
                return Err(ParsingError::UnexpectedEndOfInput);
            }
        };

        // Check for orphan comments
        let orphan_comment = check_for_orphan_comment(tokens, members.len(), &comment);

        if let Some(orphan_comment) = orphan_comment {
            // Add orphan comment to list
            orphan_comments.push(orphan_comment);

            // If the next token is a right brace, then the definition has ended, so break and return
            if tokens.maybe_expect(Token::RightBrace).is_some() {
                break;
            }
            continue;
        }

        // Check for reserved values
        if peeked_token.item == Token::Reserve {
            // Push field index to reservation list if valid, throw error if not
            for item in parse_reserved(tokens, false)? {
                let index = item.to_bit_index()?;
                match backing_type.validate_bit_index(&index) {
                    true => reserved_indexes.push(item.to_bit_index()?),
                    false => {
                        error!("Reserved index {0} in bitfield {1} is not valid within backing type {2}", index, name, backing_type.to_string());
                        return Err(ParsingError::InvalidBitIndex(NumericLiteral::PositiveDecimal(index as u64)));
                    }
                }

                // If the next token is a right brace, then the definition has ended, so break and return
                if tokens.maybe_expect(Token::RightBrace).is_some() {
                    break;
                }

                continue;
            }
        }

        // Parser bitfield member
        // ———————————————————————

        // Identifier
        let identifier = tokens.expect_identifier()?.item;

        // Bit size
        tokens.expect_token(Token::Colon)?;
        let size_token: Spanned<BitSize> = tokens.expect_bitfield_size()?;
        let size: BitSize = size_token.item;

        // Bit field index
        tokens.expect_token(Token::Equals)?;
        let bit_index_token = tokens.expect_next()?;

        let index = match bit_index_token.item {
            Token::NumericLiteral(value) => value.to_bit_index()?,
            _ => return Err(ParsingError::UnexpectedToken(bit_index_token))
        };

        if !backing_type.validate_bit_index(&index) {
            error!("Index {0} in bitfield {1} is not valid within backing type {2}", index, name, backing_type.to_string());
            return Err(ParsingError::InvalidBitIndex(NumericLiteral::PositiveDecimal(index as u64)));
        };

        members.push(BitfieldMember {
            identifier,
            size,
            index,
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

    return Ok(BitfieldDefinition {
        name,
        backing_type,
        members,
        reserved_indexes,
        comment,
        orphan_comments
    });
}

fn parse_define(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<DefineDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get define token
    tokens.expect_next()?;

    // Get definition name
    let name = tokens.expect_identifier()?.item;

    let value_token = tokens.expect_next()?;
    let value: DefineValue = match value_token.item {
        Token::NumericLiteral(value) => DefineValue::NumericLiteral(value),
        _ => return Err(ParsingError::UnexpectedToken(value_token))
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
        name,
        value,
        comment,
        redefinition: None
    })
}

fn parse_enum(tokens: &mut impl TokenSource, last_comment: &mut Option<String>) -> Result<EnumDefinition, ParsingError> {
    // Get comment if any
    let comment = last_comment.take();

    // Get enum token
    tokens.expect_token(Token::Enum)?;

    // Get identifier
    let name = tokens.expect_identifier()?.item;

    // Get backing type
    tokens.expect_token(Token::Colon)?;
    let backing_type = tokens.expect_type()?.item;

    // Validate backing type
    if !backing_type.can_back_enum() {
        error!("{0} is not a valid backing type for an enum!", backing_type.to_string());
        return Err(ParsingError::InvalidEnumBackingType(backing_type));
    }

    tokens.expect_token(Token::LeftBrace)?;

    let mut members: Vec<EnumMember> = Vec::new();
    let mut orphan_comments: Vec<StandaloneCommentDefinition> = Vec::new();
    let mut reserved_values: Vec<NumericLiteral> = Vec::new();

    loop {
        let comment = tokens.maybe_expect_comment();

        // Peek next token
        let peeked_token = match tokens.peek() {
            Some(token) => token.clone(),
            None => {
                error!("Sudden end of file in the middle of a struct!");
                return Err(ParsingError::UnexpectedEndOfInput);
            }
        };

        // Check for orphan comments
        let orphan_comment = check_for_orphan_comment(tokens, members.len(), &comment);

        if let Some(orphan_comment) = orphan_comment {
            // Add orphan comment to list
            orphan_comments.push(orphan_comment);

            // If the next token is a right brace, then the definition has ended, so break and return
            if tokens.maybe_expect(Token::RightBrace).is_some() {
                break;
            }
            continue;
        }

        // Check for reserved values
        if peeked_token.item == Token::Reserve {
            // Push field index to reservation list if valid, throw error if not
            for item in parse_reserved(tokens, true)? {
                match backing_type.validate_value(&item) {
                    true => reserved_values.push(item),
                    false => {
                        error!(
                            "Reserved enum value {0} in enum {1} does not conform within backing type {2}",
                            item.to_string(),
                            name,
                            backing_type.to_string()
                        );
                        return Err(ParsingError::InvalidEnumValue(item));
                    }
                }
            }

            // If the next token is a right brace, then the definition has ended, so break and return
            if tokens.maybe_expect(Token::RightBrace).is_some() {
                break;
            }

            continue;
        }

        // Parse enum member
        // ——————————————————

        let identifier = tokens.expect_identifier()?.item;

        tokens.expect_token(Token::Equals)?;

        let value_token = tokens.expect_next()?;
        let value = match value_token.item {
            Token::NumericLiteral(value) => value,
            _ => return Err(ParsingError::UnexpectedToken(value_token))
        };

        // Validate value against backing type
        if !backing_type.validate_value(&value) {
            error!("Value {0} in enum {1} does not conform within backing type {2}", value.to_string(), name, backing_type.to_string());
            return Err(ParsingError::InvalidEnumValue(value));
        }

        members.push(EnumMember {
            identifier,
            value,
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

    Ok(EnumDefinition {
        name,
        backing_type,
        orphan_comments,
        members,
        reserved_values,
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
    let name = tokens.expect_identifier()?.item;

    let value_token = tokens.expect_next()?;
    let value: DefineValue = match value_token.item {
        Token::NumericLiteral(value) => DefineValue::NumericLiteral(value),
        _ => return Err(ParsingError::UnexpectedToken(value_token))
    };

    tokens.expect_token(Token::SemiColon)?;

    Ok(RedefineDefinition { name, value, comment })
}

fn parse_reserved(tokens: &mut impl TokenSource, allow_negative: bool) -> Result<Vec<NumericLiteral>, ParsingError> {
    tokens.expect_reserve()?;

    // A vector with capacity 32 should be plenty in most cases to handle most common use cases for reserved values
    let mut reserved_values: Vec<NumericLiteral> = Vec::with_capacity(0x20);

    // Loop until we find a semicolon
    loop {
        let token = tokens.expect_next()?;

        match &token.item {
            Token::NumericLiteral(value) => reserved_values.push(value.clone()),
            Token::NumericRange(start_value, end_value) => {
                let mut negatives: bool = false;

                // Verify start
                match start_value {
                    NumericLiteral::PositiveBinary(_) | NumericLiteral::PositiveDecimal(_) | NumericLiteral::PositiveHexadecimal(_) => (),
                    NumericLiteral::NegativeBinary(_) | NumericLiteral::NegativeDecimal(_) | NumericLiteral::NegativeHexadecimal(_) => negatives = true,
                    _ => return Err(ParsingError::UnexpectedToken(token))
                };

                // Verify end
                match end_value {
                    NumericLiteral::PositiveBinary(_) | NumericLiteral::PositiveDecimal(_) | NumericLiteral::PositiveHexadecimal(_) => (),
                    NumericLiteral::NegativeBinary(_) | NumericLiteral::NegativeDecimal(_) | NumericLiteral::NegativeHexadecimal(_) => {
                        if !negatives {
                            return Err(ParsingError::UnexpectedToken(token));
                        }
                    },
                    _ => return Err(ParsingError::UnexpectedToken(token))
                };

                if negatives && !allow_negative {
                    return Err(ParsingError::UnexpectedToken(token));
                }

                // Process range differently depending on if there are negatives
                match negatives {
                    // Process signed range
                    true => {
                        let start = match start_value {
                            NumericLiteral::NegativeBinary(value) | NumericLiteral::NegativeDecimal(value) | NumericLiteral::NegativeHexadecimal(value) => *value,
                            _ => return Err(ParsingError::UnexpectedToken(token))
                        };
                        let end = match end_value {
                            NumericLiteral::NegativeBinary(value) | NumericLiteral::NegativeDecimal(value) | NumericLiteral::NegativeHexadecimal(value) => *value,
                            _ => return Err(ParsingError::UnexpectedToken(token))
                        };

                        // Check that end is larger than start
                        if !(end > start) {
                            error!("Start of range was larger or equal to end of range");
                            return Err(ParsingError::LogicError);
                        }

                        for i in start..end {
                            // Use the first value as reference
                            reserved_values.push(match start_value {
                                NumericLiteral::PositiveBinary(_) | NumericLiteral::NegativeBinary(_) => match i < 0 {
                                    true => NumericLiteral::NegativeBinary(i),
                                    false => NumericLiteral::PositiveBinary(i as u64)
                                },
                                NumericLiteral::PositiveDecimal(_) | NumericLiteral::NegativeDecimal(_) => match i < 0 {
                                    true => NumericLiteral::NegativeDecimal(i),
                                    false => NumericLiteral::PositiveDecimal(i as u64)
                                },
                                NumericLiteral::PositiveHexadecimal(_) | NumericLiteral::NegativeHexadecimal(_) => match i < 0 {
                                    true => NumericLiteral::NegativeHexadecimal(i),
                                    false => NumericLiteral::PositiveHexadecimal(i as u64)
                                },
                                _ => return Err(ParsingError::UnexpectedToken(token))
                            })
                        }
                    },
                    // Process unsigned range
                    false => {
                        let start = match start_value {
                            NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value,
                            _ => return Err(ParsingError::UnexpectedToken(token))
                        };
                        let end = match end_value {
                            NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value,
                            _ => return Err(ParsingError::UnexpectedToken(token))
                        };

                        // Check that end is larger than start
                        if !(end > start) {
                            error!("Start of range was larger or equal to end of range");
                            return Err(ParsingError::LogicError);
                        }

                        for i in start..end {
                            // Use the first value as reference
                            reserved_values.push(match start_value {
                                NumericLiteral::PositiveBinary(_) => NumericLiteral::PositiveBinary(i),
                                NumericLiteral::PositiveDecimal(_) => NumericLiteral::PositiveDecimal(i),
                                NumericLiteral::PositiveHexadecimal(_) => NumericLiteral::PositiveHexadecimal(i),
                                _ => return Err(ParsingError::UnexpectedToken(token))
                            })
                        }
                    }
                }
            },

            // Keep parsing
            Token::Comma => continue,

            // Done parsing
            Token::SemiColon => break,
            _ => return Err(ParsingError::UnexpectedToken(token))
        }
    }

    return Ok(reserved_values);
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
    let mut reserved_indexes: Vec<FieldIndex> = Vec::new();

    loop {
        let comment = tokens.maybe_expect_comment();

        // Peek next token
        let peeked_token = match tokens.peek() {
            Some(token) => token.clone(),
            None => {
                error!("Sudden end of file in the middle of a struct!");
                return Err(ParsingError::UnexpectedEndOfInput);
            }
        };

        // Check for orphan comments
        let orphan_comment = check_for_orphan_comment(tokens, members.len(), &comment);

        if let Some(orphan_comment) = orphan_comment {
            // Add orphan comment to list
            orphan_comments.push(orphan_comment);

            // If the next token is a right brace, then the definition has ended, so break and return
            if tokens.maybe_expect(Token::RightBrace).is_some() {
                break;
            }
            continue;
        }

        // Check for reserved values
        if peeked_token.item == Token::Reserve {
            // Push field index to reservation list if valid, throw error if not
            for item in parse_reserved(tokens, false)? {
                reserved_indexes.push(FieldIndex::Numeric(item.to_field_index()?));
            }

            // If the next token is a right brace, then the definition has ended, so break and return
            if tokens.maybe_expect(Token::RightBrace).is_some() {
                break;
            }

            continue;
        }

        // Parse struct member
        // ————————————————————

        let field_ident = tokens.expect_identifier()?;

        tokens.expect_token(Token::Colon)?;
        let tk = tokens.expect_type()?;

        tokens.expect_token(Token::Equals)?;

        let index_token = tokens.expect_next()?;
        let index: FieldIndex = match &index_token.item {
            Token::Verifier => FieldIndex::Verifier,
            Token::NumericLiteral(literal) => match literal.to_field_index() {
                Err(_) => return Err(ParsingError::UnexpectedToken(index_token)),
                Ok(index) => FieldIndex::Numeric(index)
            },
            _ => return Err(ParsingError::UnexpectedToken(index_token))
        };

        members.push(StructMember {
            identifier: field_ident.item.clone(),
            data_type: tk.item.clone(),
            index,
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
        reserved_indexes,
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
                        comment: match last_comment {
                            None => {
                                error!("Something went wrong in comment parsing logic");
                                return Err(ParsingError::LogicError);
                            },
                            Some(string) => string
                        },
                        // Use index 0 for stray comments in Rune files for now
                        index:   0
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
