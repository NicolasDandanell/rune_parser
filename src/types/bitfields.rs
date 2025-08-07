use crate::types::FieldType;

#[derive(Debug, Clone)]
pub struct BitfieldDefinition {
    pub name:         String,
    pub backing_type: FieldType,
    pub members:      Vec<BitfieldMember>,
    pub comment:      Option<String>,
}

#[derive(Debug, Clone)]
pub struct BitfieldMember {
    pub ident:    String,
    pub bit_size: usize,
    pub bit_slot: usize,
    pub comment:  Option<String>
}
