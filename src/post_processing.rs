use crate::{
    types::{
        ArraySize, BitfieldDefinition, DefineDefinition, DefineValue, EnumDefinition, FieldType, IncludeDefinition,
        RedefineDefinition, StructDefinition, UserDefinitionLink
    },
    RuneFileDescription
};

const VEC_SIZE: usize = 0x40;

pub fn parse_extensions(definitions: &mut Vec<RuneFileDescription>, append_definitions: bool) {
    println!("Parsing extensions");

    // Create a list of all extensions found across all files
    // ———————————————————————————————————————————————————————

    let mut bitfield_extensions: Vec<BitfieldExtension> = Vec::with_capacity(VEC_SIZE);
    let mut enum_extensions: Vec<EnumExtension> = Vec::with_capacity(VEC_SIZE);
    let mut struct_extensions: Vec<StructExtension> = Vec::with_capacity(VEC_SIZE);

    for file_index in 0..definitions.len() {
        // Check if there is any definitions in this file
        if !definitions[file_index].definitions.extensions.is_empty() {
            // Add all bitfield extensions, as well as which file they came from
            for bitfield_extension in &definitions[file_index].definitions.extensions.bitfields {
                println!(
                    "    Found extension to {0} in file {1}.rune",
                    bitfield_extension.name, definitions[file_index].file_name
                );

                let extension: BitfieldExtension = BitfieldExtension {
                    files:      Vec::from([definitions[file_index].file_name.clone()]),
                    definition: bitfield_extension.clone()
                };

                bitfield_extensions.push(extension);
            }

            // Add all enum extensions, as well as which file they came from
            for enum_extension in &definitions[file_index].definitions.extensions.enums {
                println!(
                    "    Found extension to {0} in {1}.rune",
                    enum_extension.name, definitions[file_index].file_name
                );

                let extension: EnumExtension = EnumExtension {
                    files:      Vec::from([definitions[file_index].file_name.clone()]),
                    definition: enum_extension.clone()
                };

                enum_extensions.push(extension);
            }

            // Add all struct extensions, as well as which file they came from
            for struct_extension in &definitions[file_index].definitions.extensions.structs {
                println!(
                    "    Found extension to {0} in file {1}.rune",
                    struct_extension.name, definitions[file_index].file_name
                );

                let extension: StructExtension = StructExtension {
                    files:      Vec::from([definitions[file_index].file_name.clone()]),
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
                        panic!(
                            "Two extensions of {0} do not have mismatching backing types {1} and {2}",
                            bitfield_extensions[i].definition.name,
                            bitfield_extensions[i].definition.backing_type.print(),
                            bitfield_extensions[z].definition.backing_type.print()
                        );
                    }

                    // Check every member of 'z' for duplicates in 'i'
                    for z_member in &bitfield_extensions[z].definition.members {
                        for i_member in &bitfield_extensions[i].definition.members {
                            if z_member.ident == i_member.ident {
                                panic!(
                                    "Collision between two {0} extensions at index {1}",
                                    bitfield_extensions[i].definition.name, z_member.ident
                                );
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
                        panic!(
                            "Two extensions of {0} do not have mismatching backing types {1} and {2}",
                            enum_extensions[i].definition.name,
                            enum_extensions[i].definition.backing_type.print(),
                            enum_extensions[z].definition.backing_type.print()
                        );
                    }

                    // Check every member of 'z' for duplicates in 'i'
                    for z_member in &enum_extensions[z].definition.members {
                        for i_member in &enum_extensions[i].definition.members {
                            if z_member.ident == i_member.ident {
                                panic!(
                                    "Collision between two {0} extensions at index {1}",
                                    enum_extensions[i].definition.name, z_member.ident
                                );
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
                            if z_member.ident == i_member.ident {
                                panic!(
                                    "Collision between two {0} extensions at index {1}",
                                    struct_extensions[i].definition.name, z_member.ident
                                );
                            }
                        }
                    }

                    // Copy all origin files of 'z' to 'i'
                    let mut z_files_copy = struct_extensions[z].files.clone();
                    struct_extensions[i].files.append(&mut z_files_copy);

                    // Copy all members of 'z' to 'i'
                    let mut z_member_list_copy = struct_extensions[z].definition.members.clone();
                    struct_extensions[i].definition.members.append(&mut z_member_list_copy);

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

    // Check the extensions with the original definition, and append them if there are no collisions
    // ——————————————————————————————————————————————————————————————————————————————————————————————

    if append_definitions {
        // Append Bitfields
        for extension in bitfield_extensions {
            // Find original definition
            for i in 0..definitions.len() {
                let file = &mut definitions[i];

                for bitfield_definition in &mut file.definitions.bitfields {
                    if bitfield_definition.name == extension.definition.name {
                        // Check that backing types match
                        if bitfield_definition.backing_type != extension.definition.backing_type {
                            panic!(
                                "Extension to {0} has wrong backing type {1} instead of original type {2}",
                                bitfield_definition.name,
                                extension.definition.backing_type.print(),
                                bitfield_definition.backing_type.print()
                            );
                        }

                        // Check for collisions
                        for extension_member in &extension.definition.members {
                            for definition_member in &bitfield_definition.members {
                                if extension_member.ident == definition_member.ident {
                                    panic!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        bitfield_definition.name, definition_member.ident
                                    );
                                }
                            }
                        }

                        // Add extension to definition
                        bitfield_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition {
                                file: include_file.clone()
                            });
                        }
                    }
                }
            }
        }

        // Append Enums
        for extension in enum_extensions {
            // Find original definition
            for i in 0..definitions.len() {
                let file = &mut definitions[i];

                for enum_definition in &mut file.definitions.enums {
                    if enum_definition.name == extension.definition.name {
                        // Check that backing types match
                        if enum_definition.backing_type != extension.definition.backing_type {
                            panic!(
                                "Extension to {0} has wrong backing type {1} instead of original type {2}",
                                enum_definition.name,
                                extension.definition.backing_type.print(),
                                enum_definition.backing_type.print()
                            );
                        }

                        // Check for collisions
                        for extension_member in &extension.definition.members {
                            for definition_member in &enum_definition.members {
                                if extension_member.ident == definition_member.ident {
                                    panic!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        enum_definition.name, definition_member.ident
                                    );
                                }
                            }
                        }

                        // Add extension to definition
                        enum_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition {
                                file: include_file.clone()
                            });
                        }
                    }
                }
            }
        }

        // Append Structs
        for extension in struct_extensions {
            // Find original definition
            for i in 0..definitions.len() {
                let file = &mut definitions[i];

                for struct_definition in &mut file.definitions.structs {
                    if struct_definition.name == extension.definition.name {
                        // Check for collisions
                        for extension_member in &extension.definition.members {
                            for definition_member in &struct_definition.members {
                                if extension_member.ident == definition_member.ident {
                                    panic!(
                                        "Collision between original {0} definition and extension at index {1}",
                                        struct_definition.name, definition_member.ident
                                    );
                                }
                            }
                        }

                        // Add extension to definition
                        struct_definition.members.append(&mut extension.definition.members.clone());

                        // Add files as inclusions
                        for include_file in &extension.files {
                            file.definitions.includes.push(IncludeDefinition {
                                file: include_file.clone()
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn parse_define_statements(definitions: &mut Vec<RuneFileDescription>) {
    println!("Parsing define statements");

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
                if defines_list[i].identifier == definition.identifier {
                    panic!(
                        "Found duplicate definition of {0}. Aborting parsing.",
                        defines_list[i].identifier
                    );
                }
            }
        }
    }

    // Check for multiple definitions of the same redefine. Only necessary if more than one item in the list
    if redefines_list.len() > 1 {
        for i in 0..(redefines_list.len() - 1) {
            for redefinition in &redefines_list[(i + 1)..] {
                if redefines_list[i].identifier == redefinition.identifier {
                    panic!(
                        "Multiple redefinitions of {0}! Only a single redefinition of a define is supported.",
                        redefines_list[i].identifier
                    );
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
                // Check to see if identifiers match
                if define_definition.identifier == redefines_list[i].identifier {
                    // Add redefinition to the define
                    define_definition.redefinition = Some(redefines_list[i].clone());

                    // Remove from redefines_list so we can check for orphan redefinitions after processing al files
                    redefines_list.swap_remove(i);
                }
            }
        }

        // So far, array sizes are the only valid place to use define values inside Rune itself
        // Check all struct fields for array members, and check if their size is defined by a UserDefinition
        for struct_definition in &mut file.definitions.structs {
            // Check all struct members
            for member in &mut struct_definition.members {
                // Check if field type is array
                match &mut member.field_type {
                    FieldType::Array(_, array_size) => {
                        // Check if array size is a define value
                        match array_size {
                            ArraySize::UserDefinition(definition) => {
                                // Find define value
                                for user_define in &defines_list {
                                    // Match with identifier string
                                    if user_define.identifier == definition.identifier {
                                        // Check for redefinition
                                        let define_value: &DefineValue = match &user_define.redefinition {
                                            None => &user_define.value,
                                            Some(redefine) => &redefine.value
                                        };

                                        // Parse the value. Only integer values are valid
                                        match define_value {
                                            DefineValue::DecimalLiteral(value) => definition.value = DefineValue::DecimalLiteral(*value),
                                            DefineValue::HexLiteral(value) => definition.value = DefineValue::HexLiteral(*value),
                                            _ => panic!("Could not parse {0} into a valid integer value!", definition.identifier)
                                        }
                                    }
                                }
                            },
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }
        }
    }

    for orphan_redefinition in redefines_list {
        println!(
            "Warning: Define statement for redefinition {0} not found, so it will thus be ignored and do nothing.",
            orphan_redefinition.identifier
        );
    }
}

pub fn link_user_definitions(definitions: &mut Vec<RuneFileDescription>) {
    println!("Linking user definitions");

    let immutable_reference = definitions.clone();

    // Find every struct member with the type UserDefinition, and add a link to its name and link to the list
    for file in definitions {
        // Check all structs
        for struct_definition in &mut file.definitions.structs {
            // Check all struct members
            for member in &mut struct_definition.members {
                // Check if struct member is user defined
                match &member.field_type {
                    FieldType::UserDefined(name) => {
                        member.user_definition_link = find_definition(name, &immutable_reference);
                    },
                    _ => ()
                }
            }
        }
    }
}

fn find_definition(identifier: &String, definitions: &Vec<RuneFileDescription>) -> UserDefinitionLink {
    // Then find the struct or enum with the corresponding name, and link to it

    for file in definitions {
        // Check if a bitfield's name matches the identifier
        for bitfield_definition in &file.definitions.bitfields {
            // Check if bitfield matches the identifier
            if identifier == bitfield_definition.name.as_str() {
                return UserDefinitionLink::BitfieldLink(bitfield_definition.clone());
            }
        }

        // Check if an enum's name matches the identifier
        for enum_definition in &file.definitions.enums {
            // Check if enum matches the identifier
            if identifier == enum_definition.name.as_str() {
                return UserDefinitionLink::EnumLink(enum_definition.clone());
            }
        }

        // Check if a structs's name matches the identifier
        for struct_definition in &file.definitions.structs {
            // Check if struct matches the identifier
            if identifier == struct_definition.name.as_str() {
                // !!! Using defines as array sizes might also require work here !!!

                let mut definition_copy = struct_definition.clone();

                // Call recursively if struct found contains user defined members
                for member in &mut definition_copy.members {
                    match &member.field_type {
                        FieldType::UserDefined(name) => {
                            // Since we return a copy, we can easily modify the definition_copy without issue
                            member.user_definition_link = find_definition(&name, definitions)
                        },
                        _ => ()
                    }
                }

                return UserDefinitionLink::StructLink(definition_copy.clone());
            }
        }
    }

    panic!("Found no user definition for identifier '{0}'!", identifier);
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

struct StructExtension {
    files:      Vec<String>,
    definition: StructDefinition
}
