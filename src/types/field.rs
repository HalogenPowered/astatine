use bytes::{Buf, Bytes};
use internship::IStr;
use java_desc::{FieldType, SingleType};
use super::access_flags::*;
use super::constant_pool::*;
use crate::class_file::attribute_tags::*;
use crate::class_file::utils::parse_generic_signature;
use crate::class_file::version::ClassFileVersion;

#[derive(Debug)]
pub struct Field {
    name: IStr,
    descriptor: FieldType,
    generic_signature: Option<IStr>,
    access_flags: u16,
    constant_value: Option<ConstantValue>
}

macro_rules! is_constant {
    ($name:ident, $return:ident, $constant_type:ident) => {
        pub fn $name(&self) -> Option<$return> {
            match &self.constant_value {
                Some(ConstantValue::$constant_type(value)) => Some(*value),
                _ => None
            }
        }
    }
}

pub(self) const PUBLIC_STATIC_FINAL: u16 = ACC_PUBLIC | ACC_STATIC | ACC_FINAL;

impl Field {
    pub(crate) fn parse(
        class_file_name: &str,
        pool: &ConstantPool,
        buf: &mut Bytes,
        version: &ClassFileVersion,
        class_flags: u16
    ) -> Self {
        let access_flags = buf.get_u16();
        if class_flags & ACC_INTERFACE != 0 {
            assert_eq!(access_flags, PUBLIC_STATIC_FINAL, "Invalid field in class file {}! All \
                fields in interfaces must be public static final and not have any other \
                modifiers!", class_file_name);
        }

        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid field in class file {}! Expected name at index {} in \
                constant pool!", class_file_name, name_index));

        let descriptor_index = buf.get_u16();
        let descriptor_string = pool.get_utf8(descriptor_index as usize)
            .expect(&format!("Invalid field in class file {}! Expected descriptor at index {} in \
                constant pool!", class_file_name, descriptor_index));
        let descriptor = FieldType::parse(descriptor_string)
            .expect(&format!("Invalid descriptor for field in class file {}!", class_file_name));

        let attributes_count = buf.get_u16();
        let is_static = access_flags & ACC_STATIC != 0;
        let (constant_value, generic_signature) = parse_attributes(
            class_file_name,
            pool,
            buf,
            version,
            attributes_count,
            is_static,
            &descriptor
        );
        Field::new(name, descriptor, generic_signature, access_flags, constant_value)
    }

    pub fn new(
        name: IStr,
        descriptor: FieldType,
        generic_signature: Option<IStr>,
        access_flags: u16,
        constant_value: Option<ConstantValue>
    ) -> Self {
        Field { name, descriptor, generic_signature, access_flags, constant_value }
    }

    pub fn constant_value(&self) -> Option<&ConstantValue> {
        self.constant_value.as_ref()
    }

    is_constant!(constant_int, i32, Integer);
    is_constant!(constant_long, i64, Long);
    is_constant!(constant_float, f32, Float);
    is_constant!(constant_double, f64, Double);

    pub fn constant_string(&self) -> Option<IStr> {
        match &self.constant_value {
            Some(ConstantValue::String(value)) => Some(value.clone()),
            _ => None
        }
    }

    pub fn is_volatile(&self) -> bool {
        self.access_flags & ACC_VOLATILE != 0
    }

    pub fn is_transient(&self) -> bool {
        self.access_flags & ACC_TRANSIENT != 0
    }
}

impl_field!(Field);
impl_accessible!(Field);
impl_accessible!(Field, FinalAccessible);
impl_accessible!(Field, PublicAccessible);
impl_accessible!(Field, EnumAccessible);
impl_accessible!(Field, PrivateProtectedStaticAccessible);

fn parse_attributes(
    class_file_name: &str,
    pool: &ConstantPool,
    buf: &mut Bytes,
    version: &ClassFileVersion,
    mut attributes_count: u16,
    is_static: bool,
    descriptor: &FieldType
) -> (Option<ConstantValue>, Option<IStr>) {
    let mut constant_value = None;
    let mut generic_signature = None;

    while attributes_count > 0 {
        assert!(buf.len() >= 6, "Truncated field attributes for field in class file {}!",
                class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid field attribute index {} in class file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if is_static && attribute_name == TAG_CONSTANT_VALUE {
            if constant_value.is_some() {
                panic!("Duplicate ConstantValue attribute!")
            }
            assert_eq!(attribute_length, 2, "Invalid ConstantValue attribute! Expected length \
                of 2, was {} for class file {}!", attribute_length, class_file_name);
            let constant_value_index = buf.get_u16();
            constant_value = ConstantValue::parse(class_file_name, pool, constant_value_index,
                                                  descriptor);
        } else if attribute_name == TAG_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for field in \
                class file {}!", attribute_length, class_file_name);
        } else if attribute_name == TAG_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for field in \
                class file {}!", attribute_length, class_file_name);
        } else if version >= &ClassFileVersion::RELEASE_1_5 {
            if attribute_name == TAG_SIGNATURE {
                assert!(generic_signature.is_none(), "Duplicate generic signature attribute found \
                    for field in class file {}!", class_file_name);
                generic_signature = parse_generic_signature(class_file_name, pool, buf,
                                                            attribute_length, "field");
            }
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attributes_count -= 1;
    };
    (constant_value, generic_signature)
}

#[derive(Debug)]
pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(IStr)
}

pub(self) const STRING_DESCRIPTOR: &str = "Ljava/lang/String;";

impl ConstantValue {
    fn parse(
        class_file_name: &str,
        pool: &ConstantPool,
        index: u16,
        descriptor: &FieldType
    ) -> Option<Self> {
        assert!(index > 0 && index < (pool.len() as u16), "Bad constant value! Failed to find \
            value at index {}!", index);

        let value_type = pool.get_tag(index as usize)
            .expect(&format!("Invalid constant value for field in class file {}! Expected tag for \
                constant value index {}!", class_file_name, index));
        match &descriptor.base {
            SingleType::Long => {
                assert_eq!(value_type, LONG_TAG, "Inconsistent constant value type! Expected \
                    long!");
                pool.get_long(index as usize).map(|value| ConstantValue::Long(value))
            },
            SingleType::Float => {
                assert_eq!(value_type, FLOAT_TAG, "Inconsistent constant value type! Expected \
                    float!");
                pool.get_float(index as usize).map(|value| ConstantValue::Float(value))
            },
            SingleType::Double => {
                assert_eq!(value_type, DOUBLE_TAG, "Inconsistent constant value type! Expected \
                    double!");
                pool.get_double(index as usize).map(|value| ConstantValue::Double(value))
            },
            SingleType::Byte | SingleType::Char | SingleType::Short | SingleType::Boolean |
            SingleType::Int => {
                assert_eq!(value_type, INT_TAG, "Inconsistent constant value type! Expected \
                    integer");
                pool.get_int(index as usize).map(|value| ConstantValue::Integer(value))
            },
            SingleType::Reference(name) => {
                assert!(value_type == CLASS_TAG && name == STRING_DESCRIPTOR, "Inconsistent \
                    constant value type or descriptor! Expected string!");
                pool.get_string(index as usize).map(|value| ConstantValue::String(value))
            }
        }
    }
}
