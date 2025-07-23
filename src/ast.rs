// use crate::scanner::Spanned;

/// Top Level Struct containing all message definitions in a compilation unit (file + includes)
#[derive(Debug, Clone)]
pub struct Definitions {
    pub includes: Vec<IncludeDefinition>,
    pub structs: Vec<StructDefinition>,
    pub enums: Vec<EnumDefinition>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            includes: Default::default(),
            structs: Default::default(),
            enums: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncludeDefinition {
    // pub path: String, --> Implement later if needed
    pub file: String
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub members: Vec<StructMember>,
    pub comment: Option<String>,
}

///
/// /* $comment */
/// $ident: $field_type = $field_slot;
#[derive(Debug, Clone)]
pub struct StructMember {
    pub ident: String,
    pub field_type: FieldType,
    pub field_slot: FieldSlot,
    pub comment: Option<String>,
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

    Array(Box<FieldType>, usize),
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
    FloatLiteral(f64),
}
