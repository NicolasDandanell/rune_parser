use crate::types::{BitfieldDefinition, EnumDefinition, MessageDefinition, StructDefinition};

#[derive(Debug, Clone)]
pub enum UserDefinitionLink {
    NoLink,
    // Clone value of the bitfield definition
    BitfieldLink(BitfieldDefinition),
    // Clone value of the enum definition
    EnumLink(EnumDefinition),
    // Clone value of the message definition
    MessageLink(MessageDefinition),
    // Clone value of the struct definition
    StructLink(StructDefinition)
}
