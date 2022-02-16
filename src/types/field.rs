/*
 * Copyright (C) 2022 Callum Seabrook <callum.seabrook@prevarinite.com>
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; version 2.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program; if not, write to the Free Software Foundation, Inc., 51 Franklin
 * Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */

use astatine_macros::{FieldDescribable, Nameable, Generic, accessible};
use bytes::{Buf, Bytes};
use internship::IStr;
use crate::class_file::parse_generic_signature;
use crate::constants::*;
use crate::utils::descriptors::{FieldDescriptor, FieldType};
use super::access_flags::*;
use super::constant_pool::*;

#[accessible(final, public, private, protected, static, volatile, transient)]
#[derive(Debug, Nameable, FieldDescribable, Generic)]
pub struct Field {
    name: IStr,
    descriptor: FieldDescriptor,
    access_flags: AccessFlags,
    generic_signature: Option<IStr>,
    constant_value: Option<ConstantValue>
}

macro_rules! is_constant {
    ($name:ident, $return:ident, $constant_type:ident) => {
        pub fn $name(&self) -> Option<$return> {
            match self.constant_value() {
                Some(ConstantValue::$constant_type(value)) => Some(*value),
                _ => None
            }
        }
    }
}

const PUBLIC_STATIC_FINAL: u32 = JVM_ACC_PUBLIC | JVM_ACC_STATIC | JVM_ACC_FINAL;

impl Field {
    pub(crate) fn parse(
        pool: &ConstantPool,
        buf: &mut Bytes,
        major_version: u16,
        class_flags: AccessFlags
    ) -> Self {
        let access_flags = buf.get_u16() as u32;
        if class_flags.is_interface() {
            assert_eq!(access_flags, PUBLIC_STATIC_FINAL, "Invalid field! All fields in interfaces \
                must be public static final and not have any other modifiers!");
        }
        let name = pool.get_utf8(buf.get_u16() as usize)
            .expect("Invalid field! Expected name in constant pool!");
        let descriptor = pool.get_utf8(buf.get_u16() as usize)
            .and_then(|value| FieldDescriptor::parse(value.as_str()))
            .expect(&format!("Invalid field! Expected descriptor in constant pool!"));

        let access_flags = AccessFlags::from(access_flags);
        let attributes = parse_attributes(pool, buf, major_version, access_flags.is_static(), &descriptor);
        Field { name, descriptor, access_flags, generic_signature: attributes.1, constant_value: attributes.0 }
    }

    pub fn constant_value(&self) -> Option<&ConstantValue> {
        self.constant_value.as_ref()
    }

    is_constant!(constant_int, i32, Integer);
    is_constant!(constant_long, i64, Long);
    is_constant!(constant_float, f32, Float);
    is_constant!(constant_double, f64, Double);

    pub fn constant_string(&self) -> Option<IStr> {
        match self.constant_value() {
            Some(ConstantValue::String(value)) => Some(value.clone()),
            _ => None
        }
    }
}

type FieldAttributes = (Option<ConstantValue>, Option<IStr>);

fn parse_attributes(
    pool: &ConstantPool,
    buf: &mut Bytes,
    major_version: u16,
    is_static: bool,
    descriptor: &FieldDescriptor
) -> FieldAttributes {
    let mut constant_value = None;
    let mut generic_signature = None;

    let mut attributes_count = buf.get_u16();
    while attributes_count > 0 {
        assert!(buf.len() >= 6, "Truncated field attributes!");
        let attribute_name = pool.get_utf8(buf.get_u16() as usize).unwrap();
        let attribute_length = buf.get_u32();

        if is_static && attribute_name == JVM_ATTRIBUTE_CONSTANT_VALUE {
            if constant_value.is_some() {
                panic!("Duplicate ConstantValue attribute!")
            }
            assert_eq!(attribute_length, 2, "Invalid ConstantValue attribute! Expected length \
                of 2, was {}!", attribute_length);
            let constant_value_index = buf.get_u16();
            constant_value = ConstantValue::parse(pool, constant_value_index, descriptor);
        } else if attribute_name == JVM_ATTRIBUTE_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for field!", attribute_length);
        } else if attribute_name == JVM_ATTRIBUTE_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for field !", attribute_length);
        } else if major_version >= JAVA_VERSION_1_5 && attribute_name == JVM_ATTRIBUTE_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found for field!");
            generic_signature = parse_generic_signature(pool, buf, attribute_length, "field");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attributes_count -= 1;
    };
    (constant_value, generic_signature)
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(IStr)
}

const STRING_DESCRIPTOR: &str = "Ljava/lang/String;";

impl ConstantValue {
    fn parse(pool: &ConstantPool, index: u16, descriptor: &FieldDescriptor) -> Option<Self> {
        assert!(index > 0 && index < (pool.len() as u16), "Bad constant value! Failed to find \
            value at index {}!", index);
        let value_type = pool.get_tag(index as usize)
            .expect("Invalid field constant value! Expected tag for constant value!");
        match &descriptor.base() {
            FieldType::Long => {
                assert_eq!(value_type, LONG_TAG, "Inconsistent constant value type! Expected long!");
                pool.get_long(index as usize).map(|value| ConstantValue::Long(value))
            },
            FieldType::Float => {
                assert_eq!(value_type, FLOAT_TAG, "Inconsistent constant value type! Expected float!");
                pool.get_float(index as usize).map(|value| ConstantValue::Float(value))
            },
            FieldType::Double => {
                assert_eq!(value_type, DOUBLE_TAG, "Inconsistent constant value type! Expected double!");
                pool.get_double(index as usize).map(|value| ConstantValue::Double(value))
            },
            FieldType::Byte | FieldType::Char | FieldType::Short | FieldType::Boolean |
            FieldType::Int => {
                assert_eq!(value_type, INT_TAG, "Inconsistent constant value type! Expected integer");
                pool.get_int(index as usize).map(|value| ConstantValue::Integer(value))
            },
            FieldType::Reference(name) => {
                assert!(value_type == CLASS_TAG && name == STRING_DESCRIPTOR, "Inconsistent \
                    constant value type or descriptor! Expected string!");
                pool.get_utf8(index as usize).map(|value| ConstantValue::String(value))
            }
        }
    }
}
