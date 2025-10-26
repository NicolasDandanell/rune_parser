use crate::types::{FieldType, StandaloneCommentDefinition};

#[derive(Debug, Clone)]
pub struct BitfieldDefinition {
    /// Name of the bitfield
    pub name:             String,
    /// The primitive backing type of the bitfield. Only integers are valid
    pub backing_type:     FieldType,
    /// Members of the bitfield
    pub members:          Vec<BitfieldMember>,
    /// Indexes that are reserved, and should not be used
    pub reserved_indexes: Vec<u64>,
    /// Comment describing the bitfield
    pub comment:          Option<String>,
    /// Loose comments inside the bitfield declaration
    pub orphan_comments:  Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
/// Describes the size of the bit field, and whether it's signed or not
pub enum BitSize {
    Signed(u64),
    Unsigned(u64)
}

impl BitSize {
    pub const LIMIT: u64 = 64;

    pub fn absolute(&self) -> u64 {
        match self {
            BitSize::Signed(size) => *size,
            BitSize::Unsigned(size) => *size
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitfieldMember {
    /// Name of the bit field
    pub identifier: String,
    /// Size of the bit field
    pub size:       BitSize,
    /// Index of the bit field
    pub index:      u64,
    /// Comment describing the bit field
    pub comment:    Option<String>
}
