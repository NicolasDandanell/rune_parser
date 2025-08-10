mod header;
mod parser;
mod runic_definitions;
mod source;
mod utilities;

use crate::languages::c::{ utilities::CConfigurations, header::output_header, parser::output_parser, runic_definitions::output_runic_definitions, source::output_source };
use crate::{ Configurations, RuneFileDescription };
use std::path::Path;

pub fn output_c_files(file_descriptions: Vec<RuneFileDescription>, output_path: &Path, configurations: Configurations) {

    let c_configurations: CConfigurations = CConfigurations::parse(&file_descriptions, &configurations);

    // Create runic definitions file
    println!("Outputting runic definitions");
    output_runic_definitions(&file_descriptions, &c_configurations, output_path);

    // Create source and header files matching the Rune files
    println!("Outputting headers and sources for:");
    for file in &file_descriptions {
        println!("    {0}.rune", file.file_name);

        // Create header file
        output_header(&file, output_path);

        // Create source file
        output_source(&file, output_path);
    }

    // Create parser
    println!("Outputting parser file");
    output_parser(&file_descriptions, output_path);

    println!("Done!");
}
