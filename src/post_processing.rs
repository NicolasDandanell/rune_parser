use crate::ast::{ ArraySize, FieldType, DefineDefinition, DefineValue, UserDefinitionLink };
use crate::RuneFileDescription;

pub fn parse_define_statements(definitions: &mut Vec<RuneFileDescription>) {
    println!("Parsing define statements");
    println!("——————————————————————————");
    println!("");

    let mut defines_list: Vec<DefineDefinition> = Vec::with_capacity(0x40);

    // Create a list of all user defines, and store in a list
    for file in definitions.clone() {
        for definition in &file.definitions.defines {
            defines_list.push(definition.clone());
        }
    }

    // So far, array sizes are the only valid place to use define values
    // Check all struct fields for array members, and check if their size is defined by a UserDefinition
    for file in definitions {

        // Check all structs
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
                                            DefineValue::IntegerLiteral(value) => definition.value = DefineValue::IntegerLiteral(value),
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
}

pub fn link_user_definitions(definitions: &mut Vec<RuneFileDescription>) {

    println!("Linking user definitions");
    println!("—————————————————————————");
    println!("");


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
                        println!("Found user defined type '{0}'", name);

                        member.user_definition_link = find_definition(name, &immutable_reference);

                        println!("");
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
        // Find an enum matches one of the entries on the list
        for enum_definition in &file.definitions.enums {

            // Check if enum matches the identifier
            if identifier == enum_definition.name.as_str() {
                println!("    Found enum match for definition '{0}'", identifier);
                return UserDefinitionLink::EnumLink(enum_definition.clone())
            }
        }

        // Find a struct that matches one of the entries on the list
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

                println!("    Found struct match for definition '{0}'", identifier);
                return UserDefinitionLink::StructLink(definition_copy.clone())
            }
        }
    }

    println!("    Found no user definition for identifier '{0}'!", identifier);
    return UserDefinitionLink::NoLink
}
