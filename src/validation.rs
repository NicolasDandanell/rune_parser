use crate::types::FieldSlot;
use crate::RuneFileDescription;

/// Check that two fields do not have the same field index, considering that VerificationField is an alias for index 0
pub fn validate_struct_indexes(files: &Vec<RuneFileDescription>) {

    // Check all files for struct definitions
    for file in files {
        for struct_definition in &file.definitions.structs {

            // Check whether a verification field has been declared
            let mut has_verification: bool = false;

            let mut index_list: Vec<u8> = Vec::with_capacity(0x20);

            for member in &struct_definition.members {
                let index: u8 = match member.field_slot {
                    FieldSlot::NamedSlot(value) => value as u8,
                    FieldSlot::VerificationField       => {
                        if has_verification {
                            panic!("Cannot have more than one VerificationField!");
                        } else {
                            has_verification = true;
                            0
                        }
                    }
                };

                if index_list.contains(&index) {
                    if (index == 0) && (has_verification) {
                        panic!("Cannot have a VerificationField and a field with index 0! This is due to VerificationField being an alias for index 0");
                    } else {
                        panic!("Cannot have multiple fields with the same index! Found multiple instances of index: {0}", index);
                    }
                }

                index_list.push(index);
            }
        }
    }
}