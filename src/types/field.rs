use java_desc::FieldType;
use crate::types::access_flags::*;

pub struct Field<'a> {
    pub name: &'a str,
    pub descriptor: FieldType,
    pub generic_signature: Option<&'a str>,
    pub access_flags: u16,
    pub constant_value: Option<ConstantValue>
}

impl Field<'_> {
    pub fn is_volatile(&self) -> bool {
        self.access_flags & ACC_VOLATILE != 0
    }

    pub fn is_transient(&self) -> bool {
        self.access_flags & ACC_TRANSIENT != 0
    }

    pub fn is_enum_constant(&self) -> bool {
        self.access_flags & ACC_ENUM != 0
    }
}

impl Accessible for Field<'_> {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}

pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String)
}
