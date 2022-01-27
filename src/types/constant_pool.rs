use std::sync::Arc;
use bytes::{Buf, Bytes};
use internship::IStr;
use crate::{Class, ClassLoader};

macro_rules! get_constant {
    ($name:ident, $ty:ty, $constant_name:ident) => {
        pub fn $name(&self, index: usize) -> Option<$ty> {
            match self.get(index) {
                Some(PoolConstant::$constant_name(value)) => Some(*value),
                _ => None
            }
        }
    }
}

#[derive(Debug)]
pub struct ConstantPool {
    tags: Vec<u8>,
    constants: Vec<PoolConstant>
}

impl ConstantPool {
    pub(crate) fn parse(buf: &mut Bytes) -> Self {
        let count = buf.get_u16();
        let mut tags = Vec::with_capacity(count as usize);
        let mut constants = Vec::with_capacity(count as usize);
        for _ in 0..count - 1 {
            let tag = buf.get_u8();
            tags.push(tag);
            constants.push(PoolConstant::parse(tag, buf));
        }
        // No funny business on my watch!
        assert_eq!(tags.len(), constants.len(), "Tags and constants size mismatch!");
        ConstantPool::new(tags, constants)
    }

    pub const fn new(tags: Vec<u8>, constants: Vec<PoolConstant>) -> Self {
        ConstantPool { tags, constants }
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn has(&self, index: usize) -> bool {
        index < self.tags.len()
    }

    pub fn get_tag(&self, index: usize) -> Option<u8> {
        self.tags.get(index - 1).map(|value| *value)
    }

    fn get(&self, index: usize) -> Option<&PoolConstant> {
        self.constants.get(index - 1)
    }

    pub fn get_utf8(&self, index: usize) -> Option<&str> {
        self.get_string(index).map(|value| value.as_str())
    }

    // Same as get_utf8, but returns the underlying IStr object, rather than a splice
    pub fn get_string(&self, index: usize) -> Option<IStr> {
        match self.get(index) {
            Some(PoolConstant::Utf8(value)) => Some(value.clone()),
            _ => None
        }
    }

    get_constant!(get_int, i32, Integer);
    get_constant!(get_float, f32, Float);
    get_constant!(get_long, i64, Long);
    get_constant!(get_double, f64, Double);

    pub fn resolve_class_name(&self, index: usize) -> Option<IStr> {
        match self.get(index) {
            Some(PoolConstant::Class { name_index }) => self.get_string(*name_index as usize),
            _ => None
        }
    }

    pub fn resolve_class(&self, index: usize, loader: &ClassLoader) -> Option<Arc<Class>> {
        match self.get(index) {
            Some(PoolConstant::Class { name_index }) => self.get_string(*name_index as usize)
                .map(|name| loader.load_class(name.as_str())),
            _ => None
        }
    }
}

pub const UTF8_TAG: u8 = 1;
pub const INT_TAG: u8 = 3;
pub const FLOAT_TAG: u8 = 4;
pub const LONG_TAG: u8 = 5;
pub const DOUBLE_TAG: u8 = 6;
pub const CLASS_TAG: u8 = 7;
pub const STRING_TAG: u8 = 8;
pub const FIELD_REF_TAG: u8 = 9;
pub const METHOD_REF_TAG: u8 = 10;
pub const INTERFACE_METHOD_REF_TAG: u8 = 11;
pub const NAME_AND_TYPE_TAG: u8 = 12;
pub const METHOD_HANDLE_TAG: u8 = 15;
pub const METHOD_TYPE_TAG: u8 = 16;
pub const DYNAMIC_TAG: u8 = 17;
pub const INVOKE_DYNAMIC_TAG: u8 = 18;
pub const MODULE_TAG: u8 = 19;
pub const PACKAGE_TAG: u8 = 20;

#[derive(Debug)]
pub enum PoolConstant {
    Utf8(IStr),
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

impl PoolConstant {
    fn parse(tag: u8, buf: &mut Bytes) -> Self {
        match tag {
            UTF8_TAG => PoolConstant::Utf8(PoolConstant::parse_utf8(buf)),
            INT_TAG => PoolConstant::Integer(buf.get_i32()),
            FLOAT_TAG => PoolConstant::Float(buf.get_f32()),
            LONG_TAG => PoolConstant::Long(buf.get_i64()),
            DOUBLE_TAG => PoolConstant::Double(buf.get_f64()),
            CLASS_TAG => PoolConstant::Class { name_index: buf.get_u16() },
            STRING_TAG => PoolConstant::String { value_index: buf.get_u16() },
            FIELD_REF_TAG => PoolConstant::FieldRef {
                class_index: buf.get_u16(),
                name_and_type_index: buf.get_u16()
            },
            METHOD_REF_TAG => PoolConstant::MethodRef {
                class_index: buf.get_u16(),
                name_and_type_index: buf.get_u16()
            },
            INTERFACE_METHOD_REF_TAG => PoolConstant::InterfaceMethodRef {
                class_index: buf.get_u16(),
                name_and_type_index: buf.get_u16()
            },
            NAME_AND_TYPE_TAG => PoolConstant::NameAndType {
                name_index: buf.get_u16(),
                descriptor_index: buf.get_u16()
            },
            METHOD_HANDLE_TAG => PoolConstant::MethodHandle {
                reference_kind: buf.get_u8(),
                reference_index: buf.get_u16()
            },
            METHOD_TYPE_TAG => PoolConstant::MethodType { descriptor_index: buf.get_u16() },
            DYNAMIC_TAG => PoolConstant::Dynamic {
                bootstrap_method_attr_index: buf.get_u16(),
                name_and_type_index: buf.get_u16()
            },
            INVOKE_DYNAMIC_TAG => PoolConstant::InvokeDynamic {
                bootstrap_method_attr_index: buf.get_u16(),
                name_and_type_index: buf.get_u16()
            },
            MODULE_TAG => PoolConstant::Module { name_index: buf.get_u16() },
            PACKAGE_TAG => PoolConstant::Package { name_index: buf.get_u16() },
            _ => panic!("Invalid tag {} for constant pool entry!", tag)
        }
    }

    fn parse_utf8(buf: &mut Bytes) -> IStr {
        let length = buf.get_u16();
        let bytes = buf.copy_to_bytes(length as usize).to_vec();
        IStr::from_utf8(bytes.as_slice()).expect("Failed to convert bytes to string!")
    }
}
