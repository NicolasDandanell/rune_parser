use crate::types::{bitfields::BitfieldDefinition, enums::EnumDefinition, structs::StructDefinition};

pub enum ExtensionDefinition {
    Bitfield(BitfieldDefinition),
    Enum(EnumDefinition),
    Struct(StructDefinition)
}

#[derive(Debug, Clone)]
pub struct Extensions {
    pub bitfields: Vec<BitfieldDefinition>,
    pub enums:     Vec<EnumDefinition>,
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
        return self.bitfields.is_empty() && self.enums.is_empty() && self.structs.is_empty();
    }

    pub fn with_capacity(size: usize) -> Extensions {
        Extensions {
            bitfields: Vec::with_capacity(size),
            enums:     Vec::with_capacity(size),
            structs:   Vec::with_capacity(size)
        }
    }
}

impl Default for Extensions {
    fn default() -> Extensions {
        Extensions {
            bitfields: Default::default(),
            enums:     Default::default(),
            structs:   Default::default()
        }
    }
}
