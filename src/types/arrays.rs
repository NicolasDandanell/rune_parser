use std::fmt::{Debug, Display, Formatter};

use crate::{
    output::*,
    scanner::{NumeralSystem, NumericLiteral},
    types::{DefineDefinition, DefineValue, Primitive, UserDefinitionLink},
    RuneParserError
};

#[derive(Clone, Debug)]
/// Size of an array, storing how the user described the value
pub enum ArraySize {
    /// Size described by a integer number. Can be written in several numeric systems
    Integer(u64, NumeralSystem),
    /// Size described by value defined elsewhere by the user
    UserDefinition(DefineDefinition)
}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Primitive(Primitive),
    UserDefined(String, UserDefinitionLink)
}

#[derive(Clone, Debug)]
pub struct Array {
    pub data_type:     ArrayType,
    pub element_count: ArraySize
}

impl ArraySize {
    pub fn last_index_string(&self) -> String {
        match self {
            ArraySize::Integer(value, numeral_system) => match numeral_system {
                NumeralSystem::Binary => format!("0b{0:b}", value - 1),
                NumeralSystem::Decimal => (value - 1).to_string(),
                NumeralSystem::Hexadecimal => format!("0x{0:02X}", value - 1)
            },
            ArraySize::UserDefinition(definition) => {
                let value = match &definition.redefinition {
                    None => &definition.value,
                    Some(redefinition) => &redefinition.value
                };

                match value {
                    DefineValue::NoValue => format!("{0} - 1", definition.name),
                    DefineValue::NumericLiteral(literal) => match literal {
                        NumericLiteral::PositiveInteger(value, numeral_system) => match numeral_system {
                            NumeralSystem::Binary => format!("0b{0:b}", value - 1),
                            NumeralSystem::Decimal => format!("{0}", value - 1),
                            NumeralSystem::Hexadecimal => format!("0x{0:02X}", value - 1)
                        },

                        _ => unreachable!("Only positive integer numbers can be indexes")
                    }
                }
            }
        }
    }

    pub fn value(&self) -> Result<u64, RuneParserError> {
        match self {
            ArraySize::Integer(value, _) => Ok(*value),
            ArraySize::UserDefinition(definition) => {
                let define_value = match &definition.redefinition {
                    None => &definition.value,
                    Some(redefinition) => &redefinition.value
                };

                match define_value {
                    DefineValue::NumericLiteral(NumericLiteral::PositiveInteger(value, _)) => Ok(*value),
                    _ => Err(RuneParserError::InvalidArraySize)
                }
            }
        }
    }
}

impl Display for ArraySize {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Integer(value, numeral_system) => match numeral_system {
                NumeralSystem::Binary => write!(formatter, "0b{0:b}", value),
                NumeralSystem::Decimal => write!(formatter, "{0}", value),
                NumeralSystem::Hexadecimal => write!(formatter, "0x{0:02X}", value)
            },

            ArraySize::UserDefinition(value) => write!(formatter, "{0}", value.name)
        }
    }
}

impl PartialEq for ArraySize {
    fn eq(&self, other: &ArraySize) -> bool {
        match self {
            ArraySize::Integer(value, _) => match other {
                ArraySize::Integer(other_value, _) => value == other_value,
                _ => false
            },
            ArraySize::UserDefinition(definition) => match other {
                ArraySize::UserDefinition(other_definition) => definition.name == other_definition.name,
                _ => false
            }
        }
    }
}

impl ArrayType {
    pub fn size(&self) -> Result<u64, RuneParserError> {
        match self {
            ArrayType::Primitive(primitive) => Ok(primitive.encoded_max_data_size()),
            ArrayType::UserDefined(_, definition_link) => match &definition_link {
                UserDefinitionLink::NoLink => {
                    error!("User defined array type had no link!");
                    Err(RuneParserError::UndefinedIdentifier)
                },
                UserDefinitionLink::EnumLink(enum_link) => Ok(enum_link.backing_type.encoded_max_data_size()),
                UserDefinitionLink::BitfieldLink(bitfield_link) => Ok(bitfield_link.backing_type.encoded_max_data_size()),
                UserDefinitionLink::MessageLink(_) => {
                    error!("Cannot have message array");
                    Err(RuneParserError::InvalidArrayType)
                },
                UserDefinitionLink::StructLink(struct_link) => Ok(struct_link.flat_size()?)
            }
        }
    }
}

impl PartialEq for ArrayType {
    fn eq(&self, other: &ArrayType) -> bool {
        match self {
            ArrayType::Primitive(primitive) => match other {
                ArrayType::Primitive(other_primitive) => primitive == other_primitive,
                _ => false
            },
            ArrayType::UserDefined(definition, _) => match other {
                ArrayType::UserDefined(other_definition, _) => definition == other_definition,
                _ => false
            }
        }
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Array) -> bool {
        (self.data_type == other.data_type) && (self.element_count == other.element_count)
    }
}

impl Array {
    pub fn byte_size(&self) -> Result<u64, RuneParserError> {
        Ok(self.data_type.size()? * self.element_count.value()?)
    }
}
