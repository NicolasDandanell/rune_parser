use crate::{
    output::is_silent,
    scanner::NumericLiteral,
    types::{FieldIndex, Primitive},
    RuneFileDescription, RuneParserError
};

impl Primitive {
    pub fn can_back_bitfield(&self) -> bool {
        match self {
            Primitive::Char | Primitive::I8 | Primitive::U8 | Primitive::I16 | Primitive::U16 | Primitive::I32 | Primitive::U32 | Primitive::I64 | Primitive::U64 => true,

            // All other types are invalid
            _ => false
        }
    }

    pub fn can_back_enum(&self) -> bool {
        match self {
            Primitive::Bool
            | Primitive::Char
            | Primitive::I8
            | Primitive::U8
            | Primitive::I16
            | Primitive::U16
            | Primitive::F32
            | Primitive::I32
            | Primitive::U32
            | Primitive::F64
            | Primitive::I64
            | Primitive::U64 => true,

            // All other types are invalid
            _ => false
        }
    }

    pub fn validate_bit_index(&self, index: &u64) -> bool {
        match self {
            Primitive::Char | Primitive::I8 | Primitive::U8 => *index < 8,
            Primitive::I16 | Primitive::U16 => *index < 16,
            Primitive::I32 | Primitive::U32 => *index < 32,
            Primitive::I64 | Primitive::U64 => *index < 64,

            // All other types are invalid
            _ => false
        }
    }

    /// Used to validate whether the total size of a bitfield can fit within its backing type
    pub fn validate_bitfield_size(&self, bitfield_size: &u64) -> bool {
        match self {
            Primitive::Char | Primitive::I8 | Primitive::U8 => *bitfield_size <= 8,
            Primitive::I16 | Primitive::U16 => *bitfield_size <= 16,
            Primitive::I32 | Primitive::U32 => *bitfield_size <= 32,
            Primitive::I64 | Primitive::U64 => *bitfield_size <= 64,

            // All other types are invalid
            _ => false
        }
    }

    /// Used for enums to validate value against backing type
    pub fn validate_value(&self, value: &NumericLiteral) -> bool {
        match self {
            // Single Byte
            Primitive::Bool => matches!(value, NumericLiteral::Boolean(_)),

            Primitive::Char | Primitive::I8 => match value {
                NumericLiteral::PositiveInteger(value, _) => *value <= i8::MAX as u64,
                NumericLiteral::NegativeInteger(value, _) => Primitive::I8_RANGE.contains(value),
                _ => false
            },

            Primitive::U8 => match value {
                NumericLiteral::PositiveInteger(value, _) => Primitive::U8_RANGE.contains(value),
                _ => false
            },

            // Two Bytes
            Primitive::I16 => match value {
                // Positives
                NumericLiteral::PositiveInteger(value, _) => *value <= i16::MAX as u64,
                // Negatives
                NumericLiteral::NegativeInteger(value, _) => Primitive::I16_RANGE.contains(value),
                _ => false
            },
            Primitive::U16 => match value {
                NumericLiteral::PositiveInteger(value, _) => Primitive::U16_RANGE.contains(value),
                _ => false
            },

            // Four Bytes
            Primitive::F32 => match value {
                NumericLiteral::Float(float) => Primitive::F32_RANGE.contains(float),
                _ => false
            },
            Primitive::I32 => match value {
                // Positives
                NumericLiteral::PositiveInteger(value, _) => *value <= i32::MAX as u64,
                // Negatives
                NumericLiteral::NegativeInteger(value, _) => Primitive::I32_RANGE.contains(value),
                _ => false
            },
            Primitive::U32 => match value {
                NumericLiteral::PositiveInteger(value, _) => Primitive::U32_RANGE.contains(value),
                _ => false
            },

            // Eight Bytes - Make assumptions, as if the value would not fit, we would not even be able to parse it into the program...
            Primitive::F64 => matches!(value, NumericLiteral::Float(_)),

            Primitive::I64 => match value {
                // Positives
                NumericLiteral::PositiveInteger(value, _) => *value < i64::MAX as u64,
                // Negatives
                NumericLiteral::NegativeInteger(_, _) => true,
                _ => false
            },

            Primitive::U64 => matches!(value, NumericLiteral::PositiveInteger(_, _)),

            _ => unreachable!("Critical! Invalid backing type for enum encountered during verification. This should never happen!")
        }
    }
}

