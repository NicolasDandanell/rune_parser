pub mod bitfields;
pub mod defines;
pub mod enums;
pub mod includes;
pub mod structs;

pub use bitfields::{ BitfieldDefinition, BitfieldMember };
pub use defines::{ DefineDefinition, DefineValue };
pub use enums::{ EnumDefinition, EnumMember, EnumValue };
pub use includes::{ IncludeDefinition };
pub use structs::{ ArraySize, FieldSlot, FieldType, StructDefinition, StructMember, UserDefinitionLink };

/// Top Level Struct containing all message definitions in a compilation unit (file + includes)
#[derive(Debug, Clone)]
pub struct Definitions {
    pub bitfields: Vec<BitfieldDefinition>,
    pub defines:   Vec<DefineDefinition>,
    pub enums:     Vec<EnumDefinition>,
    pub includes:  Vec<IncludeDefinition>,
    pub structs:   Vec<StructDefinition>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            bitfields: Default::default(),
            defines:   Default::default(),
            enums:     Default::default(),
            includes:  Default::default(),
            structs:   Default::default()
        }
    }
}
