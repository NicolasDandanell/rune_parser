use crate::languages::c::header::output_header;
use crate::RuneFileDescription;
use std::path::Path;

pub fn output_c_files(file_description: Vec<RuneFileDescription>, output_path: &Path, packed: bool) {

    for file in file_description {
        // Create header file
        output_header(file, output_path, packed);
    }

}
