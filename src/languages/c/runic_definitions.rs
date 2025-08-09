use crate::languages::c::utilities::{ CConfigurations, OutputFile };
use std::path::Path;

fn type_from_size(size: usize) -> String {
    match size {
        1 => String::from("uint8_t"),
        2 => String::from("uint16_t"),
        4 => String::from("uint32_t"),
        8 => String::from("uint64_t"),
        _ => panic!("Invalid type size given!")
    }
}

pub fn output_runic_definitions(configurations: &CConfigurations, output_path: &Path) {
    let definitions_file_string: String = format!("{0}/runic_definitions.h", output_path.to_str().unwrap());

    let     enum_attributes:     String = String::with_capacity(0x100);
    let mut bitfield_attributes: String = String::with_capacity(0x100);
    let mut struct_attributes:   String = String::with_capacity(0x100);

    // Parse "packed" attribute
    // —————————————————————————

    // Bitfields are always packed!
    match bitfield_attributes.is_empty() {
        true  => bitfield_attributes.push_str("packed"),
        false => bitfield_attributes.push_str(", packed")
    }

    // Enums have backing types, and do not need to be packed

    if configurations.compiler_configurations.pack {
        // Structs
        match struct_attributes.is_empty() {
            true  => struct_attributes.push_str("packed"),
            false => struct_attributes.push_str(", packed")
        }
    }

    // Parse "section" attribute
    // ——————————————————————————

    // For variables, and not data definitions...

    // Create attribute strings
    // —————————————————————————

    // Runic bitfields must ALWAYS be packed, so this will never be empty
    let runic_bitfield_string: String = format!("__attribute__(({0}))", bitfield_attributes);

    // Enums
    let runic_enum_string: String = match enum_attributes.is_empty() {
        true  => String::new(),
        false => format!("__attribute__(({0}))", enum_attributes)
    };

    // Structs
    let runic_struct_string: String = match struct_attributes.is_empty() {
        true  => String::new(),
        false => format!("__attribute__(({0}))", struct_attributes)
    };

    let mut definitions_file: OutputFile = OutputFile::new(definitions_file_string);

    definitions_file.add_line(format!("#ifndef RUNE_DEFINITIONS_H"));
    definitions_file.add_line(format!("#define RUNE_DEFINITIONS_H"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("// Static definitions"));
    definitions_file.add_line(format!("// ———————————————————"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("#define FIELD_INDEX_BITS         0x1F"));
    definitions_file.add_line(format!("#define LACKS_VERIFICATION_FIELD INT8_MIN"));
    definitions_file.add_line(format!("#define NO_PARSER                0"));
    definitions_file.add_line(format!("#define TRANSPORT_TYPE_BITS      0xE0"));
    definitions_file.add_line(format!("#define VERIFICATION_FIELD       0x1F"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("// Configuration dependent definitions"));
    definitions_file.add_line(format!("// ————————————————————————————————————"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("/* These definitions are based on the configurations passed by user to get code generator, such as packing, specific data sections, or other */"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("#define RUNIC_BITFIELD {0}", runic_bitfield_string));
    definitions_file.add_line(format!("#define RUNIC_ENUM     {0}", runic_enum_string));
    definitions_file.add_line(format!("#define RUNIC_STRUCT   {0}", runic_struct_string));
    definitions_file.add_newline();

    definitions_file.add_line(format!("// Message dependent definitions"));
    definitions_file.add_line(format!("// ——————————————————————————————"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("/* These definitions are dependent on the declared data, and will vary to adapt to accommodate the sizes of the declared data structures */"));
    definitions_file.add_newline();

    definitions_file.add_line(format!("#define FIELD_SIZE_TYPE   {0}", type_from_size(configurations.field_size_type_size)));
    definitions_file.add_line(format!("#define FIELD_OFFSET_TYPE {0}", type_from_size(configurations.field_offset_type_size)));
    definitions_file.add_line(format!("#define MESSAGE_SIZE_TYPE {0}", type_from_size(configurations.message_size_type_size)));
    definitions_file.add_line(format!("#define PARSER_INDEX_TYPE {0}", type_from_size(configurations.parser_index_type_size)));
    definitions_file.add_newline();

    definitions_file.add_line(format!("#endif // RUNIC_DEFINITIONS_H"));

    definitions_file.output_file();
}
