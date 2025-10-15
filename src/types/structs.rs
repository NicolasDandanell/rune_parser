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
    // Clone value of the bitfield definition
    BitfieldLink(BitfieldDefinition),
    // Clone value of the enum definition
    EnumLink(EnumDefinition),
    // Clone value of the struct definition
    StructLink(StructDefinition)
}

#[derive(Debug, Clone)]
pub enum ArraySize {
    NumericValue(usize),
    UserDefinition(DefineDefinition)
}

#[derive(Debug, Clone)]
pub enum FieldSlot {
    /// Used for regular fields
    NamedSlot(usize),

    /// Used for lone comments
    NoSlot,

    /// Used for the verification field. Aliases to 0
    VerificationField,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    /// Used for skipped fields
    Empty,

    /// Used to reserve the index for Verification Fields
    VerificationReserve,

    /// Used for adding lone comments into the struct definition
    OrphanComment,

    // 1 byte primitives
    Boolean,
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
    UserDefined(String),
}
