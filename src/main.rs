pub mod ast;
pub mod languages;
pub mod parser;
pub mod scanner;

use ast::{ UserDefinitionLink, Definitions };
use clap::Parser;
use languages::c::output_c_files;
use scanner::Scanner;
use std::{ fs::ReadDir, path::Path, process::exit };

use crate::ast::FieldType;

const ALLOCATION_SIZE: usize = 0x40;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of folder where to find Rune files
    #[arg(long, short = 'i')]
    rune_folder: String,

    /// Language of the output source code
    #[arg(long, short = 'l')]
    language: String,

    /// Path of folder where to output source code
    #[arg(long, short = 'o')]
    output_folder: String,

    /// Whether to pack (remove padding) from outputted sources
    #[arg(long, short = 'p')]
    pack: bool
}

// Supported programming languages
// ————————————————————————————————

#[derive(PartialEq)]
enum Language {
    Unsupported,
    C,
}

impl Language {
    fn from_string(string: String) -> Language {
        match string.to_ascii_lowercase().as_str() {
            "c"    => Language::C,
            "rust" => {
                println!("Rust is not implemented yet, but is planned!");
                Language::Unsupported
            },
            _   => {
                println!("Language \"{0}\" not implemented. Supported languages are:", string);
                println!(" · C");
                Language::Unsupported
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuneFileDescription {
    pub relative_path: String,
    pub file_name:     String,
    pub definitions:   Definitions
}

// Functions
// ——————————

fn get_rune_files(folder_path: &Path, mut rune_file_list: &mut Vec<String>) {

    let folder_iterator: ReadDir = match folder_path.read_dir() {
        Err(error)  => panic!("Could not read \"{0}\" directory. Got error {1}", folder_path.as_os_str().to_str().unwrap(), error),
        Ok(value) =>  value
    };

    for item in folder_iterator {
        match item {
            Err(error) => println!("Got an error {0} in one of the items in \"{1}\" directory", error, folder_path.as_os_str().to_str().unwrap()),
            Ok(item) => {
                match item.file_type() {
                    Err(error) => println!("Got error {0} in getting file type of file \"{1}\"", error, item.file_name().to_str().unwrap()),
                    Ok(file_type) => {
                        if file_type.is_dir() {

                            // Subfolder
                            // ——————————

                            println!("Found subdirectory named {0:?}", item.file_name());

                            let subfolder_string: String = format!("{0}/{1}", folder_path.as_os_str().to_str().unwrap(), item.file_name().to_str().unwrap());

                            let subfolder_path: &Path = Path::new(&subfolder_string);

                            // Recursively call function to parse files in subfolder
                            get_rune_files(subfolder_path, &mut rune_file_list);

                        } else if file_type.is_file() {

                            // Rune file
                            // ——————————

                            let file_string = item.file_name().into_string().expect("Could not parse os string!");

                            if file_string.ends_with(".rune") {
                                rune_file_list.push(format!("{0}/{1}", folder_path.as_os_str().to_str().unwrap(), file_string));
                            }

                        } else {
                            /* Nothing - Ignore anything that is not a subfolder or a .rune file */
                        }
                    }
                }
            }
        };
    }
}

fn link_user_definitions(definitions: &mut Vec<RuneFileDescription>) {

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

                        // user_definition_list.push((name, &mut definitions[i].definitions.structs[j].members[z].user_definition_link))



                        member.user_definition_link = find_definition(name, &immutable_reference);

                        println!("");
                    },
                    _ => ()
                }
            }
        }
    }

    // Then find the struct or enum with the corresponding name, and link to it
    /* for file in &mut *definitions {

        // Find an enum matches one of the entries on the list
        for enum_definition in &mut file.definitions.enums {

            // Check if enum matches any identity on the list
            for list_definition in &mut user_definition_list {
                if list_definition.0.as_str() == enum_definition.name.as_str() {
                    println!("Found enum match for definition {0}", list_definition.0);
                }
            }

        }

        // Find a struct that matches one of the entries on the list
        for struct_definition in &mut file.definitions.structs {

            // Check if struct matches any identity on the list
            for list_definition in &mut user_definition_list {
                if list_definition.0.as_str() == struct_definition.name.as_str() {
                    println!("Found struct match for definition {0}", list_definition.0);
                }
            }
        }
    } */

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

