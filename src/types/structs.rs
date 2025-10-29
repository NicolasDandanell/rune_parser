use crate::{
    scanner::{NumeralSystem, NumericLiteral},
    types::{BitfieldDefinition, DefineDefinition, DefineValue, EnumDefinition, StandaloneCommentDefinition}
};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    /// Name of the struct
    pub name:             String,
    /// Data fields of the struct
    pub members:          Vec<StructMember>,
    /// Indexes that are reserved, and should not be used
    pub reserved_indexes: Vec<FieldIndex>,
    /// Comment describing the struct
    pub comment:          Option<String>,
    /// Loose comments inside the bitfield declaration
    pub orphan_comments:  Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub struct StructMember {
    /// Name of the data field
    pub identifier:           String,
    /// Type of the data field
    pub data_type:            FieldType,
    /// Index of the data field
    pub index:                FieldIndex,
    /// If the data type of the field is a user defined one, this will contain a copy of it's definition
    pub user_definition_link: UserDefinitionLink,
    /// Comment describing the data field
    pub comment:              Option<String>
}

#[derive(Debug, Clone)]
pub enum UserDefinitionLink {
    NoLink,
    // Clone value of the bitfield definition
    BitfieldLink(BitfieldDefinition),
    // Clone value of the enum definition
    EnumLink(EnumDefinition),
    // Clone value of the struct definition
    StructLink(StructDefinition)
}

#[derive(Debug, Clone)]
/// Size of an array, storing how the user described the value
pub enum ArraySize {
    /// Size described by a integer number. Can be written in several numeric systems
    Integer(u64, NumeralSystem),
    /// Size described by value defined elsewhere by the user
    UserDefinition(DefineDefinition)
}

#[derive(Debug, Clone)]
pub enum FieldIndex {
    /// Used for regular fields
    Numeric(u64),

    /// Used for the verification field. Aliases to 0
    Verifier
}

impl FieldIndex {
    pub const LIMIT: u64 = 32;

    pub fn value(&self) -> u64 {
        match self {
            FieldIndex::Numeric(value) => *value,
            FieldIndex::Verifier => 0
        }
    }

    pub fn is_verifier(&self) -> bool {
        match self {
            FieldIndex::Verifier => true,
            _ => false
        }
    }
}

impl PartialEq for FieldIndex {
    fn eq(&self, other: &FieldIndex) -> bool {
        self.value() == other.value()
    }
}

#[derive(Debug, Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,

    // 1 byte primitives
    Bool,
    Char,
    I8,
    U8,

    // 2 byte primitives
    I16,
    U16,

    // 4 byte primitives
    F32,
    I32,
    U32,

    // 8 byte primitives
    F64,
    I64,
    U64,

    // 16 byte primitives (Not sendable)
    I128,
    U128,

    // Arrays and user definitions
    Array(Box<FieldType>, ArraySize),
    UserDefined(String)
}

impl ArraySize {
    pub fn to_string(&self) -> String {
        match self {
            ArraySize::Integer(value, numeral_system) => match numeral_system {
                NumeralSystem::Binary => format!("0b{0:b}", value),
                NumeralSystem::Decimal => value.to_string(),
                NumeralSystem::Hexadecimal => format!("0x{0:02X}", value)
            },

            ArraySize::UserDefinition(value) => value.name.clone()
        }
    }

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

impl PartialEq for ArraySize {
    fn eq(&self, other: &ArraySize) -> bool {
        match self {
            ArraySize::Integer(value, _) => match other {
                ArraySize::Integer(other_value, _) => return value == other_value,
                _ => false
            },
            ArraySize::UserDefinition(definition) => match other {
                ArraySize::UserDefinition(other_definition) => return definition.name == other_definition.name,
                _ => return false
            }
        }
    }
}

impl FieldType {
    pub fn to_string(&self) -> String {
        match self {
            FieldType::Empty => String::from("(empty)"),
            FieldType::Bool => String::from("bool"),
            FieldType::Char => String::from("char"),
            FieldType::I8 => String::from("i8"),
            FieldType::U8 => String::from("u8"),
            FieldType::I16 => String::from("i16"),
            FieldType::U16 => String::from("u16"),
            FieldType::F32 => String::from("float"),
            FieldType::I32 => String::from("i32"),
            FieldType::U32 => String::from("u32"),
            FieldType::F64 => String::from("double"),
            FieldType::I64 => String::from("i64"),
            FieldType::U64 => String::from("u64"),
            FieldType::I128 => String::from("i128"),
            FieldType::U128 => String::from("u128"),
            FieldType::Array(array_type, array_size) => format!("[{0}; {1}]", array_type.to_string(), array_size.to_string()),
            FieldType::UserDefined(string) => string.clone()
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            FieldType::Char | FieldType::I8 | FieldType::I16 | FieldType::F32 | FieldType::I32 | FieldType::F64 | FieldType::I64 => true,
            _ => false
        }
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &FieldType) -> bool {
        match self {
            FieldType::Empty => match other {
                FieldType::Empty => return true,
                _ => return false
            },

            FieldType::Bool => match other {
                FieldType::Bool => return true,
                _ => return false
            },

            FieldType::Char => match other {
                FieldType::Char => return true,
                _ => return false
            },

            FieldType::I8 => match other {
                FieldType::I8 => return true,
                _ => return false
            },

            FieldType::U8 => match other {
                FieldType::U8 => return true,
                _ => return false
            },

            FieldType::I16 => match other {
                FieldType::I16 => return true,
                _ => return false
            },

            FieldType::U16 => match other {
                FieldType::U16 => return true,
                _ => return false
            },

            FieldType::F32 => match other {
                FieldType::F32 => return true,
                _ => return false
            },

            FieldType::I32 => match other {
                FieldType::I32 => return true,
                _ => return false
            },

            FieldType::U32 => match other {
                FieldType::U32 => return true,
                _ => return false
            },

            FieldType::F64 => match other {
                FieldType::F64 => return true,
                _ => return false
            },

            FieldType::I64 => match other {
                FieldType::I64 => return true,
                _ => return false
            },

            FieldType::U64 => match other {
                FieldType::U64 => return true,
                _ => return false
            },

            FieldType::I128 => match other {
                FieldType::I128 => return true,
                _ => return false
            },

            FieldType::U128 => match other {
                FieldType::U128 => return true,
                _ => return false
            },

            FieldType::Array(array_type, array_size) => match other {
                FieldType::Array(other_type, other_size) => return (array_type == other_type) && (array_size == other_size),
                _ => return false
            },

            FieldType::UserDefined(string) => match other {
                FieldType::UserDefined(other_string) => return string == other_string,
                _ => false
            }
        }
    }
}
