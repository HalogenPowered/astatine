use bytes::{Buf, Bytes};
use java_desc::{FieldType, SingleType};
use crate::class_file::attribute_tags::*;
use crate::class_file::utils::parse_generic_signature;
use crate::utils::constants::JAVA_VERSION_1_5;
use crate::types::access_flags::ACC_STATIC;
use crate::types::constant_pool::{CLASS_TAG, ConstantPool, DOUBLE_TAG, FLOAT_TAG, INT_TAG, LONG_TAG};
use crate::types::field::{ConstantValue, Field};

pub fn parse_field(class_file_name: &str, major_version: u16, pool: &ConstantPool, buf: &mut Bytes) -> Field {
    let access_flags = buf.get_u16();
    let name_index = buf.get_u16();
    let name = pool.get_string(name_index as usize)
        .expect(&format!("Invalid field in class_file file {}! Expected name at index {} in \
            constant pool!", class_file_name, name_index))
        .clone();

    let descriptor_index = buf.get_u16();
    let descriptor_string = pool.get_utf8(descriptor_index as usize)
        .expect(&format!("Invalid field in class_file file {}! Expected descriptor at index {} in \
            constant pool!", class_file_name, descriptor_index));
    let descriptor = FieldType::parse(descriptor_string)
        .expect(&format!("Invalid descriptor for field in class_file file {}!", class_file_name));

    let attributes_count = buf.get_u16();
    let is_static = access_flags & ACC_STATIC != 0;
    let (constant_value, generic_signature) = parse_attributes(
        class_file_name,
        major_version,
        attributes_count,
        is_static,
        &descriptor,
        pool,
        buf
    );
    Field::new(name, descriptor, generic_signature, access_flags, constant_value)
}

fn parse_attributes(
    class_file_name: &str,
    major_version: u16,
    mut attributes_count: u16,
    is_static: bool,
    descriptor: &FieldType,
    pool: &ConstantPool,
    buf: &mut Bytes
) -> (Option<ConstantValue>, Option<String>) {
    let mut constant_value = None;
    let mut generic_signature = None;

    while attributes_count > 0 {
        assert!(buf.len() >= 6, "Truncated field attributes for field in class_file file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid field attribute index {} in class_file file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if is_static && attribute_name == TAG_CONSTANT_VALUE {
            if constant_value.is_some() {
                panic!("Duplicate ConstantValue attribute!")
            }
            assert_eq!(attribute_length, 2, "Invalid ConstantValue attribute! Expected length of 2, \
                    was {} for class_file file {}!", attribute_length, class_file_name);
            let constant_value_index = buf.get_u16();
            constant_value = parse_constant_value(class_file_name, pool, constant_value_index, descriptor);
        } else if attribute_name == TAG_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for field in class_file \
                file {}!", attribute_length, class_file_name);
        } else if attribute_name == TAG_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for field in class_file \
                file {}!", attribute_length, class_file_name);
        } else if major_version >= JAVA_VERSION_1_5 {
            if attribute_name == TAG_SIGNATURE {
                assert!(generic_signature.is_none(), "Duplicate generic signature attribute found \
                    for field in class_file file {}!", class_file_name);
                generic_signature = parse_generic_signature(class_file_name, pool, attribute_length, buf, "field");
            }
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attributes_count -= 1;
    };
    (constant_value, generic_signature)
}

const STRING_DESCRIPTOR: &str = "Ljava/lang/String;";

fn parse_constant_value(class_file_name: &str, pool: &ConstantPool, index: u16, descriptor: &FieldType) -> Option<ConstantValue> {
    assert!(index > 0 && index < (pool.len() as u16), "Bad constant value! Failed to find value at index {}!", index);

    let value_type = pool.get_tag(index as usize)
        .expect(&format!("Invalid constant value for field in class_file file {}! Expected tag for \
            constant value index {}!", class_file_name, index));
    match &descriptor.base {
        SingleType::Long => {
            assert_eq!(value_type, &LONG_TAG, "Inconsistent constant value type! Expected long!");
            pool.get_long(index as usize).map(|value| ConstantValue::Long(*value))
        },
        SingleType::Float => {
            assert_eq!(value_type, &FLOAT_TAG, "Inconsistent constant value type! Expected float!");
            pool.get_float(index as usize).map(|value| ConstantValue::Float(*value))
        },
        SingleType::Double => {
            assert_eq!(value_type, &DOUBLE_TAG, "Inconsistent constant value type! Expected double!");
            pool.get_double(index as usize).map(|value| ConstantValue::Double(*value))
        },
        SingleType::Byte | SingleType::Char | SingleType::Short | SingleType::Boolean | SingleType::Int => {
            assert_eq!(value_type, &INT_TAG, "Inconsistent constant value");
            pool.get_int(index as usize).map(|value| ConstantValue::Integer(*value))
        },
        SingleType::Reference(name) => {
            assert!(value_type == &CLASS_TAG && name == STRING_DESCRIPTOR, "Invalid initial string value!");
            pool.get_utf8(index as usize).map(|value| ConstantValue::String(String::from(value)))
        }
    }
}
