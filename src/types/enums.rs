use crate::{
    scanner::NumericLiteral,
    types::{FieldType, StandaloneCommentDefinition}
};

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name:            String,
    pub backing_type:    FieldType,
    pub members:         Vec<EnumMember>,
    pub reserved_values: Vec<NumericLiteral>,
    pub comment:         Option<String>,
    pub orphan_comments: Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub identifier: String,
    pub value:      NumericLiteral,
    pub comment:    Option<String>
}
