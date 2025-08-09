pub mod c;
pub mod header;
pub mod source;
pub mod utilities;

pub use c::output_c_files;
pub use crate::Configurations;

pub struct CConfigurations {
    // Configurations
    pub compiler_configurations: Configurations,

    // Data definitions
    pub field_size_type_size:    usize,
    pub field_offset_type_size:  usize,
    pub message_size_type_size:  usize,
    pub parser_index_type_size:  usize,
}
