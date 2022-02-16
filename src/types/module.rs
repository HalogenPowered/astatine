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

use astatine_macros::{Nameable, Versioned};
use bytes::{Buf, Bytes};
use internship::IStr;
use crate::constants::JAVA_VERSION_10;
use crate::utils::BufferExtras;
use super::access_flags::*;
use super::ConstantPool;
use super::constant_pool::{MODULE_TAG, PACKAGE_TAG};

macro_rules! mandated {
    () => {
        pub fn is_mandated(&self) -> bool {
            self.access_flags.value() & ACC_MANDATED != 0
        }
    }
}

#[derive(Debug, Nameable, Versioned)]
pub struct Module {
    name: IStr,
    access_flags: AccessFlags,
    version: Option<IStr>,
    requires: Vec<ModuleRequires>,
    exports: Vec<ModuleExports>,
    opens: Vec<ModuleOpens>,
    uses: Vec<u16>,
    provides: Vec<ModuleProvides>
}

const JAVA_BASE_NAME: &str = "java.base";
const ACC_OPEN: u32 = 0x0020;
const ACC_MANDATED: u32 = 0x8000;

impl Module {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes, major_version: u16) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid module! Expected name at index {} in constant pool!", name_index));

        let access_flags = AccessFlags::from(buf.get_u16());
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module! Expected value for \
                non-zero version index {}!", version_index);
        }

        let requires_count = buf.get_u16();
        if requires_count != 0 && name == JAVA_BASE_NAME {
            panic!("Invalid java.base module! The base module cannot have requirements!");
        }
        if requires_count == 0 && name != JAVA_BASE_NAME {
            panic!("Invalid module {}! All modules other than the base module must explicitly \
                require the base module!", name);
        }

        let requires = buf.get_generic_array(requires_count as usize, |buf| {
            ModuleRequires::parse(pool, buf, major_version, name.as_str())
        });
        let exports = buf.get_generic_u16_array(|buf| ModuleExports::parse(pool, buf));
        let opens = buf.get_generic_u16_array(|buf| ModuleOpens::parse(pool, buf));
        let uses = buf.get_u16_array();
        let provides = buf.get_generic_u16_array(|buf| ModuleProvides::parse(pool, buf));
        Module { name, access_flags, version, requires, exports, opens, uses, provides }
    }

    pub fn requires(&self) -> &[ModuleRequires] {
        self.requires.as_slice()
    }

    pub fn exports(&self) -> &[ModuleExports] {
        self.exports.as_slice()
    }

    pub fn opens(&self) -> &[ModuleOpens] {
        self.opens.as_slice()
    }

    pub fn uses(&self) -> &[u16] {
        self.uses.as_slice()
    }

    pub fn provides(&self) -> &[ModuleProvides] {
        self.provides.as_slice()
    }

    pub fn is_open(&self) -> bool {
        self.access_flags.value() & ACC_OPEN != 0
    }

    mandated!();
}

#[derive(Debug, Versioned)]
pub struct ModuleRequires {
    module_index: u16,
    access_flags: AccessFlags,
    version: Option<IStr>
}

impl ModuleRequires {
    pub(crate) fn parse(
        pool: &ConstantPool,
        buf: &mut Bytes,
        major_version: u16,
        module_name: &str
    ) -> Self {
        let module_index = read_module_index(pool, buf);
        let access_flags = AccessFlags::from(buf.get_u16());
        check_requires_flags(module_name, major_version, access_flags.value());
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module requirement! Expected \
                value for non-zero version index {}!", version_index);
        }
        ModuleRequires { module_index, access_flags, version }
    }

    pub fn module_index(&self) -> u16 {
        self.module_index
    }

    mandated!();
}

const ACC_STATIC_PHASE: u32 = 0x0040;
const ACC_TRANSITIVE: u32 = 0x0020;

fn check_requires_flags(module_name: &str, major_version: u16, flags: u32) {
    if module_name == JAVA_BASE_NAME || major_version < JAVA_VERSION_10 {
        return;
    }
    assert!(flags & ACC_TRANSITIVE == 0 && flags & ACC_STATIC_PHASE == 0, "Invalid module requires \
        flags! ACC_TRANSITIVE ({}) and ACC_STATIC_PHASE ({}) cannot be set for requirements on \
        modules other than java.base for class files from Java 10 or \
        later!", ACC_TRANSITIVE, ACC_STATIC_PHASE);
}

macro_rules! common_exports_opens {
    ($T:ident) => {
        #[derive(Debug)]
        pub struct $T {
            package_index: u16,
            access_flags: AccessFlags,
            to_indices: Vec<u16>
        }

        impl $T {
            pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
                $T {
                    package_index: read_package_index(pool, buf),
                    access_flags: AccessFlags::from(buf.get_u16()),
                    to_indices: buf.get_u16_array()
                }
            }

            pub fn package_index(&self) -> u16 {
                self.package_index
            }

            pub fn is_qualified(&self) -> bool {
                !self.to_indices.is_empty()
            }

            pub fn to_indices(&self) -> &[u16] {
                self.to_indices.as_slice()
            }

            mandated!();
        }
    }
}


common_exports_opens!(ModuleExports);
common_exports_opens!(ModuleOpens);

#[derive(Debug)]
pub struct ModuleProvides {
    module_index: u16,
    with_indices: Vec<u16>
}

impl ModuleProvides {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        ModuleProvides { module_index: read_module_index(pool, buf), with_indices: buf.get_u16_array() }
    }

    pub fn module_index(&self) -> u16 {
        self.module_index
    }
}

macro_rules! generate_index_reader {
    ($name:ident, $index_name:literal, $tag:expr) => {
        fn $name(pool: &ConstantPool, buf: &mut Bytes) -> u16 {
            let index = buf.get_u16();
            assert!(pool.has(index as usize), "Invalid {} index for module part! Expected index to \
                be in constant pool!", $index_name);
            let tag = pool.get_tag(index as usize)
                .expect(&format!("Expected tag for module part {} index!", $index_name));
            assert_eq!(tag, $tag, "Expected {} at index for module part {} index!", $index_name, $index_name);
            index
        }
    }
}

generate_index_reader!(read_module_index, "module", MODULE_TAG);
generate_index_reader!(read_package_index, "package", PACKAGE_TAG);
