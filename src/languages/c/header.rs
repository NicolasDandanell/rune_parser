use crate::types::{ BitfieldDefinition, BitfieldMember, DefineDefinition, DefineValue, EnumDefinition, StructDefinition, StructMember };
use crate::RuneFileDescription;
use crate::languages::c::utilities::{ OutputFile, pascal_to_snake_case, pascal_to_uppercase, spaces };
use std::fmt::format;
use std::path::Path;

/// Outputs a bitfield definition into the header file
fn output_bitfield(header_file: &mut OutputFile, bitfield_definition: &BitfieldDefinition) {
    // Print comment if present
    match &bitfield_definition.comment {
        Some(comment) =>  header_file.add_line(format!("/**{0}*/", comment)),
        None => ()
    }

    let bitfield_name: String = pascal_to_snake_case(&bitfield_definition.name);

    let mut little_endian_order: Vec<BitfieldMember> = Vec::with_capacity(bitfield_definition.members.len());
    let mut big_endian_order:    Vec<BitfieldMember> = Vec::with_capacity(bitfield_definition.members.len());

    // Calculate required padding for ensuring proper alignment
    let mut total_size: usize = 0;

    for member in &bitfield_definition.members {
        total_size += member.bit_size;
    }

    let padding: BitfieldMember = BitfieldMember {
        ident:    String::from("padding"),
        bit_size: (bitfield_definition.backing_type.primitive_c_size() * 8) - total_size,
        bit_slot: 0, // Does not matter
        comment:  Some(String::from(" Padding to ensure proper alignment "))
    };

    // Calculate longest member name for spacing
    let mut longest_name: usize = match padding.bit_size {
        0 => 0,
        _ => String::from("padding").len()
    };

    for member in &bitfield_definition.members {
        let member_name = pascal_to_snake_case(&member.ident);
        if member_name.len() > longest_name {
            longest_name = member_name.len();
        }
    }

    // Disclaimer
    // ———————————

    header_file.add_line(String::from("// Disclaimer ! Run rune_bitfield_tester() function to check whether bitfields are behaving as intended"));

    // Little endian order
    // ————————————————————

    header_file.add_line(String::from("#if defined __LITTLE_ENDIAN__"));
    header_file.add_line(format!("typedef struct RUNIC_BITS {0} {{", bitfield_name));

    // Comment
    if bitfield_definition.comment.is_some() {
        header_file.add_line(format!("/**{0}*/", bitfield_definition.comment.as_ref().unwrap()));
    }

    // Get little endian order
    for i in 0..bitfield_definition.members.len() {
        for member in &bitfield_definition.members {
            if member.bit_slot == i {
                little_endian_order.push(member.clone());
            }
        }
    }

    // Add padding - In the end for little endian
    little_endian_order.push(padding.clone());

    // Print bits
    for member in &little_endian_order {
        if member.comment.is_some() {
            // Member comment
            header_file.add_line(format!("    /**{0}*/", member.comment.as_ref().unwrap()));
        }

        let member_name = pascal_to_snake_case(&member.ident);

        header_file.add_line(format!("    {0} {1}{2} : {3};",
            bitfield_definition.backing_type.to_c_type(),
            member_name,
            spaces(longest_name - member_name.len()),
            member.bit_size
        ));
    }

    header_file.add_line(format!("}} {0}_t;", bitfield_name));

    // Big endian order
    // —————————————————

    header_file.add_line(String::from("#elif defined __BIG_ENDIAN__"));
    header_file.add_line(format!("typedef struct RUNIC_BITS {0} {{", bitfield_name));

    // Comment
    if bitfield_definition.comment.is_some() {
        header_file.add_line(format!("/**{0}*/", bitfield_definition.comment.as_ref().unwrap()));
    }

    // Add padding - In the beginning for little endian
    big_endian_order.push(padding.clone());

    // Get big endian order
    for z in 0..bitfield_definition.members.len() {
        let i = bitfield_definition.members.len() - 1 - z;
        for member in &bitfield_definition.members {
            if member.bit_slot == i {
                big_endian_order.push(member.clone());
            }
        }
    }

    // Print bits
    for member in &big_endian_order {
        if member.comment.is_some() {
            // Member comment
            header_file.add_line(format!("    /**{0}*/", member.comment.as_ref().unwrap()));
        }

        let member_name: String = pascal_to_snake_case(&member.ident);

        header_file.add_line(format!("    {0} {1}{2} : {3};",
            bitfield_definition.backing_type.to_c_type(),
            member_name,
            spaces(longest_name - member_name.len()),
            member.bit_size
        ));
    }

    header_file.add_line(format!("}} {0}_t;", bitfield_name));

    // Error
    // ——————

    header_file.add_line(String::from("#else"));
    header_file.add_line(String::from("#error \"Only little and big endianness is supported by this Rune C implementation\""));
    header_file.add_line(String::from("#endif // __BYTE_ORDER__"));
    header_file.add_newline();

    // Initializer
    // ————————————

    header_file.add_line(format!("#define {0}_INIT 0", pascal_to_uppercase(&bitfield_definition.name)));
    header_file.add_newline();
}

