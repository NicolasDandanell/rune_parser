use std::ops::Range;

use crate::{
    output::is_silent,
    scanner::NumericLiteral,
    types::{FieldSlot, FieldType},
    RuneFileDescription, RuneParserError
};

impl FieldType {
    // Single Byte
    const BYTE_RANGE: Range<i64> = (i8::MIN as i64)..(i8::MAX as i64);
    const UBYTE_RANGE: Range<u64> = (u8::MIN as u64)..(u8::MAX as u64);

    // Two Bytes
    const SHORT_RANGE: Range<i64> = (i16::MIN as i64)..(i16::MAX as i64);
    const USHORT_RANGE: Range<u64> = (u16::MIN as u64)..(u16::MAX as u64);

    // Four Bytes
    const FLOAT_RANGE: Range<f64> = (f32::MIN as f64)..(f32::MAX as f64);
    const INT_RANGE: Range<i64> = (i32::MIN as i64)..(i32::MAX as i64);
    const UINT_RANGE: Range<u64> = (u32::MIN as u64)..(u32::MAX as u64);

    pub fn can_back_bitfield(&self) -> bool {
        match self {
            FieldType::Char | FieldType::UByte | FieldType::Byte | FieldType::UShort | FieldType::Short | FieldType::UInt | FieldType::Int | FieldType::ULong | FieldType::Long => true,

            // All other types are invalid
            _ => false
        }
    }

    pub fn can_back_enum(&self) -> bool {
        match self {
            FieldType::Boolean
            | FieldType::Char
            | FieldType::UByte
            | FieldType::Byte
            | FieldType::UShort
            | FieldType::Short
            | FieldType::Float
            | FieldType::UInt
            | FieldType::Int
            | FieldType::Double
            | FieldType::ULong
            | FieldType::Long => true,

            // All other types are invalid
            _ => false
        }
    }

    pub fn validate_bit_slot(&self, slot: &u64) -> bool {
        match self {
            FieldType::Char | FieldType::UByte | FieldType::Byte => *slot < 8,
            FieldType::UShort | FieldType::Short => *slot < 16,
            FieldType::UInt | FieldType::Int => *slot < 32,
            FieldType::ULong | FieldType::Long => *slot < 64,

            // All other types are invalid
            _ => false
        }
    }

    /// Used to validate whether the total size of a bitfield can fit within its backing type
    pub fn validate_bitfield_size(&self, bitfield_size: &u64) -> bool {
        match self {
            FieldType::Char | FieldType::UByte | FieldType::Byte => *bitfield_size <= 8,
            FieldType::UShort | FieldType::Short => *bitfield_size <= 16,
            FieldType::UInt | FieldType::Int => *bitfield_size <= 32,
            FieldType::ULong | FieldType::Long => *bitfield_size <= 64,

            // All other types are invalid
            _ => false
        }
    }

