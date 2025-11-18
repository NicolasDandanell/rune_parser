use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    // 1 byte primitives
    Bool,
    Char,
    I8,
    U8,

    // 2 byte primitives
    I16,
    U16,

    // 4 byte primitives
    F32,
    I32,
    U32,

    // 8 byte primitives
    F64,
    I64,
    U64,

    // 16 byte primitives (Not sendable as primitive)
    I128,
    U128
}

impl Primitive {
    // Single Byte
    pub const I8_RANGE: Range<i64> = (i8::MIN as i64)..(i8::MAX as i64);
    pub const U8_RANGE: Range<u64> = (u8::MIN as u64)..(u8::MAX as u64);

    // Two Bytes
    pub const I16_RANGE: Range<i64> = (i16::MIN as i64)..(i16::MAX as i64);
    pub const U16_RANGE: Range<u64> = (u16::MIN as u64)..(u16::MAX as u64);

    // Four Bytes
    pub const F32_RANGE: Range<f64> = (f32::MIN as f64)..(f32::MAX as f64);
    pub const I32_RANGE: Range<i64> = (i32::MIN as i64)..(i32::MAX as i64);
    pub const U32_RANGE: Range<u64> = (u32::MIN as u64)..(u32::MAX as u64);

    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Primitive::Char | Primitive::I8 | Primitive::I16 | Primitive::F32 | Primitive::I32 | Primitive::F64 | Primitive::I64 | Primitive::I128
        )
    }

    pub fn encoded_max_data_size(&self) -> u64 {
        match self {
            Primitive::Bool | Primitive::Char | Primitive::I8 | Primitive::U8 => 1,
            Primitive::I16 | Primitive::U16 => 2,
            Primitive::F32 | Primitive::I32 | Primitive::U32 => 4,
            Primitive::F64 | Primitive::I64 | Primitive::U64 => 8,
            Primitive::I128 | Primitive::U128 => 16
        }
    }
}
