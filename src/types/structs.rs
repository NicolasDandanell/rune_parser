use std::fmt::{Debug, Formatter, Result};

use crate::types::{ArraySize, ArrayType, BitfieldDefinition, EnumDefinition, StandaloneCommentDefinition};

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
        matches!(self, FieldIndex::Verifier)
    }
}

impl PartialEq for FieldIndex {
    fn eq(&self, other: &FieldIndex) -> bool {
        self.value() == other.value()
    }
}

#[derive(Clone, Debug)]
pub enum Primitive {
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

    // 16 byte primitives (Not sendable as primitive)
    I128,
    U128
}

#[derive(Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,
    Primitive(Primitive),
    Array(ArrayType, ArraySize),
    UserDefined(String)
}

impl Debug for FieldType {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            FieldType::Empty => write!(formatter, "(empty)"),
            FieldType::Primitive(primitive) => match primitive {
                Primitive::Bool => write!(formatter, "bool"),
                Primitive::Char => write!(formatter, "char"),
                Primitive::I8 => write!(formatter, "i8"),
                Primitive::U8 => write!(formatter, "u8"),
                Primitive::I16 => write!(formatter, "i16"),
                Primitive::U16 => write!(formatter, "u16"),
                Primitive::F32 => write!(formatter, "f32"),
                Primitive::I32 => write!(formatter, "i32"),
                Primitive::U32 => write!(formatter, "u32"),
                Primitive::F64 => write!(formatter, "f64"),
                Primitive::I64 => write!(formatter, "i64"),
                Primitive::U64 => write!(formatter, "u64"),
                Primitive::I128 => write!(formatter, "i128"),
                Primitive::U128 => write!(formatter, "u128")
            },
            FieldType::Array(array_type, array_size) => write!(formatter, "[{0:?}; {1}]", array_type, array_size.to_string()),
            FieldType::UserDefined(string) => write!(formatter, "{0}", string.clone())
        }
    }
}

impl Primitive {
    pub fn is_signed(&self) -> bool {
        match self {
            Primitive::Char | Primitive::I8 | Primitive::I16 | Primitive::F32 | Primitive::I32 | Primitive::F64 | Primitive::I64 | Primitive::I128 => true,
            _ => false
        }
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Primitive) -> bool {
        self.clone() as usize == other.clone() as usize
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &FieldType) -> bool {
        match self {
            FieldType::Empty => match other {
                FieldType::Empty => return true,
                _ => return false
            },

            FieldType::Primitive(primitive) => match other {
                FieldType::Primitive(other_primitive) => primitive == other_primitive,
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
