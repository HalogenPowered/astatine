use bytes::{Buf, Bytes};
use crate::types::constant_pool::{ConstantPool, PoolConstant};
use crate::types::access_flags::*;

pub struct Field {
    pub name: String,
    pub descriptor: String,
    pub access_flags: u16,
    pub signature: String,
    pub constant_value: Option<ConstantValue>
}

impl Field {
    pub fn parse(pool: &ConstantPool, mut buf: &Bytes) -> Self {
        let access_flags = buf.get_u16();
        let name_index = buf.get_u16();
        let descriptor_index = buf.get_u16();
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

pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64)
}
