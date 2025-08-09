mod header;
mod runic_definitions;
mod source;
mod utilities;

use crate::languages::c::{ utilities::CConfigurations, header::output_header, runic_definitions::output_runic_definitions };
use crate::{ Configurations, RuneFileDescription };
use std::path::Path;

pub fn output_c_files(file_descriptions: Vec<RuneFileDescription>, output_path: &Path, configurations: Configurations) {

    let c_configurations: CConfigurations = CConfigurations::parse(&file_descriptions, &configurations);

    // Create runic definitions file
    output_runic_definitions(&c_configurations, output_path);

    for file in file_descriptions {
        // Create header file
        output_header(file, output_path);
    }
}
