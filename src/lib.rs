#[macro_use]
pub mod output;
pub mod parser;
pub mod post_processing;
pub mod scanner;
pub mod types;
pub mod validation;

use std::{fs::ReadDir, path::Path};

use output::{enable_silent, is_silent};
use parser::parse_tokens;
use post_processing::{link_user_definitions, parse_define_statements, parse_extensions};
use scanner::Scanner;
use types::Definitions;
use validation::validate_parsed_files;

const ALLOCATION_SIZE: usize = 0x40;

#[derive(Debug, Clone)]
pub struct RuneFileDescription {
    pub relative_path: String,
    pub file_name:     String,
    pub definitions:   Definitions
}

#[derive(Debug)]
pub enum RuneParserError {
    InvalidInputPath,
    InvalidFilePath,
    FileSystemError,
    IdentifierCollision,
    IndexCollision,
    NameCollision,
    ValueCollision,
    InvalidTotalBitfieldSize,
    UseOfReservedIndex,
    ExtensionMismatch,
    UndefinedIdentifier,
    MultipleDefinitions,
    MultipleRedefinitions,
    InvalidNumericValue
}

pub fn parser_rune_files(input_path: &Path, append_extensions: bool, silent: bool) -> Result<Vec<RuneFileDescription>, RuneParserError> {
    // Enable silent mode if requested by user
    if silent {
        enable_silent();
    }

    if !input_path.exists() || !input_path.is_dir() {
        if !input_path.exists() {
            error!("Input path \"{0}\" does not exist!", input_path.to_str().expect("Could not parse OS string!"));
        } else if !input_path.is_dir() {
            error!("Input path \"{0}\" is not a directory!", input_path.to_str().expect("Could not parse OS string!"));
        }

        return Err(RuneParserError::InvalidInputPath);
    }

    // Get rune files from folder
    // ———————————————————————————

    // Create a vector with allocated space for 64 rune files, which should be more than plenty for most projects
    let mut rune_file_list: Vec<String> = Vec::with_capacity(ALLOCATION_SIZE);

    match get_rune_files(input_path, &mut rune_file_list) {
        Err(error) => return Err(error),
        Ok(_) => ()
    }

    if rune_file_list.is_empty() {
        warning!("Could not parse any rune files from folder. Returning empty list");
        return Ok(Vec::new());
    }

    // Print all found files
    info!("\nFound the following rune files:");
    for i in 0..rune_file_list.len() {
        info!("    {0}", rune_file_list[i]);
    }

    // Process rune files
    // ———————————————————

    let mut definitions_list: Vec<RuneFileDescription> = Vec::with_capacity(ALLOCATION_SIZE);

    for filepath in rune_file_list {
        let file_path: &Path = Path::new(&filepath);

        let file = match std::fs::read_to_string(file_path) {
            Err(error) => {
                error!("Error in reading file to string. Got error {0}", error);
                continue;
            },
            Ok(path) => path
        };

        // Scan file for tokens
        let tokens = match Scanner::new(file.chars()).scan_all() {
            Err(error) => {
                error!("Error while scanning file {0}: {1:#?}", filepath, error);
                continue;
            },
            Ok(tokens) => tokens
        };

        // Parse all scanned tokens
        let types: Definitions = match parse_tokens(&mut tokens.into_iter().peekable()) {
            Err(error) => {
                error!("Error while parsing file {0}: {1:#?}", filepath, error);
                continue;
            },
            Ok(tokens) => tokens
        };

        // Get isolated file name (without .rune extension)
        let full_file_name: String = match file_path.file_name() {
            None => {
                error!("File given at path {0:?} had no name!", file_path);
                continue;
            },
            Some(os_string) => match os_string.to_str() {
                None => {
                    error!("Could not parse OS string: \"{0:?}\"", os_string);
                    continue;
                },
                Some(string) => string.to_string()
            }
        };

        let file_name: String = match full_file_name.strip_suffix(".rune") {
            None => {
                error!("Could not strip '.rune' suffix from file name!");
                continue;
            },
            Some(stripped_name) => stripped_name.to_string()
        };

        let input_path_string = match input_path.to_str() {
            None => {
                error!("Could not parse input path: \"{0:?}\"", input_path);
                continue;
            },
            Some(string) => string
        };

        // Get relative path (from input path)
        let relative_path = match filepath.strip_prefix(input_path_string) {
            None => {
                warning!("Could not get relative path from input path string \"{0}\"", input_path_string);
                continue;
            },
            Some(string) => match string.strip_prefix("/") {
                None => {
                    warning!("Could not get relative path from input path string \"{0}\"", input_path_string);
                    continue;
                },
                Some(stripped_path) => match stripped_path.strip_suffix(&full_file_name) {
                    None => {
                        warning!("Could not get relative path from input path string \"{0}\"", input_path_string);
                        continue;
                    },
                    Some(relative_path) => relative_path.to_string()
                }
            }
        };

        definitions_list.push(RuneFileDescription {
            relative_path: relative_path,
            file_name:     file_name,
            definitions:   types
        });
    }

    // Post-processing
    // ————————————————

    // Parse and resolve define statements
    parse_define_statements(&mut definitions_list)?;

    // Parse and link user defined data types across files
    link_user_definitions(&mut definitions_list)?;

    // Parse extensions
    parse_extensions(&mut definitions_list, append_extensions)?;

    // Validate parsed data structures
    // ————————————————————————————————

    validate_parsed_files(&definitions_list)?;

    // Return list
    // ————————————

    Ok(definitions_list)
}

