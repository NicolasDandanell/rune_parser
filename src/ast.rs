// use crate::scanner::Spanned;

/// Top Level Struct containing all message definitions in a compilation unit (file + includes)
#[derive(Debug, Clone)]
pub struct Definitions {
    pub defines: Vec<DefineDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub includes: Vec<IncludeDefinition>,
    pub structs: Vec<StructDefinition>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            defines: Default::default(),
            enums: Default::default(),
            includes: Default::default(),
            structs: Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum DefineValue {
    NoValue,
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Composite(String)
}

#[derive(Debug, Clone)]
pub struct DefineDefinition {
    pub identifier: String,
    pub value:      DefineValue,
    pub comment:    Option<String>
}

#[derive(Debug, Clone)]
pub struct IncludeDefinition {
    // pub path: String, --> Implement later if needed
    pub file: String
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name:    String,
    pub members: Vec<StructMember>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum UserDefinitionLink {
    NoLink,
    // Copy value of the enum definition
    EnumLink(EnumDefinition),
    // Copy value of the struct definition
    StructLink(StructDefinition)
}

///
/// /* $comment */
/// $ident: $field_type = $field_slot;
#[derive(Debug, Clone)]
pub struct StructMember {
    pub ident: String,
    pub field_type: FieldType,
    pub field_slot: FieldSlot,
    pub user_definition_link: UserDefinitionLink,
    pub comment: Option<String>
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

///
/// /* $comment */
/// enum $name: $backing_type { (members)* }
///
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: FieldType,
    pub members: Vec<EnumMember>,
    pub comment: Option<String>,
}

///
/// /* $comment */
/// $ident = $value
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub ident: String,
    pub value: EnumValue,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EnumValue {
    IntegerLiteral(i64),
    FloatLiteral(f64)
}

impl EnumValue {
    pub fn to_string(&self) -> String{
        match self {
            EnumValue::FloatLiteral(float)     => float.to_string(),
            EnumValue::IntegerLiteral(integer) => integer.to_string()
        }
    }
}
