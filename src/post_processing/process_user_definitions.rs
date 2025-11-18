use crate::{
    output::*,
    types::{FieldType, MemberType, UserDefinitionLink},
    ArrayType, RuneFileDescription, RuneParserError
};

pub fn link_user_definitions(definitions: &mut Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    info!("Linking user definitions");

    let immutable_reference = definitions.clone();

    // Find every message member with the type UserDefinition, and add a link to its name and link to the list
    for file in definitions {
        // Check all messages
        for message_definition in &mut file.definitions.messages {
            // Check all message fields
            for field in &mut message_definition.fields {
                // Check if type is user defined, or array with user defined type
                match &mut field.data_type {
                    FieldType::Array(array) => {
                        if let ArrayType::UserDefined(definition_name, definition_link) = &mut array.data_type {
                            *definition_link = find_data_definition(definition_name, &immutable_reference)?;
                        }
                    },

                    FieldType::Empty => {
                        error!("Message field definition was empty! This should not happen!");
                        return Err(RuneParserError::EmptyMessageField);
                    },

                    FieldType::UserDefined(definition_name, definition_link) => {
                        *definition_link = find_field_definition(definition_name, &immutable_reference)?;
                    },

                    _ => () // Nothing
                }
            }
        }

        // Check all structs
        for struct_definition in &mut file.definitions.structs {
            // Check all struct members
            for member in &mut struct_definition.members {
                // Check if type is user defined, or array with user defined type
                match &mut member.data_type {
                    MemberType::Array(array) => {
                        if let ArrayType::UserDefined(definition_name, definition_link) = &mut array.data_type {
                            *definition_link = find_data_definition(definition_name, &immutable_reference)?;
                        }
                    },

                    MemberType::UserDefined(definition_name, definition_link) => {
                        *definition_link = find_data_definition(definition_name, &immutable_reference)?;
                    },
                    _ => () // Nothing
                }
            }
        }
    }

    Ok(())
}

fn find_data_definition(identifier: &String, definitions: &Vec<RuneFileDescription>) -> Result<UserDefinitionLink, RuneParserError> {
    // Then find the enum, field, message, struct with the corresponding name, and link to it

    for file in definitions {
        // Check if a bitfields name matches the identifier
        for bitfield_definition in &file.definitions.bitfields {
            // Check if bitfield matches the identifier
            if identifier == bitfield_definition.name.as_str() {
                return Ok(UserDefinitionLink::BitfieldLink(bitfield_definition.clone()));
            }
        }

        // Check if an enums name matches the identifier
        for enum_definition in &file.definitions.enums {
            // Check if enum matches the identifier
            if identifier == enum_definition.name.as_str() {
                return Ok(UserDefinitionLink::EnumLink(enum_definition.clone()));
            }
        }

        // Check if a structs name matches the identifier
        for struct_definition in &file.definitions.structs {
            // Check if struct matches the identifier
            if identifier == struct_definition.name.as_str() {
                let mut definition_copy = struct_definition.clone();

                // Call recursively if struct found contains user defined members
                for member in &mut definition_copy.members {
                    if let MemberType::UserDefined(definition_name, definition_link) = &mut member.data_type {
                        // Since we return a copy, we can easily modify the definition_copy without issue
                        *definition_link = find_data_definition(definition_name, definitions)?;
                    }
                }

                return Ok(UserDefinitionLink::StructLink(definition_copy.clone()));
            }
        }

        // Check messages in case a message type was used in an illegal way
        for message_definition in &file.definitions.messages {
            // Check if message matches the identifier
            if identifier == message_definition.name.as_str() {
                error!(
                    "Found a use of message type {0} being used somewhere else than a message! Messages cannot be used as array types, or as struct members!",
                    identifier
                );
                return Err(RuneParserError::InvalidTypeUse);
            }
        }
    }

    error!("Found no user definition for identifier '{0}'!", identifier);
    Err(RuneParserError::UndefinedIdentifier)
}

fn find_field_definition(identifier: &String, definitions: &Vec<RuneFileDescription>) -> Result<UserDefinitionLink, RuneParserError> {
    for file in definitions {
        // Check if a messages name matches the identifier
        for message_definition in &file.definitions.messages {
            // Check if message matches the identifier
            if identifier == message_definition.name.as_str() {
                // !!! Using defines as array sizes might also require work here !!!

                let mut definition_copy = message_definition.clone();

                // Call recursively if struct found contains user defined members
                for field in &mut definition_copy.fields {
                    if let FieldType::UserDefined(definition_name, definition_link) = &mut field.data_type {
                        // Since we return a copy, we can easily modify the definition_copy without issue
                        *definition_link = find_field_definition(definition_name, definitions)?;
                    }
                }

                return Ok(UserDefinitionLink::MessageLink(definition_copy.clone()));
            }
        }
    }

    find_data_definition(identifier, definitions)
}
