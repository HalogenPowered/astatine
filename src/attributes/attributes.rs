use bytes::{Buf, Bytes};

use crate::attributes::stack_map_frames::StackMapFrame;
use crate::class_structures::{AttributeInfo, ConstantPool, ConstantPoolEntry, Utf8ConstantInfo};

pub trait Attribute {
    fn name(&self, pool: &ConstantPool) -> &str;

    fn name_internal(index: &u16, pool: &ConstantPool) -> &str {
        let entry: ConstantPoolEntry = pool[index];
        let value: Option<&str> = match entry.info {
            Utf8ConstantInfo(_, value) => Option::Some(value),
            _ => Option::None
        };
        value.expect("Expected type at " + index + " to be a UTF-8 string, got " + entry.tag)
    }

    fn read_from(mut buf: &Bytes) -> Box<Self> {
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        Box::new(read_from_internal(attribute_name_index, attribute_length))
    }

    fn read_from_internal(attribute_name_index: u16, attribute_length: u32, mut buf: &Bytes) -> Self;
}

pub struct ConstantValueAttribute {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub constant_value_index: u16
}

impl Attribute for ConstantValueAttribute {
    fn name(&self, pool: &ConstantPool) -> &str {
        name_internal(self.attribute_name_index, pool)
    }

    fn read_from_internal(attribute_name_index: u16, attribute_length: u32, mut buf: &Bytes) -> Self {
        let constant_value_index = buf.get_u16();
        ConstantValueAttribute { attribute_name_index, attribute_length, constant_value_index }
    }
}
