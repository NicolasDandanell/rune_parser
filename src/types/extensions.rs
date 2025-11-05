use crate::types::{bitfields::BitfieldDefinition, enums::EnumDefinition, structs::StructDefinition};

/// Helper holding all three possible extension types. Used only when parsing
pub enum ExtensionDefinition {
    Bitfield(BitfieldDefinition),
    Enum(EnumDefinition),
    Struct(StructDefinition)
}

#[derive(Debug, Default, Clone)]
pub struct Extensions {
    /// List of bitfield extensions
    pub bitfields: Vec<BitfieldDefinition>,
    /// List of enum extensions
    pub enums:     Vec<EnumDefinition>,
    /// List of struct extensions
    pub structs:   Vec<StructDefinition>
}

impl Extensions {
    pub fn add_entry(&mut self, entry: ExtensionDefinition) {
        match entry {
            ExtensionDefinition::Bitfield(entry) => self.bitfields.push(entry),
            ExtensionDefinition::Enum(entry) => self.enums.push(entry),
            ExtensionDefinition::Struct(entry) => self.structs.push(entry)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bitfields.is_empty() && self.enums.is_empty() && self.structs.is_empty()
    }

    pub fn with_capacity(size: usize) -> Extensions {
        Extensions {
            bitfields: Vec::with_capacity(size),
            enums:     Vec::with_capacity(size),
            structs:   Vec::with_capacity(size)
        }
    }
}
