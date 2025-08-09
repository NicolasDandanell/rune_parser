pub mod types;
pub mod languages;
pub mod parser;
pub mod post_processing;
pub mod scanner;

use clap::Parser;
use languages::c::output_c_files;
use post_processing::{ link_user_definitions, parse_define_statements };
use scanner::Scanner;
use std::{ fs::ReadDir, path::Path, process::exit };
use types::Definitions;

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

    /// Whether to pack (remove padding) from outputted sources - Defaults to false
    #[arg(long, short = 'p', default_value = "false")]
    pack: bool,

    /// Whether to store all Rune data in a specific section. By default no section is declared
    #[arg(long, short = 'd')]
    data_section: Option<String>,

    /// Whether to sort struct field placement to optimize alignment - Defaults to true
    #[arg(long, short = 's', default_value = "true")]
    sort: bool
}

#[derive(Debug, Clone)]
pub struct Configurations {
    /// Whether or not to pack data structures
    pack: bool,

    /// Whether to declare all rune data in a specific section - Default to None
    section: Option<String>,

    /// Whether to size sort structs to optimize packing - Defaults to true
    sort: bool
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

fn main() -> Result<(), usize> {

    // Parse arguments
    // ————————————————

    let args: Args = Args::parse();

    let input_path: &Path              = Path::new(args.rune_folder.as_str());
    let output_path: &Path             = Path::new(args.output_folder.as_str());
    let output_language: Language      = Language::from_string(args.language);
    let configurations: Configurations = Configurations {
        pack:    args.pack,
        section: args.data_section,
        sort:    args.sort
    };

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

    // Post-processing
    // ————————————————

    // Parse and resolve define statements
    parse_define_statements(&mut definitions_list);

    // Parse and link user defined data types across files
    link_user_definitions(&mut definitions_list);

    // Validate parsed data structures
    // ————————————————————————————————

    // To be implemented...

    // Create source files
    // ————————————————————

    match output_language {
        Language::C => output_c_files(definitions_list, output_path, configurations),
        Language::Unsupported => unreachable!()
    }

    Ok(())
}
