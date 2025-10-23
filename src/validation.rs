use crate::{output::is_silent, types::FieldSlot, RuneFileDescription, RuneParserError};

/// Check that two fields do not have the same field index, considering that verifier is an alias for index 0
pub fn validate_struct_indexes(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check all files for struct definitions
    for file in files {
        for struct_definition in &file.definitions.structs {
            // Check whether a verification field has been declared
            let has_verifier: bool = match struct_definition.members.iter().filter(|&x| x.field_slot.is_verifier() == true).count() {
                0 => false,
                1 => true,
                _ => {
                    error!("Error at {0}: Cannot have more than one verifier field per struct!", struct_definition.name);
                    return Err(RuneParserError::IndexCollision);
                }
            };

            // Check all indexes for duplicates
            for i in 0..32 {
                let count = struct_definition.members.iter().filter(|&x| x.field_slot.value() == i).count();

                if count > 1 {
                    if i == 0 && has_verifier {
                        error!(
                            "Error at {0}: Cannot have a verifier field and a field with index 0! This is due to verifier being an alias for index 0",
                            struct_definition.name
                        );
                    } else {
                        error!(
                            "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                            struct_definition.name, i
                        );
                    }
                    return Err(RuneParserError::IndexCollision);
                } else if count == 1 {
                    if struct_definition.reserved_slots.contains(&FieldSlot::Numeric(i)) {
                        error!("Error at {0}: A field with index {1} is declared even though field index {1} is reserved", struct_definition.name, i);
                        return Err(RuneParserError::UseOfReservedIndex);
                    }
                }
            }
        }
    }

    return Ok(());
}
