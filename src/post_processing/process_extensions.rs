use crate::{
    output::*,
    types::{BitfieldDefinition, EnumDefinition, IncludeDefinition, MessageDefinition, StructDefinition},
    RuneFileDescription, RuneParserError
};

const VEC_SIZE: usize = 0x40;

pub fn parse_extensions(definitions: &mut Vec<RuneFileDescription>, append_definitions: bool) -> Result<(), RuneParserError> {
    info!("Parsing extensions");

    // Create a list of all extensions found across all files
    // ———————————————————————————————————————————————————————

    let mut bitfield_extensions: Vec<BitfieldExtension> = Vec::with_capacity(VEC_SIZE);
    let mut enum_extensions: Vec<EnumExtension> = Vec::with_capacity(VEC_SIZE);
    let mut message_extensions: Vec<MessageExtension> = Vec::with_capacity(VEC_SIZE);
    let mut struct_extensions: Vec<StructExtension> = Vec::with_capacity(VEC_SIZE);

    for file in &mut *definitions {
        // Check if there is any definitions in this file
        if !file.definitions.extensions.is_empty() {
            // Add all bitfield extensions, as well as which file they came from
            for bitfield_extension in &file.definitions.extensions.bitfields {
                info!("    Found extension to {0} in file {1}.rune", bitfield_extension.name, file.name);

                let extension: BitfieldExtension = BitfieldExtension {
                    files:      Vec::from([file.name.clone()]),
                    definition: bitfield_extension.clone()
                };

                bitfield_extensions.push(extension);
            }

            // Add all enum extensions, as well as which file they came from
            for enum_extension in &file.definitions.extensions.enums {
                info!("    Found extension to {0} in {1}.rune", enum_extension.name, file.name);

                let extension: EnumExtension = EnumExtension {
                    files:      Vec::from([file.name.clone()]),
                    definition: enum_extension.clone()
                };

                enum_extensions.push(extension);
            }

            // Add all message extensions, as well as which file they came from
            for message_extension in &file.definitions.extensions.messages {
                info!("    Found extension to {0} in file {1}.rune", message_extension.name, file.name);

                let extension: MessageExtension = MessageExtension {
                    files:      Vec::from([file.name.clone()]),
                    definition: message_extension.clone()
                };

                message_extensions.push(extension);
            }

            // Add all struct extensions, as well as which file they came from
            for struct_extension in &file.definitions.extensions.structs {
                info!("    Found extension to {0} in file {1}.rune", struct_extension.name, file.name);

                let extension: StructExtension = StructExtension {
                    files:      Vec::from([file.name.clone()]),
                    definition: struct_extension.clone()
                };

                struct_extensions.push(extension);
            }
        }
    }

    // Check the extensions for collisions between two extensions for the same item, and merge them if there are no collisions
    // ————————————————————————————————————————————————————————————————————————————————————————————————————————————————————————

    // Check Bitfields
    if bitfield_extensions.len() > 1 {
        let mut i: usize = 0;
        let mut list_size: usize = bitfield_extensions.len();

        while i < list_size - 1 {
            let mut z = i + 1;
            while z < list_size {
                // Merge extensions of the same bitfield if there is no collision between them
                if bitfield_extensions[i].definition.name == bitfield_extensions[z].definition.name {
                    // Check that backing types match
                    if bitfield_extensions[i].definition.backing_type != bitfield_extensions[z].definition.backing_type {
                        error!(
                            "Two extensions of {0} have mismatching backing types {1:?} and {2:?}",
                            bitfield_extensions[i].definition.name, bitfield_extensions[i].definition.backing_type, bitfield_extensions[z].definition.backing_type
                        );
                        return Err(RuneParserError::ExtensionMismatch);
                    }

                    // Check every member of 'z' for duplicates in 'i'
                    for z_member in &bitfield_extensions[z].definition.members {
                        for i_member in &bitfield_extensions[i].definition.members {
                            if z_member.identifier == i_member.identifier {
                                error!("Collision between two {0} extensions at index {1}", bitfield_extensions[i].definition.name, z_member.identifier);
                                return Err(RuneParserError::IndexCollision);
                            }
                        }
                    }

                    // Copy all origin files of 'z' to 'i'
                    let mut z_files_copy = bitfield_extensions[z].files.clone();
                    bitfield_extensions[i].files.append(&mut z_files_copy);

                    // Copy all members of 'z' to 'i'
                    let mut z_member_list_copy = bitfield_extensions[z].definition.members.clone();
                    bitfield_extensions[i].definition.members.append(&mut z_member_list_copy);

                    // Remove index 'z' from list
                    bitfield_extensions.swap_remove(z);

                    list_size -= 1;
                } else {
                    z += 1;
                }
            }
            i += 1;
        }
    }

    // Check Enums
    if enum_extensions.len() > 1 {
        let mut i: usize = 0;
        let mut list_size: usize = enum_extensions.len();

        while i < list_size - 1 {
            let mut z = i + 1;
            while z < list_size {
                // Merge extensions of the same enum if there is no collision between them
                if enum_extensions[i].definition.name == enum_extensions[z].definition.name {
                    // Check that backing types match
                    if enum_extensions[i].definition.backing_type != enum_extensions[z].definition.backing_type {
                        error!(
                            "Two extensions of {0} have mismatching backing types {1:?} and {2:?}",
                            enum_extensions[i].definition.name, enum_extensions[i].definition.backing_type, enum_extensions[z].definition.backing_type
                        );
                        return Err(RuneParserError::ExtensionMismatch);
                    }

                    // Check every member of 'z' for duplicates in 'i'
                    for z_member in &enum_extensions[z].definition.members {
                        for i_member in &enum_extensions[i].definition.members {
                            if z_member.identifier == i_member.identifier {
                                error!("Collision between two {0} extensions at index {1}", enum_extensions[i].definition.name, z_member.identifier);
                                return Err(RuneParserError::IndexCollision);
                            }
                        }
                    }

                    // Copy all origin files of 'z' to 'i'
                    let mut z_files_copy = enum_extensions[z].files.clone();
                    enum_extensions[i].files.append(&mut z_files_copy);

                    // Copy all members of 'z' to 'i'
                    let mut z_member_list_copy = enum_extensions[z].definition.members.clone();
                    enum_extensions[i].definition.members.append(&mut z_member_list_copy);

                    // Remove index 'z' from list
                    enum_extensions.swap_remove(z);

                    list_size -= 1;
                } else {
                    z += 1;
                }
            }
            i += 1;
        }
    }

    // Check Messages
    if message_extensions.len() > 1 {
        let mut i: usize = 0;
        let mut list_size: usize = message_extensions.len();

        while i < list_size - 1 {
            let mut z = i + 1;
            while z < list_size {
                // Merge extensions of the same message if there is no collision between them
                if message_extensions[i].definition.name == message_extensions[z].definition.name {
                    // Check every field of 'z' for duplicates in 'i'
                    for z_field in &message_extensions[z].definition.fields {
                        for i_field in &message_extensions[i].definition.fields {
                            if z_field.identifier == i_field.identifier {
                                error!("Collision between two {0} extensions at index {1}", message_extensions[i].definition.name, z_field.identifier);
                                return Err(RuneParserError::IndexCollision);
                            }
                        }
                    }

                    // Copy all origin files of 'z' to 'i'
                    let mut z_files_copy = struct_extensions[z].files.clone();
                    struct_extensions[i].files.append(&mut z_files_copy);

                    // Copy all fields of 'z' to 'i'
                    let mut z_field_list_copy = struct_extensions[z].definition.members.clone();
                    struct_extensions[i].definition.members.append(&mut z_field_list_copy);

                    // Remove index 'z' from list
                    struct_extensions.swap_remove(z);

                    list_size -= 1;
                } else {
                    z += 1;
                }
            }
            i += 1;
        }
    }

    // Check Structs
    if struct_extensions.len() > 1 {
        let mut i: usize = 0;
        let mut list_size: usize = struct_extensions.len();

        while i < list_size - 1 {
            let mut z = i + 1;
            while z < list_size {
                // Merge extensions of the same struct if there is no collision between them
                if struct_extensions[i].definition.name == struct_extensions[z].definition.name {
                    // Check every member of 'z' for duplicates in 'i'
                    for z_member in &struct_extensions[z].definition.members {
                        for i_member in &struct_extensions[i].definition.members {
                            if z_member.identifier == i_member.identifier {
                                error!("Collision between two {0} extensions at index {1}", message_extensions[i].definition.name, z_member.identifier);
                                return Err(RuneParserError::IndexCollision);
                            }
                        }
                    }

                    // Copy all origin files of 'z' to 'i'
                    let mut z_files_copy = message_extensions[z].files.clone();
                    message_extensions[i].files.append(&mut z_files_copy);

                    // Copy all members of 'z' to 'i'
                    let mut z_member_list_copy = message_extensions[z].definition.fields.clone();
                    message_extensions[i].definition.fields.append(&mut z_member_list_copy);

                    // Remove index 'z' from list
                    message_extensions.swap_remove(z);

                    list_size -= 1;
                } else {
                    z += 1;
                }
            }
            i += 1;
        }
    }

    // Check the extensions with the original definition, and append them if there are no collisions
    // ——————————————————————————————————————————————————————————————————————————————————————————————

    if append_definitions {
        // Append Bitfields
        for extension in bitfield_extensions {
            // Find original definition
            for file in &mut *definitions {
                for bitfield_definition in &mut file.definitions.bitfields {
                    if bitfield_definition.name == extension.definition.name {
                        // Check that backing types match
                        if bitfield_definition.backing_type != extension.definition.backing_type {
                            error!(
                                "Extension to {0} has wrong backing type {1:?} instead of original type {2:?}",
                                bitfield_definition.name, extension.definition.backing_type, bitfield_definition.backing_type
                            );
                            return Err(RuneParserError::ExtensionMismatch);
                        }

                        // Check for collisions
                        for extension_member in &extension.definition.members {
                            for definition_member in &bitfield_definition.members {
                                if extension_member.identifier == definition_member.identifier {
                                    error!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        bitfield_definition.name, definition_member.identifier
                                    );
                                    return Err(RuneParserError::IndexCollision);
                                }
                            }
                        }

                        // Add extension to definition
                        bitfield_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition { file: include_file.clone() });
                        }
                    }
                }
            }
        }

        // Append Enums
        for extension in enum_extensions {
            // Find original definition
            for file in &mut *definitions {
                for enum_definition in &mut file.definitions.enums {
                    if enum_definition.name == extension.definition.name {
                        // Check that backing types match
                        if enum_definition.backing_type != extension.definition.backing_type {
                            error!(
                                "Extension to {0} has wrong backing type {1:?} instead of original type {2:?}",
                                enum_definition.name, extension.definition.backing_type, enum_definition.backing_type
                            );
                            return Err(RuneParserError::ExtensionMismatch);
                        }

                        // Check for collisions
                        for extension_member in &extension.definition.members {
                            for definition_member in &enum_definition.members {
                                if extension_member.identifier == definition_member.identifier {
                                    error!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        enum_definition.name, definition_member.identifier
                                    );
                                    return Err(RuneParserError::IndexCollision);
                                }
                            }
                        }

                        // Add extension to definition
                        enum_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition { file: include_file.clone() });
                        }
                    }
                }
            }
        }

        // Append Messages
        for extension in message_extensions {
            // Find original definition
            for file in &mut *definitions {
                for message_definition in &mut file.definitions.messages {
                    if message_definition.name == extension.definition.name {
                        // Check for collisions
                        for extension_field in &extension.definition.fields {
                            for definition_field in &message_definition.fields {
                                if extension_field.identifier == definition_field.identifier {
                                    error!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        message_definition.name, definition_field.identifier
                                    );
                                    return Err(RuneParserError::IndexCollision);
                                }
                            }
                        }

                        // Add extension to definition
                        message_definition.fields.append(&mut extension.definition.fields.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition { file: include_file.clone() });
                        }
                    }
                }
            }
        }

        // Append Structs
        for extension in struct_extensions {
            // Find original definition
            for file in &mut *definitions {
                for struct_definition in &mut file.definitions.structs {
                    if struct_definition.name == extension.definition.name {
                        // Check for collisions
                        for extension_field in &extension.definition.members {
                            for definition_field in &struct_definition.members {
                                if extension_field.identifier == definition_field.identifier {
                                    error!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        struct_definition.name, definition_field.identifier
                                    );
                                    return Err(RuneParserError::IndexCollision);
                                }
                            }
                        }

                        // Add extension to definition
                        struct_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition { file: include_file.clone() });
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// Utility Structs
// ————————————————

struct BitfieldExtension {
    files:      Vec<String>,
    definition: BitfieldDefinition
}

struct EnumExtension {
    files:      Vec<String>,
    definition: EnumDefinition
}

struct MessageExtension {
    files:      Vec<String>,
    definition: MessageDefinition
}

struct StructExtension {
    files:      Vec<String>,
    definition: StructDefinition
}
