use crate::types::{FieldType, StandaloneCommentDefinition};

#[derive(Debug, Clone)]
pub struct BitfieldDefinition {
    pub name:            String,
    pub backing_type:    FieldType,
    pub members:         Vec<BitfieldMember>,
    pub reserved_slots:  Vec<u64>,
    pub comment:         Option<String>,
    pub orphan_comments: Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub enum BitSize {
    Signed(u64),
    Unsigned(u64)
}

impl BitSize {
    pub const BIT_SLOT_LIMIT: u64 = 64;

    pub fn absolute(&self) -> u64 {
        match self {
            BitSize::Signed(size) => *size,
            BitSize::Unsigned(size) => *size
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitfieldMember {
    pub identifier: String,
    pub bit_size:   BitSize,
    pub bit_slot:   u64,
    pub comment:    Option<String>
}