    /// Used for enums to validate value against backing type
    pub fn validate_value(&self, value: &NumericLiteral) -> bool {
        match self {
            // Single Byte
            FieldType::Boolean => match value {
                NumericLiteral::Boolean(_) => true,
                _ => false
            },

            FieldType::Char | FieldType::Byte => match value {
                // Positives
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value <= i8::MAX as u64,
                // Negatives
                NumericLiteral::NegativeBinary(value) | NumericLiteral::NegativeDecimal(value) | NumericLiteral::NegativeHexadecimal(value) => FieldType::BYTE_RANGE.contains(value),
                _ => false
            },

            FieldType::UByte => match value {
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => FieldType::UBYTE_RANGE.contains(value),
                _ => false
            },

            // Two Bytes
            FieldType::Short => match value {
                // Positives
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value <= i16::MAX as u64,
                // Negatives
                NumericLiteral::NegativeBinary(value) | NumericLiteral::NegativeDecimal(value) | NumericLiteral::NegativeHexadecimal(value) => FieldType::SHORT_RANGE.contains(value),
                _ => false
            },
            FieldType::UShort => match value {
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => FieldType::USHORT_RANGE.contains(value),
                _ => false
            },

            // Four Bytes
            FieldType::Float => match value {
                NumericLiteral::Float(float) => FieldType::FLOAT_RANGE.contains(float),
                _ => false
            },
            FieldType::Int => match value {
                // Positives
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value <= i32::MAX as u64,
                // Negatives
                NumericLiteral::NegativeBinary(value) | NumericLiteral::NegativeDecimal(value) | NumericLiteral::NegativeHexadecimal(value) => FieldType::INT_RANGE.contains(value),
                _ => false
            },
            FieldType::UInt => match value {
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => FieldType::UINT_RANGE.contains(value),
                _ => false
            },

            // Eight Bytes - Make assumptions, as if the value would not fit, we would not even be able to parse it into the program...
            FieldType::Double => match value {
                NumericLiteral::Float(_) => true,
                _ => false
            },

            FieldType::Long => match value {
                // Positives
                NumericLiteral::PositiveBinary(value) | NumericLiteral::PositiveDecimal(value) | NumericLiteral::PositiveHexadecimal(value) => *value < i64::MAX as u64,
                // Negatives
                NumericLiteral::NegativeBinary(_) | NumericLiteral::NegativeDecimal(_) | NumericLiteral::NegativeHexadecimal(_) => true,
                _ => false
            },
            FieldType::ULong => match value {
                NumericLiteral::PositiveBinary(_) | NumericLiteral::PositiveDecimal(_) | NumericLiteral::PositiveHexadecimal(_) => true,
                _ => false
            },

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
            let mut total_bit_size: u64 = 0;

            for member in &bitfield_definition.members {
                let field_slot: u64 = member.bit_slot;
                let identifier: String = member.identifier.clone();

                // Add bit size to total
                total_bit_size += member.bit_size.absolute();

                // Check field index
                // ——————————————————

                let slot_count = bitfield_definition.members.iter().filter(|&member| member.bit_slot == field_slot).count();

                if slot_count > 1 {
                    error!(
                        "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                        bitfield_definition.name, field_slot
                    );
                    return Err(RuneParserError::IndexCollision);
                }

                if bitfield_definition.reserved_slots.contains(&field_slot) {
                    error!(
                        "Error at {0}: Field {1} was declared with index {2} is declared even though field index {2} is reserved",
                        bitfield_definition.name, identifier, field_slot
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
            if !bitfield_definition.backing_type.validate_bitfield_size(&total_bit_size) {
                error!(
                    "Error at {0}: Total size of members ({1} bytes) cannot fit within backing type {2}",
                    bitfield_definition.name,
                    total_bit_size,
                    bitfield_definition.backing_type.to_string()
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
pub fn validate_structs(files: &Vec<RuneFileDescription>) -> Result<(), RuneParserError> {
    // Check all files for struct definitions
    for file in files {
        for struct_definition in &file.definitions.structs {
            // Check whether a verification field has been declared
            let has_verifier: bool = match struct_definition.members.iter().filter(|&x| x.field_slot.is_verifier() == true).count() {
                0 => false,
                1 => true,
                _ => {
                    error!("Error at {0}: Cannot have more than one verifier field per struct!", struct_definition.name);
                    return Err(RuneParserError::IndexCollision);
                }
            };

            // Check all identifiers for collisions
            for member in &struct_definition.members {
                let field_slot: FieldSlot = member.field_slot.clone();
                let identifier: String = member.identifier.clone();

                // Check field index
                // ——————————————————

                let slot_count = struct_definition.members.iter().filter(|&member| member.field_slot.value() == field_slot.value()).count();

                if slot_count > 1 {
                    if field_slot.value() == 0 && has_verifier {
                        error!(
                            "Error at {0}: Cannot have a verifier field and a field with index 0! This is due to verifier being an alias for index 0",
                            struct_definition.name
                        );
                    } else {
                        error!(
                            "Error at {0}: Cannot have multiple fields with the same index! Found multiple instances of index: {1}",
                            struct_definition.name,
                            field_slot.value()
                        );
                    }
                    return Err(RuneParserError::IndexCollision);
                }

                if struct_definition.reserved_slots.contains(&field_slot) {
                    error!(
                        "Error at {0}: Field {1} was declared with index {2} is declared even though field index {2} is reserved",
                        struct_definition.name,
                        identifier,
                        field_slot.value()
                    );
                    return Err(RuneParserError::UseOfReservedIndex);
                }

                // Check field identifier
                // ———————————————————————

                let identifier_count = struct_definition.members.iter().filter(|&member| member.identifier == identifier).count();

                if identifier_count > 1 {
                    error!("Error at {0}: Found multiple definitions of identifier {1} in member fields", struct_definition.name, identifier);
                    return Err(RuneParserError::IdentifierCollision);
                }
            }
        }
    }

    Ok(())
}
