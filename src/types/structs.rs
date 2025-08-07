use crate::types::{ BitfieldDefinition, DefineDefinition, EnumDefinition };

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name:    String,
    pub members: Vec<StructMember>,
    pub comment: Option<String>,
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
    // Copy value of the bitfield defintion
    BitfieldLink(BitfieldDefinition),
    // Copy value of the enum definition
    EnumLink(EnumDefinition),
    // Copy value of the struct definition
    StructLink(StructDefinition)
}

#[derive(Debug, Clone)]
pub enum ArraySize {
    NumericValue(usize),
    UserDefinition(DefineDefinition)
}

#[derive(Debug, Clone)]
pub enum FieldSlot {
    NamedSlot(u64),
    VerificationField,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Boolean,
    UByte,
    Byte,

    UShort,
    Short,

    Float,
    UInt,
    Int,

    Double,
    ULong,
    Long,

    Array(Box<FieldType>, ArraySize),

    UserDefined(String),
}
