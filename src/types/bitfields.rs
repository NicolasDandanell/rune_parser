use crate::types::{FieldType, StandaloneCommentDefinition};

#[derive(Debug, Clone)]
pub struct BitfieldDefinition {
    pub name:            String,
    pub backing_type:    FieldType,
    pub members:         Vec<BitfieldMember>,
    pub reserved_slots:  Vec<usize>,
    pub comment:         Option<String>,
    pub orphan_comments: Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub enum BitSize {
    Signed(usize),
    Unsigned(usize)
}

#[derive(Debug, Clone)]
pub struct BitfieldMember {
    pub identifier: String,
    pub bit_size:   BitSize,
    pub bit_slot:   usize,
    pub comment:    Option<String>
}
