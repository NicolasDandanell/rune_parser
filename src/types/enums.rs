use crate::types::{ FieldType, StandaloneCommentDefinition };

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: FieldType,
    pub members: Vec<EnumMember>,
    pub orphan_comments: Vec<StandaloneCommentDefinition>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub ident: String,
    pub value: EnumValue,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EnumValue {
    IntegerLiteral(i64),
    HexLiteral(u64),
    FloatLiteral(f64)
}

impl EnumValue {
    pub fn to_string(&self) -> String {
        match self {
            EnumValue::FloatLiteral(float)     => float.to_string(),
            EnumValue::IntegerLiteral(integer) => integer.to_string(),
            EnumValue::HexLiteral(hex)         => format!("0x{0:02X}", hex)
        }
    }
}
