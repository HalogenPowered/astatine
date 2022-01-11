use java_desc::FieldType;

use crate::types::access_flags::*;
use crate::types::utils::{Generic, Nameable};

pub struct Field {
    name: String,
    pub descriptor: FieldType,
    generic_signature: Option<String>,
    pub access_flags: u16,
    pub constant_value: Option<ConstantValue>
}

impl Field {
    pub const fn new(
        name: String,
        descriptor: FieldType,
        generic_signature: Option<String>,
        access_flags: u16,
        constant_value: Option<ConstantValue>
    ) -> Self {
        Field { name, descriptor, generic_signature, access_flags, constant_value }
    }

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

impl Accessible for Field {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}

impl Nameable for Field {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Generic for Field {
    fn generic_signature(&self) -> Option<&str> {
        self.generic_signature.as_ref().map(|value| value.as_str())
    }
}

pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String)
}
