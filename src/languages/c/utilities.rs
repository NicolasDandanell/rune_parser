use crate::types::{ ArraySize, DefineValue, FieldType, StructDefinition, StructMember, UserDefinitionLink };
use std::{ fs::{ File, remove_file }, io::Write, path::Path };

// String helper functions
// ————————————————————————

pub fn spaces(amount: usize) -> String {
    let mut spaces = String::with_capacity(0x40);

    for _ in 0..amount {
        spaces.push(' ');
    }

    spaces
}

pub fn pascal_to_snake_case(pascal: &String) -> String {
    let mut snake: String = String::with_capacity(0x40);

    for i in 0..pascal.len() {

        let letter: char = pascal.chars().nth(i).unwrap();

        if i != 0 && letter.is_ascii_uppercase() { snake.push('_'); }

        snake.push(letter.to_ascii_lowercase());
    }

    snake
}

pub fn pascal_to_uppercase(pascal: &String) -> String {
    let mut uppecase: String = String::with_capacity(0x40);

    for i in 0..pascal.len() {

        let letter: char = pascal.chars().nth(i).unwrap();

        if i != 0 && letter.is_ascii_uppercase() { uppecase.push('_'); }

        uppecase.push(letter.to_ascii_uppercase());
    }

    uppecase
}

// Field type methods
// ———————————————————

impl FieldType {
    pub fn to_c_type(&self) -> String {
        match self {
            FieldType::Boolean => String::from("bool"),
            FieldType::UByte   => String::from("uint8_t"),
            FieldType::Byte    => String::from("int8_t"),

            FieldType::UShort  => String::from("uint16_t"),
            FieldType::Short   => String::from("int16_t"),

            FieldType::Float   => String::from("float"),
            FieldType::UInt    => String::from("uint32_t"),
            FieldType::Int     => String::from("int32_t"),

            FieldType::Double  => String::from("double"),
            FieldType::ULong   => String::from("uint64_t"),
            FieldType::Long    => String::from("int64_t"),

            FieldType::UserDefined(string) => format!("{0}_t", pascal_to_snake_case(string)),

            // Should not be called on this type
            FieldType::Array(_, _) => unreachable!()
        }
    }

    pub fn create_c_variable(&self, name: &String) -> String {
        match self {
            FieldType::Boolean => format!("bool {0}", name),
            FieldType::UByte   => format!("uint8_t {0}", name),
            FieldType::Byte    => format!("int8_t {0}", name),

            FieldType::UShort  => format!("uint16_t {0}", name),
            FieldType::Short   => format!("int16_t {0}", name),

            FieldType::Float   => format!("float {0}", name),
            FieldType::UInt    => format!("uint32_t {0}", name),
            FieldType::Int     => format!("int32_t {0}", name),

            FieldType::Double  => format!("double {0}", name),
            FieldType::ULong   => format!("uint64_t {0}", name),
            FieldType::Long    => format!("int64_t {0}", name),

            FieldType::UserDefined(string) => format!("{0}_t {1}", pascal_to_snake_case(string), name),

            FieldType::Array(field_type, field_size) => {

                let array_size: String = match field_size {
                    ArraySize::UserDefinition(definition) => definition.identifier.clone(),
                    ArraySize::NumericValue(size) => size.to_string()
                };

                format!("{0} {1}[{2}]", field_type.to_c_type(), name, array_size)
            }
        }
    }

    // Size is calculated without padding, and is a guesstimate at best
    pub fn primitive_c_size(&self) -> usize {
        match self {
            FieldType::Boolean => 1,
            FieldType::UByte   => 1,
            FieldType::Byte    => 1,

            FieldType::UShort  => 2,
            FieldType::Short   => 2,

            FieldType::Float   => 4,
            FieldType::UInt    => 4,
            FieldType::Int     => 4,

            FieldType::Double  => 8,
            FieldType::ULong   => 8,
            FieldType::Long    => 8,

            _ => panic!("Cannot call this function on an array or user defined type")
        }
    }

    pub fn c_initializer(&self) -> String {
        match self {
            FieldType::Boolean                    => String::from("false"),
            FieldType::Byte                       => String::from("0"),
            FieldType::UByte                      => String::from("0"),
            FieldType::Short                      => String::from("0"),
            FieldType::UShort                     => String::from("0"),
            FieldType::Float                      => String::from("0.0"),
            FieldType::Int                        => String::from("0"),
            FieldType::UInt                       => String::from("0"),
            FieldType::Double                     => String::from("0.0"),
            FieldType::Long                       => String::from("0"),
            FieldType::ULong                      => String::from("0"),
            FieldType::UserDefined(name) => format!("{0}_INIT", pascal_to_uppercase(&name)),
            FieldType::Array(field_type, array_size) =>
                format!("{{ [0 ... {0}] = {1} }}",
                    match array_size {
                        ArraySize::NumericValue(value) => value - 1,
                        ArraySize::UserDefinition(definition) => {
                            let size_value: usize = match definition.value {
                                DefineValue::IntegerLiteral(value) => match value.try_into() {
                                    Err(error) => panic!("Could not parse \"{0:?}\" array size into a positive integer value! Got error {1}", self, error),
                                    Ok(value) => value
                                },
                                _ => panic!("Got \"{0:?}\" array size definition of an invalid type!", self)
                            };
                            size_value - 1
                        }
                    },
                    match field_type.as_ref() {
                        FieldType::Boolean           => String::from("false"),
                        FieldType::Byte              => String::from("0"),
                        FieldType::UByte             => String::from("0"),
                        FieldType::Short             => String::from("0"),
                        FieldType::UShort            => String::from("0"),
                        FieldType::Float             => String::from("0.0"),
                        FieldType::Int               => String::from("0"),
                        FieldType::UInt              => String::from("0"),
                        FieldType::Double            => String::from("0.0"),
                        FieldType::Long              => String::from("0"),
                        FieldType::ULong             => String::from("0"),
                        FieldType::Array(_, _)       => panic!("Nested arrays are not currently supported"),
                        FieldType::UserDefined(name) => format!("{0}_INIT", pascal_to_uppercase(&name))
                    }
            )
        }
    }
}

