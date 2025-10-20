use crate::types::{BitfieldDefinition, DefineDefinition, EnumDefinition, StandaloneCommentDefinition};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name:            String,
    pub members:         Vec<StructMember>,
    pub orphan_comments: Vec<StandaloneCommentDefinition>,
    pub comment:         Option<String>
}

#[derive(Debug, Clone)]
pub struct StructMember {
    pub ident:                String,
    pub field_type:           FieldType,
    pub field_slot:           FieldSlot,
    pub user_definition_link: UserDefinitionLink,
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
pub enum ArraySize {
    DecimalValue(usize),
    HexValue(usize),
    UserDefinition(DefineDefinition)
}

#[derive(Debug, Clone)]
pub enum FieldSlot {
    /// Used for regular fields
    Numeric(usize),

    /// Used for the verification field. Aliases to 0
    Verifier
}

#[derive(Debug, Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,

    /// Used to reserve the index for Verification Fields. Not implemented yet!
    VerificationReserve,

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
    pub fn print(&self) -> String {
        match self {
            ArraySize::DecimalValue(value) => format!("{0}", value),
            ArraySize::HexValue(value) => format!("0x{0:02X}", value),
            ArraySize::UserDefinition(value) => value.identifier.clone()
        }
    }
}

impl PartialEq for ArraySize {
    fn eq(&self, other: &ArraySize) -> bool {
        match self {
            ArraySize::DecimalValue(value) | ArraySize::HexValue(value) => match other {
                ArraySize::DecimalValue(other_value) | ArraySize::HexValue(other_value) => return value == other_value,
                _ => false
            },
            ArraySize::UserDefinition(definition) => match other {
                ArraySize::UserDefinition(other_definition) => return definition.identifier == other_definition.identifier,
                _ => return false
            }
        }
    }
}

impl FieldType {
    pub fn print(&self) -> String {
        match self {
            FieldType::Empty => String::from("(empty)"),
            FieldType::VerificationReserve => String::from("verifier"),
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
            FieldType::Array(array_type, array_size) => format!("[{0}; {1}]", array_type.print(), array_size.print()),
            FieldType::UserDefined(string) => string.clone()
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

            FieldType::VerificationReserve => match other {
                FieldType::VerificationReserve => return true,
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
