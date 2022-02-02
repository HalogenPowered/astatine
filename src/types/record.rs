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

use bytes::{Buf, Bytes};
use internship::IStr;
use crate::class_file::attribute_tags::TAG_SIGNATURE;
use crate::class_file::parse_generic_signature;
use crate::utils::descriptors::FieldDescriptor;
use super::ConstantPool;

#[derive(Debug)]
pub struct RecordComponent {
    name: IStr,
    descriptor: FieldDescriptor,
    generic_signature: Option<IStr>
}

impl RecordComponent {
    pub(crate) fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid record component for class_file file {}! Expected name at \
                index {} in constant pool!", class_file_name, name_index));
        let descriptor_index = buf.get_u16();
        let descriptor = pool.get_utf8(descriptor_index as usize)
            .and_then(|value| FieldDescriptor::parse(value.as_str()))
            .expect(&format!("Invalid record component for class_file file {}! Expected \
                descriptor at index {} in constant pool!", class_file_name, name_index));

        let attribute_count = buf.get_u16();
        let generic_signature = parse_attributes(class_file_name, pool, buf, attribute_count);
        RecordComponent { name, descriptor, generic_signature }
    }

    pub fn new(name: &str, descriptor: FieldDescriptor, generic_signature: Option<&str>) -> Self {
        RecordComponent {
            name: IStr::new(name),
            descriptor,
            generic_signature: generic_signature.map(|value| IStr::new(value))
        }
    }

    // TODO: Procedural macros
    named!();
    describable!(FieldDescriptor);
    generic!();
}

fn parse_attributes(
    class_file_name: &str,
    pool: &ConstantPool,
    buf: &mut Bytes,
    mut attribute_count: u16
) -> Option<IStr> {
    let mut generic_signature = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated record component attributes for field in class \
            file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid record component attribute index {} in class_file file {}! \
                Expected name to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found for \
                record component in class_file file {}!", class_file_name);
            generic_signature = parse_generic_signature(class_file_name, pool, buf,
                                                        attribute_length, "record component");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    generic_signature
}