fn main() -> Result<(), usize> {

    // Parse arguments
    // ————————————————

    let args: Args = Args::parse();

    let input_path: &Path         = Path::new(args.rune_folder.as_str());
    let output_path: &Path        = Path::new(args.output_folder.as_str());
    let output_language: Language = Language::from_string(args.language);
    let pack_output: bool         = args.pack;

    // Validate arguments
    // ———————————————————

    if !input_path.exists() || !input_path.is_dir() {
        if !input_path.exists() {
            println!("Input path \"{0}\" does not exist!", input_path.as_os_str().to_str().unwrap());
        } else if !input_path.is_dir() {
            println!("Input path \"{0}\" is not a directory!", input_path.as_os_str().to_str().unwrap());
        }

        return Err(1)
    }

    // Output folder
    if !output_path.exists() || !output_path.is_dir() {
        if !output_path.exists() {
            println!("Output path \"{0}\" does not exist!", output_path.as_os_str().to_str().unwrap());
        } else if !input_path.is_dir() {
            println!("Output path \"{0}\" is not a directory!", output_path.as_os_str().to_str().unwrap());
        }

        return Err(2)
    }

    // Language
    if output_language == Language::Unsupported {
        return Err(3)
    }

    // Get rune files from folder
    // ———————————————————————————

    // Create a vector with allocated space for 64 rune files, which should be more than plenty for most projects
    let mut rune_file_list: Vec<String> = Vec::with_capacity(ALLOCATION_SIZE);

    get_rune_files(input_path, &mut rune_file_list);

    if rune_file_list.is_empty() {
        println!("Could not parse any rune files from folder");
        return Err(1)
    }

    // Print all found files
    println!("\nFound the following rune files:");
    for i in 0..rune_file_list.len() {
        println!("    {0}", rune_file_list[i]);
    }
    println!("");

    // Process rune files
    // ———————————————————

    let mut definitions_list: Vec<RuneFileDescription> = Vec::with_capacity(ALLOCATION_SIZE);

    for filepath in rune_file_list {
        let file_path: &Path = Path::new(&filepath);

        let file = match std::fs::read_to_string(file_path) {
            Err(error) => panic!("Error in reading file to string. Got error {0}", error),
            Ok(path)  => path
        };

        let tokens = match Scanner::new(file.chars()).scan_all() {
            Err(e) => {
                eprintln!("Error while scanning file {}: {:#?}", filepath, e);
                exit(-1);
            }
            Ok(t) => t,
        };

        let types: Definitions = match parser::parse_tokens(&mut tokens.into_iter().peekable()) {
            Err(e) => {
                eprintln!("Error while parsing file {}: {:#?}", filepath, e);
                exit(-1);
            }
            Ok(t) => t,
        };

        // dbg!(&types);
        // println!("\n——————————————————————————————————————————————————————————————\n");

        // Get isolated file name (without .rune extension)
        let file_name: String = file_path.file_name().unwrap().to_str().unwrap().strip_suffix(".rune").unwrap().to_string();

        // Get relative path (from input path)
        let relative_path: String = filepath.strip_prefix(input_path.to_str().unwrap()).unwrap()
                                            .strip_prefix("/").unwrap()
                                            .strip_suffix(file_path.file_name().unwrap().to_str().unwrap()).unwrap().to_string();

        definitions_list.push(
            RuneFileDescription {
                relative_path: relative_path,
                file_name:     file_name,
                definitions:   types,
            }
        );
    }

    // Link all user definitions
    // ——————————————————————————

    link_user_definitions(&mut definitions_list);

    // Validate parsed data structures
    // ————————————————————————————————

    // To be implemented...

    // Create source files
    // ————————————————————

    match output_language {
        Language::C => output_c_files(definitions_list, output_path, pack_output),
        Language::Unsupported => unreachable!()
    }

    Ok(())
}
