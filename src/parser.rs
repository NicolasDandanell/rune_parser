use std::iter::{Iterator, Peekable};

use crate::{output::*, scanner::*, types::*};

type ItemType = Spanned<Token>;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(ItemType),
    UnexpectedEndOfInput,
    ScanningError(ScanningError),
    InvalidBitSlot(NumericLiteral),
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
            NumericLiteral::Float(float) => float.to_string(),
            NumericLiteral::PositiveDecimal(integer) => integer.to_string(),
            NumericLiteral::NegativeDecimal(integer) => integer.to_string(),
            NumericLiteral::Hexadecimal(hex) => format!("0x{0:02X}", hex)
        }
    }

    pub fn to_field_index(&self) -> Result<usize, ParsingError> {
        match self {
            NumericLiteral::Boolean(_) => {
                error!("Boolean values are not valid as field indexes");
                return Err(ParsingError::InvalidIndex(self.clone()));
            },
            NumericLiteral::PositiveDecimal(decimal) => match decimal {
                // Legal values
                0..FieldSlot::FIELD_SLOT_LIMIT => Ok(*decimal as usize),
                // Higher than legal values
                FieldSlot::FIELD_SLOT_LIMIT.. => {
                    error!("Field index cannot have a value higher than 31!");
                    return Err(ParsingError::InvalidIndex(self.clone()));
                }
            },
            NumericLiteral::NegativeDecimal(_) => {
                error!("Field indexes cannot have negative values!");
                return Err(ParsingError::InvalidIndex(self.clone()));
            },
            NumericLiteral::Hexadecimal(hexadecimal) => match hexadecimal {
                // Legal values
                0..FieldSlot::FIELD_SLOT_LIMIT => Ok(*hexadecimal as usize),
                // Higher than legal values
                FieldSlot::FIELD_SLOT_LIMIT.. => {
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
                            0..FieldSlot::FIELD_SLOT_LIMIT => Ok(*float as usize),
                            // Higher than legal values
                            FieldSlot::FIELD_SLOT_LIMIT.. => {
                                error!("Field index cannot have a value higher than 31!");
                                return Err(ParsingError::InvalidIndex(self.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    fn to_bit_slot(&self) -> Result<usize, ParsingError> {
        match self {
            NumericLiteral::Boolean(_) => {
                error!("Boolean values are not valid as bitfield indexes");
                return Err(ParsingError::InvalidIndex(self.clone()));
            },
            NumericLiteral::PositiveDecimal(decimal) => match decimal {
                // Legal values
                0..BitSize::BIT_SLOT_LIMIT => Ok(*decimal as usize),
                // Higher than legal values
                BitSize::BIT_SLOT_LIMIT.. => {
                    error!("Bitfield index cannot have a value higher than 63!");
                    return Err(ParsingError::InvalidBitSlot(self.clone()));
                }
            },
            NumericLiteral::NegativeDecimal(_) => {
                error!("Bitfield indexes cannot have negative values!");
                return Err(ParsingError::InvalidBitSlot(self.clone()));
            },
            NumericLiteral::Hexadecimal(hexadecimal) => match hexadecimal {
                // Legal values
                0..BitSize::BIT_SLOT_LIMIT => Ok(*hexadecimal as usize),
                // Higher than legal values
                BitSize::BIT_SLOT_LIMIT.. => {
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
                            0..BitSize::BIT_SLOT_LIMIT => Ok(*float as usize),
                            // Higher than legal values
                            BitSize::BIT_SLOT_LIMIT.. => {
                                error!("Bitfield index cannot have a value higher than 63!");
                                return Err(ParsingError::InvalidIndex(self.clone()));
                            }
                        }
                    }
                }
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
                        NumericLiteral::PositiveDecimal(decimal) => ArraySize::DecimalValue(*decimal),
                        NumericLiteral::Hexadecimal(hexadecimal) => ArraySize::HexValue(*hexadecimal),
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

    if !backing_type.can_back_bitfield() {
        error!("{0} is not a valid backing type for a bitfield!", backing_type.to_string());
        return Err(ParsingError::InvalidBitfieldBackingType(backing_type));
    }

    // Get member fields
    tokens.expect_token(Token::LeftBrace)?;
    let mut members = Vec::new();
    let mut orphan_comments: Vec<StandaloneCommentDefinition> = Vec::new();
    let mut reserved_slots: Vec<usize> = Vec::new();

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
            for item in parse_reserved(tokens)? {
                let slot = item.to_bit_slot()?;
                match backing_type.validate_bit_slot(&slot) {
                    true => reserved_slots.push(item.to_bit_slot()?),
                    false => {
                        error!("Reserved index {0} in bitfield {1} is not valid within backing type {2}", slot, name, backing_type.to_string());
                        return Err(ParsingError::InvalidBitSlot(NumericLiteral::PositiveDecimal(slot as u64)));
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
        let bit_size_token: Spanned<BitSize> = tokens.expect_bitfield_size()?;
        let bit_size: BitSize = bit_size_token.item;

        // Bit field slot
        tokens.expect_token(Token::Equals)?;
        let bit_slot_token = tokens.expect_next()?;

        let bit_slot = match bit_slot_token.item {
            Token::NumericLiteral(value) => value.to_bit_slot()?,
            _ => return Err(ParsingError::UnexpectedToken(bit_slot_token))
        };

        if !backing_type.validate_bit_slot(&bit_slot) {
            error!("Index {0} in bitfield {1} is not valid within backing type {2}", bit_slot, name, backing_type.to_string());
            return Err(ParsingError::InvalidBitSlot(NumericLiteral::PositiveDecimal(bit_slot as u64)));
        };

        members.push(BitfieldMember {
            identifier,
            bit_size,
            bit_slot,
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
        reserved_slots,
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

    // Check against backing type
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
            for item in parse_reserved(tokens)? {
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

fn parse_reserved(tokens: &mut impl TokenSource) -> Result<Vec<NumericLiteral>, ParsingError> {
    tokens.expect_reserve()?;

    // A vector with capacity 32 should be plenty in most cases to handle most common use cases for reserved values
    let mut reserved_values: Vec<NumericLiteral> = Vec::with_capacity(0x20);

    // Loop until we find a semicolon
    loop {
        let token = tokens.expect_next()?;

        match &token.item {
            Token::NumericLiteral(value) => reserved_values.push(value.clone()),
            Token::NumericRange(start, end) => {
                let mut output_hex: bool = false;

                let start_value: u64 = match start {
                    NumericLiteral::Boolean(_) => return Err(ParsingError::UnexpectedToken(token)),
                    NumericLiteral::PositiveDecimal(value) => *value,
                    NumericLiteral::Hexadecimal(value) => {
                        output_hex = true;
                        *value
                    },
                    NumericLiteral::NegativeDecimal(_) => return Err(ParsingError::UnexpectedToken(token)),
                    NumericLiteral::Float(_) => return Err(ParsingError::UnexpectedToken(token))
                };

                let end_value: u64 = match end {
                    NumericLiteral::Boolean(_) => return Err(ParsingError::UnexpectedToken(token)),
                    NumericLiteral::PositiveDecimal(value) => *value,
                    NumericLiteral::Hexadecimal(value) => *value,
                    NumericLiteral::NegativeDecimal(_) => return Err(ParsingError::UnexpectedToken(token)),
                    NumericLiteral::Float(_) => return Err(ParsingError::UnexpectedToken(token))
                };

                for i in start_value..=end_value {
                    // Use the first value as reference
                    match output_hex {
                        true => reserved_values.push(NumericLiteral::Hexadecimal(i as u64)),
                        false => reserved_values.push(NumericLiteral::PositiveDecimal(i as u64))
                    }
                }
            },

            Token::Comma =>
            /* Keep parsing */
            {
                continue
            },

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
    let mut reserved_slots: Vec<FieldSlot> = Vec::new();

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
            for item in parse_reserved(tokens)? {
                reserved_slots.push(FieldSlot::Numeric(item.to_field_index()?));
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

        let field_slot_token = tokens.expect_next()?;
        let field_slot: FieldSlot = match &field_slot_token.item {
            Token::Verifier => FieldSlot::Verifier,
            Token::NumericLiteral(literal) => match literal.to_field_index() {
                Err(_) => return Err(ParsingError::UnexpectedToken(field_slot_token)),
                Ok(index) => FieldSlot::Numeric(index)
            },
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
        reserved_slots,
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
