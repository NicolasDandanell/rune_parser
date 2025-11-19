use crate::{
    output::*,
    scanner::NumericLiteral,
    types::{DefineDefinition, DefineValue, FieldType, RedefineDefinition},
    ArraySize, RuneFileDescription, RuneParserError
};

const VEC_SIZE: usize = 0x40;

pub fn parse_define_statements(definitions: &mut Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    info!("Parsing define statements");

    let mut defines_list: Vec<DefineDefinition> = Vec::with_capacity(VEC_SIZE);
    let mut redefines_list: Vec<RedefineDefinition> = Vec::with_capacity(VEC_SIZE);

    // Create a list of all user defines found across all files
    for file in definitions.clone() {
        for definition in &file.definitions.defines {
            defines_list.push(definition.clone());
        }

        for redefinition in &file.definitions.redefines {
            redefines_list.push(redefinition.clone());
        }
    }

    // Check for duplicates
    // —————————————————————

    // Check for multiple definitions of the same define. Only necessary if more than one item in the list
    if defines_list.len() > 1 {
        for i in 0..(defines_list.len() - 1) {
            for definition in &defines_list[(i + 1)..] {
                if defines_list[i].name == definition.name {
                    error!("Found duplicate definition of {0}. Aborting parsing.", defines_list[i].name);
                    return Err(RuneParserError::MultipleDefinitions);
                }
            }
        }
    }

    // Check for multiple definitions of the same redefine. Only necessary if more than one item in the list
    if redefines_list.len() > 1 {
        for i in 0..(redefines_list.len() - 1) {
            for redefinition in &redefines_list[(i + 1)..] {
                if redefines_list[i].name == redefinition.name {
                    error!("Multiple redefinitions of {0}! Only a single redefinition of a define is supported.", redefines_list[i].name);
                    return Err(RuneParserError::MultipleRedefinitions);
                }
            }
        }
    }

    // Process files
    // ——————————————

    for file in definitions {
        // Find all definitions in the file, and check to see if there is any redefinition for it
        for define_definition in &mut file.definitions.defines {
            for i in 0..redefines_list.len() {
                // Check to see if names match
                if define_definition.name == redefines_list[i].name {
                    // Add redefinition to the define
                    define_definition.redefinition = Some(redefines_list[i].clone());

                    // Remove from redefines_list so we can check for orphan redefinitions after processing al files
                    redefines_list.swap_remove(i);
                }
            }
        }

        // So far, array sizes are the only valid place to use define values inside Rune itself
        // Check all message fields for array members, and check if their size is defined by a UserDefinition
        for message_definition in &mut file.definitions.messages {
            // Check all message members
            for field in &mut message_definition.fields {
                // Check if field type is array
                if let FieldType::Array(array) = &mut field.data_type {
                    // Check to see if the array size is a user defined value
                    if let ArraySize::UserDefinition(definition) = &mut array.element_count {
                        // Find define value
                        for user_define in &defines_list {
                            // Match with identifier string
                            if user_define.name == definition.name {
                                // Check for redefinition
                                let define_value: &DefineValue = match &user_define.redefinition {
                                    None => &user_define.value,
                                    Some(redefine) => &redefine.value
                                };

                                // Parse the value. Only integer values are valid
                                match define_value {
                                    DefineValue::NumericLiteral(value) => match value {
                                        NumericLiteral::PositiveInteger(_, _) => definition.value = DefineValue::NumericLiteral(value.clone()),
                                        _ => {
                                            error!("Could not parse {0} into a valid positive integer value!", definition.name);
                                            return Err(RuneParserError::InvalidNumericValue);
                                        }
                                    },
                                    _ => {
                                        error!("Could not parse {0} into a valid positive integer value!", definition.name);
                                        return Err(RuneParserError::InvalidNumericValue);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for orphan_redefinition in redefines_list {
        warning!("Define statement for redefinition {0} not found, so it will thus be ignored and do nothing.", orphan_redefinition.name);
    }

    Ok(())
}