// Overall validation function
pub fn validate_parsed_files(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    info!("Validating declarations");

    // Validate all type names (Define, Bitfield, Enum, and Struct) against each other to check for collisions
    validate_names(files)?;

    // Validate bitfields
    validate_bitfields(files)?;

    // Validate defines - Not needed, as they are mere text replace, and thus have no backing type

    // Validate enums
    validate_enums(files)?;

    // Validate messages
    validate_messages(files)?;

    // Validate structs
    validate_structs(files)?;

    Ok(())
}

pub fn validate_names(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Assume there are 5 definitions per list
    let mut names_list: Vec<String> = Vec::with_capacity(files.len() * 5);

    // Get the names of all declared data types
    for file in files {
        // Bitfields
        for definition in &file.definitions.bitfields {
            names_list.push(definition.name.clone());
        }
        // Defines
        for definition in &file.definitions.defines {
            names_list.push(definition.name.clone());
        }
        // Enums
        for definition in &file.definitions.enums {
            names_list.push(definition.name.clone());
        }
        // Structs
        for definition in &file.definitions.structs {
            names_list.push(definition.name.clone());
        }
    }

    for i in 0..names_list.len() - 1 {
        if names_list[i + 1..].contains(&names_list[i]) {
            error!("Found two data types with the name {0}!", names_list[i]);
            return Err(RuneParserError::NameCollision);
        }
    }

    Ok(())
}

// Bitfield validation
// ————————————————————

/// Check that no two fields have the same index or identifier, and that the total size of the bitfield is valid
pub fn validate_bitfields(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check that there are no two bitfield fields that have the same identifier
    // No use of reserved indexes
    // No duplicate indexes
    // !!! Indexes against backing type with SIZES - No overflow !!!
    //  - Overall index check in done in parser, but it does not take field sizes into account

    // Check all files for struct definitions
    for file in files {
        for bitfield_definition in &file.definitions.bitfields {
            let mut total_size: u64 = 0;

            for member in &bitfield_definition.members {
                let index: u64 = member.index;
                let identifier: String = member.identifier.clone();

                // Add bit size to total
                total_size += member.size.absolute();

                // Check field index
                // ——————————————————

                let index_count = bitfield_definition.members.iter().filter(|&member| member.index == index).count();

                if index_count > 1 {
                    error!(
                        "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                        bitfield_definition.name, index
                    );
                    return Err(RuneParserError::IndexCollision);
                }

                if bitfield_definition.reserved_indexes.contains(&index) {
                    error!(
                        "Error at {0}: Field {1} was declared with index {2} is declared even though field index {2} is reserved",
                        bitfield_definition.name, identifier, index
                    );
                    return Err(RuneParserError::UseOfReservedIndex);
                }

                // Check field identifier
                // ———————————————————————

                let identifier_count = bitfield_definition.members.iter().filter(|&member| member.identifier == identifier).count();

                if identifier_count > 1 {
                    error!("Error at {0}: Found multiple definitions of identifier {1} in member fields", bitfield_definition.name, identifier);
                    return Err(RuneParserError::IdentifierCollision);
                }
            }

            // Check if bitfield members can fit within backing type
            if !bitfield_definition.backing_type.validate_bitfield_size(&total_size) {
                error!(
                    "Error at {0}: Total size of members ({1} bytes) cannot fit within backing type {2:?}",
                    bitfield_definition.name, total_size, bitfield_definition.backing_type
                );
                return Err(RuneParserError::InvalidTotalBitfieldSize);
            }
        }
    }

    Ok(())
}

// Enum validation
// ————————————————