fn get_rune_files(folder_path: &Path, mut rune_file_list: &mut Vec<String>) -> Result<(), RuneParserError> {
    let folder_iterator: ReadDir = match folder_path.read_dir() {
        Err(error) => {
            error!(
                "Could not read \"{0}\" directory. Got error {1}",
                folder_path.to_str().expect("Could not get string from folder path"),
                error
            );
            return Err(RuneParserError::FileSystemError);
        },
        Ok(value) => value
    };

    for item in folder_iterator {
        // Check if we got a valid entry
        let directory_entry = match item {
            Err(error) => {
                warning!(
                    "Got an error {0} in one of the items in \"{1}\" directory",
                    error,
                    folder_path.to_str().expect("Could not get string from folder path")
                );
                continue;
            },
            Ok(entry) => entry
        };

        // Get entry type
        let entry_type = match directory_entry.file_type() {
            Err(error) => {
                warning!(
                    "Got error {0} in getting file type of file \"{1}\"",
                    error,
                    directory_entry.file_name().to_str().expect("Could not get string from file name")
                );
                continue;
            },
            Ok(file_type) => file_type
        };

        if entry_type.is_dir() {
            // Subfolder
            // ——————————

            info!("Found subdirectory named {0:?}", directory_entry.file_name());

            let subfolder_string: String = format!(
                "{0}/{1}",
                match folder_path.to_str() {
                    None => {
                        warning!("Could not get string from file path {0:?}", folder_path);
                        continue;
                    },
                    Some(string) => string
                },
                match directory_entry.file_name().to_str() {
                    None => {
                        warning!("Could not get string from file name {0:?}", directory_entry.file_name());
                        continue;
                    },
                    Some(string) => string
                }
            );

            let subfolder_path: &Path = Path::new(&subfolder_string);

            // Recursively call function to parse files in subfolder
            match get_rune_files(subfolder_path, &mut rune_file_list) {
                Err(error) => return Err(error),
                Ok(_) => ()
            }
        } else if entry_type.is_file() {
            // Rune file
            // ——————————

            let file_string = match directory_entry.file_name().into_string() {
                Ok(string) => string,
                Err(error) => {
                    warning!("Could not parse file name into string. Got error: {0:#?}", error);
                    continue;
                }
            };

            if file_string.ends_with(".rune") {
                rune_file_list.push(format!(
                    "{0}/{1}",
                    match folder_path.to_str() {
                        None => {
                            warning!("Could not parse OS string: \"{0:?}\"", folder_path);
                            continue;
                        },
                        Some(string) => string
                    },
                    file_string
                ));
            }
        } else {
            /* Nothing - Ignore anything that is not a subfolder or a .rune file */
        }
    }

    return Ok(());
}
