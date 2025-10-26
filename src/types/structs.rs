use crate::{
    scanner::NumericLiteral,
    types::{BitfieldDefinition, DefineDefinition, DefineValue, EnumDefinition, StandaloneCommentDefinition}
};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    /// Name of the struct
    pub name:             String,
    /// Data fields of the struct
    pub members:          Vec<StructMember>,
    /// Indexes that are reserved, and should not be used
    pub reserved_indexes: Vec<FieldSlot>,
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
    pub index:                FieldSlot,
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
    /// Size described by a binary number
    Binary(u64),
    /// Size described by a decimal number
    Decimal(u64),
    /// Size described by a hexadecimal number
    Hexadecimal(u64),
    /// Size described by value defined elsewhere by the user
    UserDefinition(DefineDefinition)
}

#[derive(Debug, Clone)]
pub enum FieldSlot {
    /// Used for regular fields
    Numeric(u64),

    /// Used for the verification field. Aliases to 0
    Verifier
}

impl FieldSlot {
    pub const FIELD_SLOT_LIMIT: u64 = 32;

    pub fn value(&self) -> u64 {
        match self {
            FieldSlot::Numeric(value) => *value,
            FieldSlot::Verifier => 0
        }
    }

    pub fn is_verifier(&self) -> bool {
        match self {
            FieldSlot::Verifier => true,
            _ => false
        }
    }
}

impl PartialEq for FieldSlot {
    fn eq(&self, other: &FieldSlot) -> bool {
        self.value() == other.value()
    }
}

#[derive(Debug, Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,

    // 1 byte primitives
    Boolean,
    Char,
    UByte,
    Byte,

    // 2 byte primitives
    UShort,
    Short,

    // 4 byte primitives
    Float,
    UInt,
    Int,

    // 8 byte primitives
    Double,
    ULong,
    Long,

    // Arrays and user definitions
    Array(Box<FieldType>, ArraySize),
    UserDefined(String)
}

impl ArraySize {
    pub fn to_string(&self) -> String {
        match self {
            ArraySize::Binary(value) => format!("0b{0:b}", value),
            ArraySize::Decimal(value) => format!("{0}", value),
            ArraySize::Hexadecimal(value) => format!("0x{0:02X}", value),
            ArraySize::UserDefinition(value) => value.name.clone()
        }
    }

    pub fn last_index_string(&self) -> String {
        match self {
            ArraySize::Binary(value) => format!("0b{0:b}", value - 1),
            ArraySize::Decimal(value) => format!("{0}", value - 1),
            ArraySize::Hexadecimal(value) => format!("0x{0:02X}", value - 1),
            ArraySize::UserDefinition(definition) => {
                let value = match &definition.redefinition {
                    None => &definition.value,
                    Some(redefinition) => &redefinition.value
                };

                match value {
                    DefineValue::NoValue => format!("{0} - 1", definition.name),
                    DefineValue::NumericLiteral(literal) => match literal {
                        NumericLiteral::PositiveBinary(value) => format!("0b{0:b}", value - 1),
                        NumericLiteral::PositiveDecimal(value) => format!("{0}", value - 1),
                        NumericLiteral::PositiveHexadecimal(value) => format!("0x{0:02X}", value - 1),
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
            ArraySize::Binary(value) | ArraySize::Decimal(value) | ArraySize::Hexadecimal(value) => match other {
                ArraySize::Binary(other_value) | ArraySize::Decimal(other_value) | ArraySize::Hexadecimal(other_value) => return value == other_value,
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
            FieldType::Boolean => String::from("bool"),
            FieldType::Char => String::from("char"),
            FieldType::UByte => String::from("u8"),
            FieldType::Byte => String::from("i8"),
            FieldType::UShort => String::from("u16"),
            FieldType::Short => String::from("i16"),
            FieldType::Float => String::from("float"),
            FieldType::UInt => String::from("u32"),
            FieldType::Int => String::from("i32"),
            FieldType::Double => String::from("double"),
            FieldType::ULong => String::from("u64"),
            FieldType::Long => String::from("i64"),
            FieldType::Array(array_type, array_size) => format!("[{0}; {1}]", array_type.to_string(), array_size.to_string()),
            FieldType::UserDefined(string) => string.clone()
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            FieldType::Int | FieldType::Char | FieldType::Byte | FieldType::Long | FieldType::Short | FieldType::Float | FieldType::Double => true,
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

            FieldType::Boolean => match other {
                FieldType::Boolean => return true,
                _ => return false
            },

            FieldType::Char => match other {
                FieldType::Char => return true,
                _ => return false
            },

            FieldType::UByte => match other {
                FieldType::UByte => return true,
                _ => return false
            },

            FieldType::Byte => match other {
                FieldType::Byte => return true,
                _ => return false
            },

            FieldType::UShort => match other {
                FieldType::UShort => return true,
                _ => return false
            },

            FieldType::Short => match other {
                FieldType::Short => return true,
                _ => return false
            },

            FieldType::Float => match other {
                FieldType::Float => return true,
                _ => return false
            },

            FieldType::UInt => match other {
                FieldType::UInt => return true,
                _ => return false
            },

            FieldType::Int => match other {
                FieldType::Int => return true,
                _ => return false
            },

            FieldType::Double => match other {
                FieldType::Double => return true,
                _ => return false
            },

            FieldType::ULong => match other {
                FieldType::ULong => return true,
                _ => return false
            },

            FieldType::Long => match other {
                FieldType::Long => return true,
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
