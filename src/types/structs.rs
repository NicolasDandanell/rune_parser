use std::fmt::{Debug, Formatter};

use crate::{
    output::*,
    types::{Array, Primitive, StandaloneCommentDefinition, UserDefinitionLink},
    RuneParserError
};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    /// Name of the struct
    pub name:            String,
    /// Members of the struct
    pub members:         Vec<StructMember>,
    /// Comment describing the struct
    pub comment:         Option<String>,
    /// Loose comments inside the struct declaration
    pub orphan_comments: Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub struct StructMember {
    /// Name of the data field
    pub identifier: String,
    /// Type of the data field
    pub data_type:  MemberType,
    /// Index of the data field - Structs do not have a limit on indexes
    pub index:      u64,
    /// Comment describing the data field
    pub comment:    Option<String>
}

#[derive(Debug, Clone)]
pub enum MemberIndex {
    /// Used for regular fields
    Numeric(u64),

    /// Used for the verification field. Aliases to 0
    Verifier
}

#[derive(Clone)]
pub enum MemberType {
    Array(Array),
    Primitive(Primitive),

    /// If the data type of the field is a user defined one, then it will contain a copy of its definition
    UserDefined(String, UserDefinitionLink)
}

impl Debug for MemberType {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        match self {
            MemberType::Primitive(primitive) => match primitive {
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
            MemberType::Array(array) => write!(formatter, "[{0:?}; {1}]", array.data_type, array.element_count),
            MemberType::UserDefined(string, _) => write!(formatter, "{0}", string.clone())
        }
    }
}

impl PartialEq for MemberType {
    fn eq(&self, other: &MemberType) -> bool {
        match self {
            MemberType::Primitive(primitive) => match other {
                MemberType::Primitive(other_primitive) => primitive == other_primitive,
                _ => false
            },

            MemberType::Array(array) => match other {
                MemberType::Array(other_array) => array == other_array,
                _ => false
            },

            MemberType::UserDefined(string, _) => match other {
                MemberType::UserDefined(other_string, _) => string == other_string,
                _ => false
            }
        }
    }
}

impl StructDefinition {
    /// Size of struct when all members are flattened into a long data blob with no padding
    pub fn flat_size(&self) -> Result<u64, RuneParserError> {
        let mut total_size: u64 = 0;

        for member in &self.members {
            let member_size: u64 = match &member.data_type {
                MemberType::Array(array) => array.byte_size()?,
                MemberType::Primitive(primitive) => primitive.encoded_max_data_size(),
                MemberType::UserDefined(type_identifier, definition_link) => match &definition_link {
                    UserDefinitionLink::NoLink => {
                        error!(
                            "No definition for member {0} of type {1} in struct {2}! This should not happen!",
                            member.identifier, type_identifier, self.name
                        );
                        return Err(RuneParserError::UndefinedIdentifier);
                    },
                    UserDefinitionLink::BitfieldLink(bitfield_definition) => bitfield_definition.backing_type.encoded_max_data_size(),
                    UserDefinitionLink::EnumLink(enum_definition) => enum_definition.backing_type.encoded_max_data_size(),
                    UserDefinitionLink::MessageLink(message_link) => {
                        error!(
                            "Structs cannot contain message members! Member {0} of struct {1} contained message {2}",
                            member.identifier, self.name, message_link.name
                        );
                        return Err(RuneParserError::InvalidStructMemberType);
                    },
                    UserDefinitionLink::StructLink(struct_definition) => struct_definition.flat_size()?
                }
            };

            total_size += member_size;
        }

        Ok(total_size)
    }
}