/// Check that there are no two enum values that have the same identifier or value
pub fn validate_enums(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check that no two identifiers are the same
    // Check that not two values are the same
    // Check that no reserved value is being used
    // Check that all values are valid within backing type --> Done in parser

    // Check all files for enum definitions
    for file in files {
        for enum_definition in &file.definitions.enums {
            for member in &enum_definition.members {
                let value: NumericLiteral = member.value.clone();
                let identifier: String = member.identifier.clone();

                // Check field index for collisions or use of reserved values
                // ———————————————————————————————————————————————————————————

                let value_count: usize = enum_definition.members.iter().filter(|&member| member.value == value).count();

                if value_count > 1 {
                    error!(
                        "Error at {0}: Cannot have multiple enum members with the same value! Found multiple instances of value: {1}",
                        enum_definition.name,
                        value.to_string()
                    );
                    return Err(RuneParserError::ValueCollision);
                }

                if enum_definition.reserved_values.contains(&value) {
                    error!(
                        "Error at {0}: Enum member {1} was declared with value {2} even though value {2} is reserved",
                        enum_definition.name,
                        identifier,
                        value.to_string()
                    );
                    return Err(RuneParserError::UseOfReservedIndex);
                }

                // Check field identifier for collisions
                // ——————————————————————————————————————

                let identifier_count = enum_definition.members.iter().filter(|&member| member.identifier == identifier).count();

                if identifier_count > 1 {
                    error!("Error at {0}: Found multiple definitions of identifier {1} in member fields", enum_definition.name, identifier);
                    return Err(RuneParserError::IdentifierCollision);
                }
            }
        }
    }

    Ok(())
}

// Struct validation
// ——————————————————

/// Check that two fields do not have the same field index or identifier
pub fn validate_messages(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check all files for struct definitions
    for file in files {
        for message_definition in &file.definitions.messages {
            // Check whether a verification field has been declared
            let has_verifier: bool = match message_definition.fields.iter().filter(|&x| x.index.is_verifier()).count() {
                0 => false,
                1 => true,
                _ => {
                    error!("Error at {0}: Cannot have more than one verifier field per struct!", message_definition.name);
                    return Err(RuneParserError::IndexCollision);
                }
            };

            // Check all identifiers for collisions
            for field in &message_definition.fields {
                let index: FieldIndex = field.index.clone();
                let identifier: String = field.identifier.clone();

                // Check field index
                // ——————————————————

                let index_count = message_definition.fields.iter().filter(|&member| member.index.value() == index.value()).count();

                if index_count > 1 {
                    if index.value() == 0 && has_verifier {
                        error!(
                            "Error at {0}: Cannot have a verifier field and a field with index 0! This is due to verifier being an alias for index 0",
                            message_definition.name
                        );
                    } else {
                        error!(
                            "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                            message_definition.name,
                            index.value()
                        );
                    }
                    return Err(RuneParserError::IndexCollision);
                }

                if message_definition.reserved_indexes.contains(&index) {
                    error!(
                        "Error at {0}: Field {1} was declared with index {2} is declared even though field index {2} is reserved",
                        message_definition.name,
                        identifier,
                        index.value()
                    );
                    return Err(RuneParserError::UseOfReservedIndex);
                }

                // Check field identifier
                // ———————————————————————

                let identifier_count = message_definition.fields.iter().filter(|&member| member.identifier == identifier).count();

                if identifier_count > 1 {
                    error!("Error at {0}: Found multiple definitions of identifier {1} in message fields", message_definition.name, identifier);
                    return Err(RuneParserError::IdentifierCollision);
                }
            }
        }
    }

    Ok(())
}

// Struct validation
// ——————————————————

/// Check that two fields do not have the same field index or identifier
pub fn validate_structs(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check all files for struct definitions
    for file in files {
        for struct_definition in &file.definitions.structs {
            for member in &struct_definition.members {
                let index: u64 = member.index;
                let identifier: String = member.identifier.clone();

                // Check field index
                // ——————————————————

                let index_count = struct_definition.members.iter().filter(|&member| member.index == index).count();

                if index_count > 1 {
                    error!(
                        "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                        struct_definition.name, index
                    );

                    return Err(RuneParserError::IndexCollision);
                }

                // Check field identifier
                // ———————————————————————

                let identifier_count = struct_definition.members.iter().filter(|&member| member.identifier == identifier).count();

                if identifier_count > 1 {
                    error!("Error at {0}: Found multiple definitions of identifier {1} in struct members", struct_definition.name, identifier);
                    return Err(RuneParserError::IdentifierCollision);
                }
            }
        }
    }

    Ok(())
}
