pub mod bitfields;
pub mod defines;
pub mod enums;
pub mod extensions;
pub mod includes;
pub mod standalone_comments;
pub mod structs;

pub use bitfields::{BitSize, BitfieldDefinition, BitfieldMember};
pub use defines::{DefineDefinition, DefineValue, RedefineDefinition};
pub use enums::{EnumDefinition, EnumMember, EnumValue};
pub use extensions::{ExtensionDefinition, Extensions};
pub use includes::IncludeDefinition;
pub use standalone_comments::{CommentPosition, StandaloneCommentDefinition};
pub use structs::{ArraySize, FieldSlot, FieldType, StructDefinition, StructMember, UserDefinitionLink};

/// Top Level Struct containing all message definitions in a compilation unit (file + includes)
#[derive(Debug, Clone)]
pub struct Definitions {
    pub bitfields:           Vec<BitfieldDefinition>,
    pub defines:             Vec<DefineDefinition>,
    pub redefines:           Vec<RedefineDefinition>,
    pub enums:               Vec<EnumDefinition>,
    pub extensions:          Extensions,
    pub includes:            Vec<IncludeDefinition>,
    pub standalone_comments: Vec<StandaloneCommentDefinition>,
    pub structs:             Vec<StructDefinition>
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            bitfields:           Default::default(),
            defines:             Default::default(),
            redefines:           Default::default(),
            enums:               Default::default(),
            extensions:          Default::default(),
            includes:            Default::default(),
            standalone_comments: Default::default(),
            structs:             Default::default()
        }
    }
}
