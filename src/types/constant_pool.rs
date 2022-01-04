use bytes::{Buf, Bytes};
use crate::types::class::Class;
use crate::types::method_handle::MethodHandle;

pub struct ConstantPool {
    entries: Vec<PoolConstant>
}

impl ConstantPool {
    pub fn get(&self, index: usize) -> Option<&PoolConstant> {
        self.entries.get(index)
    }

    pub fn get_utf8(&self, index: usize) -> Option<&str> {
        match self.entries.get(index) {
            Some(PoolConstant::Utf8(value)) => Some(value),
            _ => None
        }
    }

    pub fn get_int(&self, index: usize) -> Option<&i32> {
        match self.get(index) {
            Some(PoolConstant::Integer(value)) => Some(&value),
            _ => None
        }
    }

    pub fn get_float(&self, index: usize) -> Option<&f32> {
        match self.entries.get(index) {
            Some(PoolConstant::Float(value)) => Some(value),
            _ => None
        }
    }

    pub fn get_long(&self, index: usize) -> Option<&i64> {
        match self.entries.get(index) {
            Some(PoolConstant::Long(value)) => Some(value),
            _ => None
        }
    }

    pub fn get_double(&self, index: usize) -> Option<&f64> {
        match self.entries.get(index) {
            Some(PoolConstant::Double(value)) => Some(value),
            _ => None
        }
    }

    pub fn resolve_class_name(&self, index: usize) -> Option<&str> {
        match self.entries.get(index) {
            Some(PoolConstant::Class(name_index)) => self.get_utf8(name_index as usize),
            _ => None
        }
    }

    pub fn resolve_string(&self, index: usize) -> Option<&str> {
        match self.entries.get(index) {
            Some(PoolConstant::String(value_index)) => self.get_utf8(value_index as usize),
            _ => None
        }
    }
}

pub fn read_constant_pool(buf: &mut Bytes) -> Vec<PoolConstant> {
    let count = buf.get_u16();
    let mut pool = Vec::with_capacity(count as usize);
    for _ in 0..count {
        pool.entries.push(PoolConstant::parse(buf));
    }
    pool
}

const UTF8_TAG: u8 = 1;
const INT_TAG: u8 = 3;
const FLOAT_TAG: u8 = 4;
const LONG_TAG: u8 = 5;
const DOUBLE_TAG: u8 = 6;
const CLASS_TAG: u8 = 7;
const STRING_TAG: u8 = 8;
const FIELD_REF_TAG: u8 = 9;
const METHOD_REF_TAG: u8 = 10;
const INTERFACE_METHOD_REF_TAG: u8 = 11;
const NAME_AND_TYPE_TAG: u8 = 12;
const METHOD_HANDLE_TAG: u8 = 15;
const METHOD_TYPE_TAG: u8 = 16;
const DYNAMIC_TAG: u8 = 17;
const INVOKE_DYNAMIC_TAG: u8 = 18;
const MODULE_TAG: u8 = 19;
const PACKAGE_TAG: u8 = 20;

impl PoolConstant {
    fn parse(buf: &mut Bytes) -> Self {
        let tag = buf.get_u8();
        match tag {
            UTF8_TAG => PoolConstant::Utf8(PoolConstant::parse_utf8(buf)),
            INT_TAG => PoolConstant::Integer(buf.get_i32()),
            FLOAT_TAG => PoolConstant::Float(buf.get_f32()),
            LONG_TAG => PoolConstant::Long(buf.get_i64()),
            DOUBLE_TAG => PoolConstant::Double(buf.get_f64()),
            CLASS_TAG => PoolConstant::Class { name_index: buf.get_u16() },
            STRING_TAG => PoolConstant::String { value_index: buf.get_u16() },
            FIELD_REF_TAG => PoolConstant::FieldRef { class_index: buf.get_u16(), name_and_type_index: buf.get_u16() },
            METHOD_REF_TAG => PoolConstant::MethodRef { class_index: buf.get_u16(), name_and_type_index: buf.get_u16() },
            INTERFACE_METHOD_REF_TAG => PoolConstant::InterfaceMethodRef { class_index: buf.get_u16(), name_and_type_index: buf.get_u16() },
            NAME_AND_TYPE_TAG => PoolConstant::NameAndType { name_index: buf.get_u16(), descriptor_index: buf.get_u16() },
            METHOD_HANDLE_TAG => PoolConstant::MethodHandle { reference_kind: buf.get_u8(), reference_index: buf.get_u16() },
            METHOD_TYPE_TAG => PoolConstant::MethodType { descriptor_index: buf.get_u16() },
            DYNAMIC_TAG => PoolConstant::Dynamic { bootstrap_method_attr_index: buf.get_u16(), name_and_type_index: buf.get_u16() },
            INVOKE_DYNAMIC_TAG => PoolConstant::InvokeDynamic { bootstrap_method_attr_index: buf.get_u16(), name_and_type_index: buf.get_u16() },
            MODULE_TAG => PoolConstant::Module { name_index: buf.get_u16() },
            PACKAGE_TAG => PoolConstant::Package { name_index: buf.get_u16() },
            _ => panic!("Invalid tag {} for constant pool entry!", tag)
        }
    }

    fn parse_utf8(buf: &mut Bytes) -> String {
        let length = buf.get_u16();
        let bytes = buf.copy_to_bytes(length as usize).to_vec();
        String::from_utf8(bytes).expect("Failed to convert bytes to string!")
    }
}

pub enum PoolConstant {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class { name_index: u16 },
    String { value_index: u16 },
    FieldRef { class_index: u16, name_and_type_index: u16 },
    MethodRef { class_index: u16, name_and_type_index: u16 },
    InterfaceMethodRef { class_index: u16, name_and_type_index: u16 },
    NameAndType { name_index: u16, descriptor_index: u16 },
    MethodHandle { reference_kind: u8, reference_index: u16 },
    MethodType { descriptor_index: u16 },
    Dynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    Module { name_index: u16 },
    Package { name_index: u16 }
}
