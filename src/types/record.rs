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

use astatine_macros::{Nameable, FieldDescribable, Generic};
use bytes::{Buf, Bytes};
use internship::IStr;
use crate::class_file::parse_generic_signature;
use crate::constants::JVM_ATTRIBUTE_SIGNATURE;
use crate::utils::descriptors::FieldDescriptor;
use super::ConstantPool;

#[derive(Debug, Nameable, FieldDescribable, Generic)]
pub struct RecordComponent {
    name: IStr,
    descriptor: FieldDescriptor,
    generic_signature: Option<IStr>
}

impl RecordComponent {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name = pool.get_utf8(buf.get_u16() as usize)
            .expect("Invalid record component! Expected name in constant pool!");
        let descriptor = pool.get_utf8(buf.get_u16() as usize)
            .and_then(|value| FieldDescriptor::parse(value.as_str()))
            .expect("Invalid record component! Expected descriptor in constant pool!");
        let generic_signature = parse_attributes(pool, buf);
        RecordComponent { name, descriptor, generic_signature }
    }
}

fn parse_attributes(pool: &ConstantPool, buf: &mut Bytes) -> Option<IStr> {
    let mut generic_signature = None;

    let mut attribute_count = buf.get_u16();
    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated record component attributes!");
        let attribute_name = pool.get_utf8(buf.get_u16() as usize).unwrap();
        let attribute_length = buf.get_u32();

        if attribute_name == JVM_ATTRIBUTE_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found for \
                record component!");
            generic_signature = parse_generic_signature(pool, buf, attribute_length, "record component");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    generic_signature
}
