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

use std::fmt::{Display, Formatter};

pub const T_BOOLEAN: u8 = 4;
pub const T_CHAR: u8 = 5;
pub const T_FLOAT: u8 = 6;
pub const T_DOUBLE: u8 = 7;
pub const T_BYTE: u8 = 8;
pub const T_SHORT: u8 = 9;
pub const T_INT: u8 = 10;
pub const T_LONG: u8 = 11;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ArrayType {
    Boolean = T_BOOLEAN,
    Char = T_CHAR,
    Float = T_FLOAT,
    Double = T_DOUBLE,
    Byte = T_BYTE,
    Short = T_SHORT,
    Int = T_INT,
    Long = T_LONG
}

impl ArrayType {
    pub fn from(value: u8) -> ArrayType {
        match value {
            T_BOOLEAN => ArrayType::Boolean,
            T_CHAR => ArrayType::Char,
            T_FLOAT => ArrayType::Float,
            T_DOUBLE => ArrayType::Double,
            T_BYTE => ArrayType::Byte,
            T_SHORT => ArrayType::Short,
            T_INT => ArrayType::Int,
            T_LONG => ArrayType::Long,
            _ => panic!("Invalid array type {}!", value)
        }
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