/// Outputs a define statement into the header file
fn output_define(header_file: &mut OutputFile, define: &DefineDefinition) {
    // Print comment if present
    match &define.comment {
        Some(comment) =>  header_file.add_line(format!("/**{0}*/", comment)),
        None => ()
    }

    let define_name: String  = define.identifier.clone();
    let define_value: String = match &define.value {
        DefineValue::NoValue => String::from(""),
        DefineValue::FloatLiteral(value)   => value.to_string(),
        DefineValue::IntegerLiteral(value) => value.to_string(),
        DefineValue::Composite(value)   => value.clone()
    };

    header_file.add_line(format!("#define {0} {1}", define_name, define_value));
}

/// Outputs an enum into the header file
fn output_enum(header_file: &mut OutputFile, enum_definition: &EnumDefinition) {
    // Print comment if present
    match &enum_definition.comment {
        Some(comment) =>  header_file.add_line(format!("/**{0}*/", comment)),
        None => ()
    }

    let enum_name: String = pascal_to_snake_case(&enum_definition.name);
    let enum_type: String = enum_definition.backing_type.to_c_type();

    header_file.add_line(format!("typedef enum {0}: {1} {{", enum_name, enum_type));

    let mut longest_member_name: usize = 0;

    // Get longest name for spacing calculations
    for i in 0..enum_definition.members.len() {
        if longest_member_name < pascal_to_uppercase(&enum_definition.members[i].ident).len() {
            longest_member_name = pascal_to_uppercase(&enum_definition.members[i].ident).len();
        }
    }

    // Print all enum members
    for i in 0..enum_definition.members.len() {

        let enum_member = &enum_definition.members[i];

        // Member comment
        if enum_member.comment.is_some() {
            header_file.add_line(format!("    /**{0}*/", enum_member.comment.as_ref().unwrap()));
        }

        let member_name: String = pascal_to_uppercase(&enum_member.ident);
        let ending: String = match i == enum_definition.members.len() - 1 {
            false => String::from(",\n"),
            true  => String::from("")
        };

        header_file.add_line(format!("    {0}{1} = {2}{3}", member_name, spaces(longest_member_name - member_name.len()), enum_member.value.to_string(), ending));
    }

    header_file.add_line(format!("}} {0}_t;", enum_name));
    header_file.add_newline();
}

/// Output a struct into the header file
fn output_struct(header_file: &mut OutputFile, struct_definition: &StructDefinition) -> Vec<StructMember> {
    // Print comment if present
    match &struct_definition.comment {
        Some(comment) =>  header_file.add_line(format!("/**{0}*/", comment)),
        None => ()
    }

    let struct_name: String = pascal_to_snake_case(&struct_definition.name);

    header_file.add_line(format!("typedef struct RUNIC {0} {{", struct_name));

    // Sorted list --> Then use sorted list instead of other one
    let sorted_member_list: Vec<StructMember> = struct_definition.sort_members();

    // Print all struct members
    for i in 0..sorted_member_list.len() {

        let struct_member = &sorted_member_list[i];

        // Member comment
        if struct_member.comment.is_some() {
            header_file.add_line(format!("    /**{0}*/", struct_member.comment.as_ref().unwrap()));
        }

        let member_name: String = pascal_to_snake_case(&struct_member.ident);

        header_file.add_line(format!("    {0};", struct_member.field_type.create_c_variable(&member_name)));

        if i < sorted_member_list.len() - 1 {
            header_file.add_newline();
        }
    }

    header_file.add_line(format!("}} {0}_t;", struct_name));
    header_file.add_newline();

    sorted_member_list
}

