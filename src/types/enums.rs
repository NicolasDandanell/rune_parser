use crate::{
    scanner::NumericLiteral,
    types::{Primitive, StandaloneCommentDefinition}
};

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    /// Name of the enum
    pub name:            String,
    /// The primitive backing type of the enum
    pub backing_type:    Primitive,
    /// Members of the enum
    pub members:         Vec<EnumMember>,
    /// Values that are reserved, and should not be used
    pub reserved_values: Vec<NumericLiteral>,
    /// Comment describing the enum
    pub comment:         Option<String>,
    /// Loose comments inside the enum declaration
    pub orphan_comments: Vec<StandaloneCommentDefinition>
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    /// Name of the enum member
    pub identifier: String,
    /// Value of the enum member
    pub value:      NumericLiteral,
    /// Comment describing the enum member
    pub comment:    Option<String>
}