// Struct member methods
// ——————————————————————

impl StructMember {
    pub fn c_size(&self) -> usize {
        match &self.field_type {

            // Calculate Array size based on (field type * field size)
            FieldType::Array(array_field_type, field_size) => {

                // Get the array size first
                let array_size: usize = match field_size {
                    ArraySize::NumericValue(value) => *value,
                    ArraySize::UserDefinition(definition) => match definition.value {
                        DefineValue::IntegerLiteral(value) => match value.try_into() {
                            Err(error) => panic!("Could not parse \"{0}\" array size into a positive integer value! Got error {1}", self.ident, error),
                            Ok(value) => value
                        },
                        _ => panic!("Got \"{0}\" array size definition of an invalid type!", self.ident)
                    }
                };

                // Parse the byte size based on the array type
                match *array_field_type.to_owned() {
                    FieldType::Array(_, _)    => panic!("Nested arrays not allowed at the moment"),

                    // Parse the user defined type using the member user_definition_link
                    FieldType::UserDefined(type_string) => match &self.user_definition_link {
                        UserDefinitionLink::NoLink                                    => panic!("Could not find definition for type {0} while parsing C size", type_string),
                        UserDefinitionLink::EnumLink(enum_definition) => enum_definition.backing_type.primitive_c_size() * array_size,
                        UserDefinitionLink::StructLink(struct_definition) => {

                            let mut struct_size = 0;

                            // Call this function recursively for each struct member to get size
                            for member in &struct_definition.members {
                                struct_size += member.c_size();
                            }

                            struct_size * array_size
                        }
                    },

                    // Primitives
                    _ => array_field_type.primitive_c_size() * array_size
                }
            },

            FieldType::UserDefined(name) => {
                match &self.user_definition_link {
                    UserDefinitionLink::NoLink                                          => panic!("Found no definition link for item {0}!", name),
                    UserDefinitionLink::EnumLink(enum_definition)       => enum_definition.backing_type.primitive_c_size(),
                    UserDefinitionLink::StructLink(struct_definition) => {
                        let mut total_size = 0;

                        for member in &struct_definition.members {
                            total_size += member.c_size();
                        }

                        total_size
                    }
                }
            },

            // Primitives
            _ =>  self.field_type.primitive_c_size()
        }
    }
}

// Struct definition methods
// ——————————————————————————

impl StructDefinition {
    /// Sort the members of a struct based on their size alignment to reduce eventual padding
    pub fn sort_members(&self) -> Vec<StructMember> {
        let mut full_list: Vec<StructMember> = Vec::with_capacity(0x20);

        let mut aligned_8: Vec<StructMember> = Vec::with_capacity(0x20);
        let mut aligned_4: Vec<StructMember> = Vec::with_capacity(0x20);
        let mut aligned_2: Vec<StructMember> = Vec::with_capacity(0x20);
        let mut aligned_1: Vec<StructMember> = Vec::with_capacity(0x20);

        for member in &self.members {

            if member.c_size() % 8 == 0 {
                // First 8 aligned
                aligned_8.push(member.clone());
            } else if member.c_size() % 4 == 0 {
                // First 4 aligned
                aligned_4.push(member.clone());
            } else if member.c_size() % 2 == 0 {
                // First 2 aligned
                aligned_2.push(member.clone());
            } else {
                // Lastly non aligned
                aligned_1.push(member.clone());
            }
        }

        full_list.append(&mut aligned_8);
        full_list.append(&mut aligned_4);
        full_list.append(&mut aligned_2);
        full_list.append(&mut aligned_1);

        full_list
    }
}

// Output file declaration
// ————————————————————————

pub struct OutputFile {
    file_name: String,
    string_buffer: String
}

impl OutputFile {
    pub fn new(file_name: String) -> OutputFile {

        // Create string buffer
        let string_buffer: String = String::with_capacity(0x2000);

        OutputFile {
            file_name,
            string_buffer
        }
    }

    pub fn add_line(&mut self, string: String) {
        self.string_buffer.push_str(format!("{0}\n", string).as_str());
    }

    pub fn add_newline(&mut self) {
        self.string_buffer.push_str("\n");
    }

    pub fn output_file(&self) {

        let output_file_path: &Path = Path::new(&self.file_name);

        // Check if file already exists
        if output_file_path.exists() {
            match remove_file(output_file_path) {
                Err(error) => panic!("Could not delete existing {0} file. Got error {1}", output_file_path.to_str().unwrap(), error),
                Ok(_) => ()
            }
        }

        let mut output_file: File = match File::create(output_file_path) {
            Err(error) => panic!("Could not create output file \"{0}\". Got error {1}", output_file_path.to_str().unwrap(), error),
            Ok(file_result) => file_result
        };

        match output_file.write(self.string_buffer.as_bytes()) {
            Err(error) => panic!("Could not write to \"{0}\" file. Got error {1}", self.file_name, error),
            Ok(_) => match output_file.flush() {
                Err(error) => panic!("Could not flush to \"{0}\" file. Got error {1}", self.file_name, error),
                Ok(_) => ()
            }
         }
    }
}
