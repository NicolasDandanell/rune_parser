use std::fmt::{Debug, Formatter};

use crate::{
    output::*,
    types::{Array, Primitive, StandaloneCommentDefinition, UserDefinitionLink},
    RuneParserError
};

#[derive(Debug, Clone)]
pub struct MessageDefinition {
    /// Name of the struct
    pub name:             String,
    /// Data fields of the message
    pub fields:           Vec<MessageField>,
    /// Indexes that are reserved, and should not be used
    pub reserved_indexes: Vec<FieldIndex>,
    /// Comment describing the message
    pub comment:          Option<String>,
    /// Loose comments inside the message declaration
    pub orphan_comments:  Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub struct MessageField {
    /// Name of the data field
    pub identifier: String,
    /// Type of the data field
    pub data_type:  FieldType,
    /// Index of the data field
    pub index:      FieldIndex,
    /// Comment describing the data field
    pub comment:    Option<String>
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

#[derive(Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,
    Primitive(Primitive),
    Array(Array),
    UserDefined(String, UserDefinitionLink)
}

impl Debug for FieldType {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
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
            FieldType::Array(array) => write!(formatter, "[{0:?}; {1}]", array.data_type, array.element_count),
            FieldType::UserDefined(string, _) => write!(formatter, "{0}", string.clone())
        }
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &FieldType) -> bool {
        match self {
            FieldType::Empty => matches!(other, FieldType::Empty),

            FieldType::Primitive(primitive) => match other {
                FieldType::Primitive(other_primitive) => primitive == other_primitive,
                _ => false
            },

            FieldType::Array(array) => match other {
                FieldType::Array(other_array) => array == other_array,
                _ => false
            },

            FieldType::UserDefined(string, _) => match other {
                FieldType::UserDefined(other_string, _) => string == other_string,
                _ => false
            }
        }
    }
}

fn optimal_encoded_data_size(size: &u64) -> Result<u64, RuneParserError> {
    const HEADER_SIZE: u64 = 1;
    const ARRAY_SIZE_U8: u64 = 1;
    const ARRAY_SIZE_U16: u64 = 2;
    const ARRAY_SIZE_U32: u64 = 4;

    match size {
        0 => Ok(0),
        1 | 2 | 4 | 8 => Ok(HEADER_SIZE + size),
        size if Primitive::U8_RANGE.contains(size) => Ok(HEADER_SIZE + ARRAY_SIZE_U8 + size),
        size if Primitive::U16_RANGE.contains(size) => Ok(HEADER_SIZE + ARRAY_SIZE_U16 + size),
        size if Primitive::U32_RANGE.contains(size) => Ok(HEADER_SIZE + ARRAY_SIZE_U32 + size),
        _ => {
            error!(
                "Encoded size {0} of element is larger than the allowed limit of u32 max value {1}. This should not happen!",
                size,
                u32::MAX
            );
            Err(RuneParserError::InvalidEncodedSize)
        }
    }
}

impl MessageField {
    /// Gives the full encoded data size of the field. If it's a message, then the flag will determine whether optimal encoding is used, or worst case encoding
    pub fn full_encoded_size(&self, worst_case: bool) -> Result<Option<u64>, RuneParserError> {
        match &self.data_type {
            FieldType::Array(array) => Ok(Some(array.byte_size()?)),
            FieldType::Empty => Ok(Some(0)),
            FieldType::Primitive(primitive) => Ok(Some(primitive.encoded_max_data_size())),
            FieldType::UserDefined(type_identifier, definition_link) => match &definition_link {
                UserDefinitionLink::NoLink => {
                    error!("No definition for message field {0} of type {1}! This should not happen!", self.identifier, type_identifier);
                    Err(RuneParserError::UndefinedIdentifier)
                },
                UserDefinitionLink::BitfieldLink(bitfield_definition) => Ok(Some(bitfield_definition.backing_type.encoded_max_data_size())),
                UserDefinitionLink::EnumLink(enum_definition) => Ok(Some(enum_definition.backing_type.encoded_max_data_size())),
                UserDefinitionLink::MessageLink(message_link) => match worst_case {
                    false => Ok(Some(message_link.optimal_full_encoded_size()?)),
                    true => message_link.worst_case_encoded_size()
                },
                UserDefinitionLink::StructLink(struct_definition) => Ok(Some(struct_definition.flat_size()?))
            }
        }
    }
}

impl MessageDefinition {
    /// Gives the encoded size of this message if all non-skipped fields have encoded to their nominal size in the most efficient manner possible. Used for allocating buffers.
    pub fn optimal_full_encoded_size(&self) -> Result<u64, RuneParserError> {
        let mut total_size: u64 = 0;

        for field in &self.fields {
            match field.full_encoded_size(false) {
                // Not setting the worst_case flag will mean optimal_encoded_data_size() never returns None, and we can thus safely unwrap the value
                Ok(value) => total_size += optimal_encoded_data_size(&value.unwrap())?,
                Err(error) => {
                    error!("Could not get encoded size of field {0} of message {1}. Got error {2:?}", field.identifier, self.name, error);
                    return Err(error);
                }
            }
        }

        Ok(total_size)
    }

    /// If there are no skipped field indexes, then this gives the largest possible encoding of the present fields will full data. Used for allocation of buffers in worst case scenarios where another implementation might not use the most efficient encoding.
    /// This returns nothing in case there are skipped fields, as there is no way of knowing if they might be sent, and how big they are
    pub fn worst_case_encoded_size(&self) -> Result<Option<u64>, RuneParserError> {
        let mut total_size: u64 = 0;

        let mut largest_index: u64 = 0;

        // Get largest index
        for field in &self.fields {
            if field.index.value() > largest_index {
                largest_index = field.index.value();
            }
        }

        // Encoding as a large array (header + 4 byte size) is the one with the largest overhead, and thus the worst case
        const WORST_CASE_ENCODING: u64 = 5;

        for i in 0..(largest_index + 1) {
            let mut found_field: bool = false;

            for field in &self.fields {
                if field.index.value() == i {
                    total_size += match field.full_encoded_size(true)? {
                        Some(value) => WORST_CASE_ENCODING + value,
                        // Field was a sub-message with a skipped field, and we thus cannot calculate a worst case size
                        None => return Ok(None)
                    };
                    found_field = true;
                    break;
                }
            }

            // If we have not found a field with the index, then it's skipped, and we thus cannot calculate a worst case size
            if !found_field {
                return Ok(None);
            }
        }

        Ok(Some(total_size))
    }
}
