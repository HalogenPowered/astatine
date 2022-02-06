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

use astatine_macros::{Nameable, Versioned, accessible};
use bytes::{Buf, Bytes};
use internship::IStr;
use crate::class_file::ClassFileVersion;
use crate::utils::BufferExtras;
use super::access_flags::*;
use super::ConstantPool;
use super::constant_pool::{MODULE_TAG, PACKAGE_TAG};

#[accessible(mandated)]
#[derive(Debug, Nameable, Versioned)]
pub struct Module {
    name: IStr,
    access_flags: u16,
    version: Option<IStr>,
    requires: Vec<ModuleRequires>,
    exports: Vec<ModuleExports>,
    opens: Vec<ModuleOpens>,
    uses: Vec<u16>,
    provides: Vec<ModuleProvides>
}

const JAVA_BASE_NAME: &str = "java.base";

impl Module {
    pub(crate) fn parse(
        class_file_name: &str,
        pool: &ConstantPool,
        buf: &mut Bytes,
        class_version: &ClassFileVersion
    ) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid module for class file {}! Expected name at index {} in \
                constant pool!", class_file_name, name_index));

        let access_flags = buf.get_u16();
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module in class file {}! \
                Expected value for non-zero version index {}!", class_file_name, version_index);
        }

        let requires_count = buf.get_u16();
        if requires_count != 0 && name == JAVA_BASE_NAME {
            panic!("Invalid java.base module in class file {}! The base module cannot have \
                requirements!", class_file_name);
        }
        if requires_count == 0 && name != JAVA_BASE_NAME {
            panic!("Invalid module {} in class file {}! All modules other than the base module \
                must explicitly require the base module!", name, class_file_name);
        }

        let requires = buf.get_generic_array(requires_count as usize, |buf| {
            ModuleRequires::parse(class_file_name, pool, buf, class_version, &name)
        });
        let exports = buf.get_generic_u16_array(|buf| {
            ModuleExports::parse(class_file_name, pool, buf)
        });
        let opens = buf.get_generic_u16_array(|buf| ModuleOpens::parse(class_file_name, pool, buf));
        let uses = buf.get_u16_array();
        let provides = buf.get_generic_u16_array(|buf| {
            ModuleProvides::parse(class_file_name, pool, buf)
        });
        Module { name, access_flags, version, requires, exports, opens, uses, provides }
    }

    pub fn new(
        name: &str,
        access_flags: u16,
        version: Option<&str>,
        requires: Vec<ModuleRequires>,
        exports: Vec<ModuleExports>,
        opens: Vec<ModuleOpens>,
        uses: Vec<u16>,
        provides: Vec<ModuleProvides>
    ) -> Module {
        Module {
            name: IStr::new(name),
            access_flags,
            version: version.map(|value| IStr::new(value)),
            requires,
            exports,
            opens,
            uses,
            provides
        }
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
        self.access_flags & ACC_OPEN != 0
    }
}

#[accessible(mandated)]
#[derive(Debug, Versioned)]
pub struct ModuleRequires {
    module_index: u16,
    access_flags: u16,
    version: Option<IStr>
}

impl ModuleRequires {
    pub(crate) fn parse(
        class_file_name: &str,
        pool: &ConstantPool,
        buf: &mut Bytes,
        version: &ClassFileVersion,
        module_name: &str
    ) -> Self {
        let module_index = read_module_index(class_file_name, pool, buf);
        let access_flags = buf.get_u16();
        check_requires_flags(module_name, version, access_flags);
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module requirement in class \
                file {}! Expected value for non-zero version index {}!", class_file_name, version_index);
        }
        ModuleRequires { module_index, access_flags, version }
    }

    pub fn new(module_index: u16, access_flags: u16, version: Option<&str>) -> Self {
        ModuleRequires {
            module_index,
            access_flags,
            version: version.map(|value| IStr::new(value))
        }
    }

    pub fn module_index(&self) -> u16 {
        self.module_index
    }
}

fn check_requires_flags(module_name: &str, version: &ClassFileVersion, flags: u16) {
    if module_name != JAVA_BASE_NAME && version >= &ClassFileVersion::RELEASE_10 {
        assert!(flags & ACC_TRANSIENT == 0 && flags & ACC_STATIC_PHASE == 0, "Invalid module \
            requires flags! ACC_TRANSITIVE ({}) and ACC_STATIC_PHASE ({}) cannot be set for \
            requirements on modules other than java.base for class files from Java 10 or \
            later!", ACC_TRANSIENT, ACC_STATIC_PHASE);
    }
}

macro_rules! common_exports_opens {
    ($T:ident) => {
        #[accessible(mandated)]
        #[derive(Debug)]
        pub struct $T {
            package_index: u16,
            access_flags: u16,
            to_indices: Vec<u16>
        }

        impl $T {
            pub(crate) fn parse(
                class_file_name: &str,
                pool: &ConstantPool,
                buf: &mut Bytes
            ) -> Self {
                $T::new(read_package_index(class_file_name, pool, buf), buf.get_u16(),
                    buf.get_u16_array())
            }

            pub const fn new(package_index: u16, access_flags: u16, to_indices: Vec<u16>) -> Self {
                $T { package_index, access_flags, to_indices }
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
    pub(crate) fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        ModuleProvides::new(read_module_index(class_file_name, pool, buf), buf.get_u16_array())
    }

    pub const fn new(module_index: u16, with_indices: Vec<u16>) -> Self {
        ModuleProvides { module_index, with_indices }
    }

    pub fn module_index(&self) -> u16 {
        self.module_index
    }
}

macro_rules! generate_index_reader {
    ($name:ident, $index_name:literal, $tag:expr) => {
        fn $name(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> u16 {
            let index = buf.get_u16();
            assert!(pool.has(index as usize), "Invalid $index_name index for module part \
                in class file {}! Expected index {} to be in constant \
                pool!", class_file_name, index);
            let tag = pool.get_tag(index as usize)
                .expect(&format!("Invalid {} index for module part in class file {}! \
                    Expected tag at index {}!", $index_name, class_file_name, index));
            assert_eq!(tag, $tag, "Invalid {} index for module part in class file {}! \
                Expected {} at index {}!", $index_name, class_file_name, $index_name, index);
            index
        }
    }
}

generate_index_reader!(read_module_index, "module", MODULE_TAG);
generate_index_reader!(read_package_index, "package", PACKAGE_TAG);
