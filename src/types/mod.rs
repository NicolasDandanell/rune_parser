pub mod arrays;
pub mod bitfields;
pub mod defines;
pub mod enums;
pub mod extensions;
pub mod includes;
pub mod links;
pub mod messages;
pub mod primitives;
pub mod standalone_comments;
pub mod structs;

pub use arrays::{Array, ArraySize, ArrayType};
pub use bitfields::{BitSize, BitfieldDefinition, BitfieldMember};
pub use defines::{DefineDefinition, DefineValue, RedefineDefinition};
pub use enums::{EnumDefinition, EnumMember};
pub use extensions::{ExtensionDefinition, Extensions};
pub use includes::IncludeDefinition;
pub use links::UserDefinitionLink;
pub use messages::{FieldIndex, FieldType, MessageDefinition, MessageField};
pub use primitives::Primitive;
pub use standalone_comments::StandaloneCommentDefinition;
pub use structs::{MemberType, StructDefinition, StructMember};

/// Top Level Struct containing all message definitions in a compilation unit (file + includes)
#[derive(Debug, Default, Clone)]
pub struct Definitions {
    pub bitfields:           Vec<BitfieldDefinition>,
    pub defines:             Vec<DefineDefinition>,
    pub redefines:           Vec<RedefineDefinition>,
    pub enums:               Vec<EnumDefinition>,
    pub extensions:          Extensions,
    pub includes:            Vec<IncludeDefinition>,
    pub messages:            Vec<MessageDefinition>,
    pub standalone_comments: Vec<StandaloneCommentDefinition>,
    pub structs:             Vec<StructDefinition>
}
