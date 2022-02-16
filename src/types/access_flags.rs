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

use paste::paste;
use crate::constants::*;

// Flags actually found in class files
pub const JVM_ACC_WRITTEN_FLAGS: u32 = 0x00007FFF;

// Method flags
pub const JVM_ACC_HAS_LINE_NUMBER_TABLE: u32 = 0x00100000;
pub const JVM_ACC_HAS_CHECKED_EXCEPTIONS: u32 = 0x00400000;
pub const JVM_ACC_CONSTRUCTOR: u32 = 0x10000000;
pub const JVM_ACC_STATIC_INITIALIZER: u32 = 0x20000000;

// Class and Method flags
pub const JVM_ACC_HAS_LOCAL_VARIABLE_TABLE: u32 = 0x00400000;

// Field flags
pub const JVM_ACC_FIELD_INTERNAL: u32 = 0x00000400;
pub const JVM_ACC_FIELD_STABLE: u32 = 0x00000020;
pub const JVM_ACC_FIELD_HAS_GENERIC_SIGNATURE: u32 = 0x00000800;

pub const JVM_RECOGNIZED_CLASS_MODIFIERS: u32 = JVM_ACC_PUBLIC | JVM_ACC_FINAL | JVM_ACC_SUPER |
    JVM_ACC_INTERFACE | JVM_ACC_ABSTRACT | JVM_ACC_ANNOTATION | JVM_ACC_ENUM | JVM_ACC_SYNTHETIC;
pub const JVM_RECOGNIZED_FIELD_MODIFIERS: u32 = JVM_ACC_PUBLIC | JVM_ACC_PRIVATE |
    JVM_ACC_PROTECTED | JVM_ACC_STATIC | JVM_ACC_FINAL | JVM_ACC_VOLATILE | JVM_ACC_TRANSIENT |
    JVM_ACC_ENUM | JVM_ACC_SYNTHETIC;
pub const JVM_RECOGNIZED_METHOD_MODIFIERS: u32 = JVM_ACC_PUBLIC | JVM_ACC_PRIVATE |
    JVM_ACC_PROTECTED | JVM_ACC_STATIC | JVM_ACC_FINAL | JVM_ACC_SYNCHRONIZED | JVM_ACC_BRIDGE |
    JVM_ACC_VARARGS | JVM_ACC_NATIVE | JVM_ACC_ABSTRACT | JVM_ACC_STRICT | JVM_ACC_SYNTHETIC;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(transparent)]
pub struct AccessFlags {
    flags: u32
}

macro_rules! is_flag {
    ($name:ident) => {
        paste! {
            #[inline]
            pub fn [<is_ $name>](&self) -> bool {
                self.flags & [<JVM_ACC_ $name:upper>] != 0
            }
        }
    }
}

impl AccessFlags {
    pub const fn new(flags: u32) -> Self {
        AccessFlags { flags }
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.flags & JVM_ACC_WRITTEN_FLAGS
    }

    is_flag!(public);
    is_flag!(private);
    is_flag!(protected);
    is_flag!(static);
    is_flag!(final);
    is_flag!(synchronized);
    is_flag!(super);
    is_flag!(volatile);
    is_flag!(transient);
    is_flag!(native);
    is_flag!(interface);
    is_flag!(abstract);
    is_flag!(synthetic);
}

impl Default for AccessFlags {
    fn default() -> Self {
        AccessFlags::new(0)
    }
}

macro_rules! impl_from {
    ($T:ident) => {
        impl From<$T> for AccessFlags {
            fn from(value: $T) -> Self {
                AccessFlags::new(value as u32)
            }
        }
    }
}

impl_from!(u8);
impl_from!(u16);
impl_from!(u32);
impl_from!(u64);
impl_from!(u128);
impl_from!(i8);
impl_from!(i16);
impl_from!(i32);
impl_from!(i64);
impl_from!(i128);
