use crate::types::{ ArraySize, DefineDefinition, DefineValue, FieldType, RedefineDefinition, UserDefinitionLink };
use crate::RuneFileDescription;

pub fn parse_define_statements(definitions: &mut Vec<RuneFileDescription>) {
    println!("Parsing define statements");

    let mut defines_list: Vec<DefineDefinition>     = Vec::with_capacity(0x40);
    let mut redefines_list: Vec<RedefineDefinition> = Vec::with_capacity(0x40);

    // Create a list of all user defines, and store in a list
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
                    panic!("Found duplicate definition of {0}. Aborting parsing.", defines_list[i].identifier);
                }
            }
        }
    }

    // Check for multiple definitions of the same redefine. Only necessary if more than one item in the list
    if redefines_list.len() > 1 {
        for i in 0..(redefines_list.len() - 1) {
            for redefinition in &redefines_list[(i + 1)..] {
                if redefines_list[i].identifier == redefinition.identifier {
                    panic!("Multiple redefinitions of {0}! Only a single redefinition of a define is supported.", redefines_list[i].identifier);
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
                                        // Parse the value. Only integer values are valid
                                        match user_define.value {
                                            DefineValue::DecimalLiteral(value) => definition.value = DefineValue::DecimalLiteral(value),
                                            DefineValue::HexLiteral(value)     => definition.value = DefineValue::HexLiteral(value),
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
        println!("Warning: Define statement for redefinition {0} not found, so it will thus be ignored and do nothing.", orphan_redefinition.identifier);
    }

}

pub fn link_user_definitions(definitions: &mut Vec<RuneFileDescription>) {
    println!("Linking user definitions");

    // Room for 64 user definitions should be plenty to begin with
    // let mut user_definition_list: Vec<(&String, &mut UserDefinitionLink)> = Vec::with_capacity(0x40);

    // Save indexes instead ??? The i, j, z values below...

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
                return UserDefinitionLink::BitfieldLink(bitfield_definition.clone())
            }
        }

        // Check if an enum's name matches the identifier
        for enum_definition in &file.definitions.enums {

            // Check if enum matches the identifier
            if identifier == enum_definition.name.as_str() {
                return UserDefinitionLink::EnumLink(enum_definition.clone())
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
                        }
                        _ => ()
                    }
                }

                return UserDefinitionLink::StructLink(definition_copy.clone())
            }
        }
    }

    panic!("Found no user definition for identifier '{0}'!", identifier);
}