fn output_struct_initializer(output_file: &mut OutputFile, struct_definition: &StructDefinition) {
    let mut pre_equal_length: usize   = 0;

    let sorted_member_list: Vec<StructMember> = struct_definition.sort_members();

    // Calculate spacing for aligning the '=' sign
    // ————————————————————————————————————————————

    for member in &sorted_member_list {
        if member.ident.len() > pre_equal_length {
            pre_equal_length = member.ident.len();
        }
    }

    // Calculate the space for aligning the '\' at the end
    // ————————————————————————————————————————————————————

    let initializer_string: String = format!("#define {0}_INIT ({1}) {{{2}", pascal_to_uppercase(&struct_definition.name), format!("{0}_t", pascal_to_snake_case(&struct_definition.name)), spaces(0));
    let mut pre_newline_length: usize = initializer_string.len(); // - 2

    // Calculate spacing for after the newline
    for member in &sorted_member_list {
        let pre_equal: usize = pre_equal_length - member.ident.len();

        let string: String = format!("    .{0}{1} = {2}, {3}\\", member.ident, spaces(pre_equal), member.field_type.c_initializer(), "");

        // I don't know why the -2 is needed, but it does not work without it
        if string.len() - 2 > pre_newline_length {
            pre_newline_length = string.len() - 2;
        }
    }

    // 20 seems to be the number of fixed characters on the define string
    let define_size: usize = 20 + pascal_to_uppercase(&struct_definition.name).len() + pascal_to_snake_case(&struct_definition.name).len();

    output_file.add_line(format!("#define {0}_INIT ({1}_t) {{ {2}\\", pascal_to_uppercase(&struct_definition.name), pascal_to_snake_case(&struct_definition.name), spaces(pre_newline_length -  define_size)));
    for member in sorted_member_list {
        let pre_equal: usize   = pre_equal_length - member.ident.len();
        let pre_newline: usize = pre_newline_length - pre_equal_length - member.field_type.c_initializer().len() - 9;

        output_file.add_line(format!("    .{0}{1} = {2}, {3}\\", member.ident, spaces(pre_equal), member.field_type.c_initializer(), spaces(pre_newline)));
    }
    output_file.add_line(format!("}}"));
    output_file.add_newline();
}

pub fn output_header(file: RuneFileDescription, output_path: &Path, packed: bool) {

    // Print disclaimers. Requires C23 compliant compiler
    //
    // · Autogenerated code info
    //
    // · Compiler version (C23 compliant)
    //
    // GCC 13 or higher
    // CLang 8.0 or higher
    //
    // · Include & C++ guards
    //
    // · standard includes
    //
    // <stdbool.h>
    // <stdint.h>
    //
    // —————————————————————————————————————————————————

    // Packed to be used in structs if activated
    // enums to be type-defined

    // String for optional packing
    let runic_string: String = match packed {
        true  => String::from("__attribute__((packed))"),
        false => String::from("")
    };

    let runic_bits_string: String = match packed {
        true  => String::from("RUNIC"),
        false => String::from("__attribute__((packed))")
    };

    let h_file_string: String = format!("{0}/{1}.rune.h", output_path.to_str().unwrap(), file.file_name);

    let mut header_file: OutputFile = OutputFile::new(h_file_string);

    // Disclaimers
    // ————————————

    // ...

    // Start & C++ guards
    // ———————————————————

    header_file.add_line(format!("#ifndef {0}_RUNE_H", file.file_name.to_uppercase()));
    header_file.add_line(format!("#define {0}_RUNE_H", file.file_name.to_uppercase()));
    header_file.add_newline();

    header_file.add_line(format!("#ifdef __cplusplus"));
    header_file.add_line(format!("extern \"C\" {{"));
    header_file.add_line(format!("#endif // __cplusplus"));
    header_file.add_newline();

    // File inclusions
    // ————————————————

    // Standard library
    header_file.add_line(format!("#include <stdbool.h>"));
    header_file.add_line(format!("#include <stdint.h>"));
    header_file.add_newline();

    if !file.definitions.includes.is_empty() {
        // Print out includes
        for include_definition in file.definitions.includes {
            header_file.add_line(format!("#include \"{0}.rune.h\"", include_definition.file));
        }

        // Separation line
        header_file.add_newline();
    }

    // Runic define
    // —————————————

    // Currently used for the packing setting, and in the future might be used for other settings
    header_file.add_line(format!("#define RUNIC {0}{1}", spaces(!file.definitions.bitfields.is_empty() as usize * 5), runic_string));
    match file.definitions.bitfields.is_empty() {
        false => { header_file.add_line(format!("#define RUNIC_BITS {0}", runic_bits_string)); },
        true  => ()
    }
    header_file.add_newline();

    // User defines
    // —————————————

    if !file.definitions.defines.is_empty() {
        for define in file.definitions.defines {
            output_define(&mut header_file, &define);
        }
        header_file.add_newline();
    }

    // Enums
    // ——————

    // Print all enum definitions
    for enum_definition in file.definitions.enums {
        output_enum(&mut header_file, &enum_definition);
    }

    // Bitfields
    // ——————————

    for bitfield_definition in file.definitions.bitfields {
        output_bitfield(&mut header_file, &bitfield_definition);
    }

    // Structs
    // ————————

    if !file.definitions.structs.is_empty() {

        // Print out structs
        for struct_definition in file.definitions.structs {
            output_struct(&mut header_file, &struct_definition);

            // Add struct initializer
            output_struct_initializer(&mut header_file, &struct_definition)
        }
    }

    // End & C++ guards
    // —————————————————

    header_file.add_line(format!("#ifdef __cplusplus"));
    header_file.add_line(format!("}}"));
    header_file.add_line(format!("#endif // __cplusplus"));
    header_file.add_newline();

    header_file.add_line(format!("#endif // {0}_RUNE_H", file.file_name.to_uppercase()));

    // Output file
    // ————————————

    header_file.output_file();

}
