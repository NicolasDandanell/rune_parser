use std::fmt::{Debug, Display, Formatter};

use crate::{
    scanner::{NumeralSystem, NumericLiteral},
    types::{structs::Primitive, DefineDefinition, DefineValue}
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
    UserDefined(String)
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

impl PartialEq for ArrayType {
    fn eq(&self, other: &ArrayType) -> bool {
        match self {
            ArrayType::Primitive(primitive) => match other {
                ArrayType::Primitive(other_primitive) => primitive == other_primitive,
                _ => false
            },
            ArrayType::UserDefined(definition) => match other {
                ArrayType::UserDefined(other_definition) => definition == other_definition,
                _ => false
            }
        }
    }
}
