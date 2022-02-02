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
use crate::types::ConstantPool;
use crate::types::constant_pool::UTF8_TAG;

pub(crate) fn parse_generic_signature(
    class_file_name: &str,
    pool: &ConstantPool,
    buf: &mut Bytes,
    length: u32,
    type_name: &str
) -> Option<IStr> {
    assert!(length == 2 || buf.len() < 2, "Invalid generic signature attribute for {} in \
        class file {}! Expected length of 2, was {}!", type_name, class_file_name, length);
    let index = buf.get_u16();
    assert!(pool.has(index as usize), "Invalid generic signature attribute for {} in class \
        file {}! Expected index {} to be in constant pool!", type_name, class_file_name, index);
    let tag = pool.get_tag(index as usize)
        .expect(&format!("Invalid generic signature attribute for {} in class file {}! Expected \
            value at index {} in constant pool!", type_name, class_file_name, index));
    assert_eq!(tag, UTF8_TAG, "Invalid generic signature attribute for {} in class file {}! \
        Expected UTF-8 string at {}, was {}!", type_name, class_file_name, index, tag);
    let value = pool.get_utf8(index as usize);
    assert!(value.is_some(), "Invalid {} in class file {}! Expected generic signature to be at \
        index {}!", type_name, class_file_name, index);
    value
}
