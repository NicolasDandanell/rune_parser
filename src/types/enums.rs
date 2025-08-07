use crate::types::FieldType;

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: FieldType,
    pub members: Vec<EnumMember>,
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
