use bytes::{Buf, Bytes};

use crate::types::access_flags::{ACC_MANDATED, ACC_OPEN, ACC_SYNTHETIC};
use crate::types::constant_pool::{ConstantPool, MODULE_TAG};

pub struct Module<'a> {
    pub name: &'a str,
    pub flags: u16,
    pub version: Option<&'a str>,
    pub requires: Vec<ModuleRequires<'a>>,
    pub exports: Vec<ModuleExports>,
    pub opens: Vec<ModuleOpens>,
    pub uses: Vec<u16>,
    pub provides: Vec<ModuleProvides>
}

impl<'a> Module<'a> {
    pub fn parse(class_file_name: &str, pool: &'a ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid module for class file {}! Expected name at index {} in constant \
                pool!", class_file_name, name_index));
        let flags = buf.get_u16();
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module in class file {}! Expected \
                value for non-zero version index {}!", class_file_name, version_index);
        }

        let requires_count = buf.get_u16();
        let mut requires = Vec::with_capacity(requires_count as usize);
        for _ in 0..requires_count {
            requires.push(ModuleRequires::parse(class_file_name, pool, buf));
        }
        let exports_count = buf.get_u16();
        let mut exports = Vec::with_capacity(exports_count as usize);
        for _ in 0..exports_count {
            exports.push(ModuleExports::parse(class_file_name, pool, buf));
        }
        let opens_count = buf.get_u16();
        let mut opens = Vec::with_capacity(opens_count as usize);
        for _ in 0..opens_count {
            opens.push(ModuleOpens::parse(class_file_name, pool, buf));
        }
        let uses_count = buf.get_u16();
        let mut uses = Vec::with_capacity(uses_count as usize);
        for _ in 0..uses_count {
            uses.push(buf.get_u16());
        }
        let provides_count = buf.get_u16();
        let mut provides = Vec::with_capacity(provides_count as usize);
        for _ in 0..provides_count {
            provides.push(ModuleProvides::parse(class_file_name, pool, buf));
        }
        Module { name, flags, version, requires, exports, opens, uses, provides }
    }

    pub fn is_open(&self) -> bool {
        self.flags & ACC_OPEN != 0
    }

    pub fn is_synthetic(&self) -> bool {
        self.flags & ACC_SYNTHETIC != 0
    }

    pub fn is_mandated(&self) -> bool {
        self.flags & ACC_MANDATED != 0
    }
}

pub trait ModuleComponent {
    fn module_index(&self) -> u16;

    fn flags(&self) -> u16;

    fn is_synthetic(&self) -> bool {
        self.flags() & ACC_SYNTHETIC != 0
    }

    fn is_mandated(&self) -> bool {
        self.flags() & ACC_MANDATED != 0
    }
}

pub struct ModuleRequires<'a> {
    pub module_index: u16,
    pub flags: u16,
    pub version: Option<&'a str>
}

impl<'a> ModuleRequires<'a> {
    pub fn parse(class_file_name: &str, pool: &'a ConstantPool, buf: &mut Bytes) -> Self {
        let module_index = read_module_index(class_file_name, pool, buf);
        let flags = buf.get_u16();
        let version_index = buf.get_u16();
        let version = pool.get_utf8(version_index as usize);
        if version_index != 0 {
            assert!(version.is_some(), "Invalid version attribute for module requirement in class \
                file {}! Expected value for non-zero version index {}!", class_file_name, version_index);
        }
        ModuleRequires { module_index, flags, version }
    }
}

impl ModuleComponent for ModuleRequires<'_> {
    fn module_index(&self) -> u16 {
        self.module_index
    }

    fn flags(&self) -> u16 {
        self.flags
    }
}

pub struct ModuleExports {
    pub module_index: u16,
    pub flags: u16,
    pub to_indices: Vec<u16>
}

impl ModuleExports {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let module_index = read_module_index(class_file_name, pool, buf);
        let flags = buf.get_u16();
        let to_count = buf.get_u16();
        let mut to_indices = Vec::with_capacity(to_count as usize);
        for _ in 0..to_count {
            to_indices.push(buf.get_u16());
        }
        ModuleExports { module_index, flags, to_indices }
    }
}

impl ModuleComponent for ModuleExports {
    fn module_index(&self) -> u16 {
        self.module_index
    }

    fn flags(&self) -> u16 {
        self.flags
    }
}

pub struct ModuleOpens {
    pub module_index: u16,
    pub flags: u16,
    pub to_indices: Vec<u16>
}

impl ModuleOpens {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let module_index = read_module_index(class_file_name, pool, buf);
        let flags = buf.get_u16();
        let to_count = buf.get_u16();
        let mut to_indices = Vec::with_capacity(to_count as usize);
        for _ in 0..to_count {
            to_indices.push(buf.get_u16());
        }
        ModuleOpens { module_index, flags, to_indices }
    }
}

impl ModuleComponent for ModuleOpens {
    fn module_index(&self) -> u16 {
        self.module_index
    }

    fn flags(&self) -> u16 {
        self.flags
    }
}

pub struct ModuleProvides {
    pub module_index: u16,
    pub with_indices: Vec<u16>
}

impl ModuleProvides {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let module_index = read_module_index(class_file_name, pool, buf);
        let with_count = buf.get_u16();
        let mut with_indices = Vec::with_capacity(with_count as usize);
        for _ in 0..with_count {
            with_indices.push(buf.get_u16());
        }
        ModuleProvides { module_index, with_indices }
    }
}

impl ModuleComponent for ModuleProvides {
    fn module_index(&self) -> u16 {
        self.module_index
    }

    fn flags(&self) -> u16 {
        0
    }
}

fn read_module_index<'a>(class_file_name: &str, pool: &'a ConstantPool, buf: &mut Bytes) -> u16 {
    let index = buf.get_u16();
    assert!(pool.has(index as usize), "Invalid module index for module part in \
            class file {}! Expected index {} to be in constant pool!", class_file_name, index);
    let tag = pool.get_tag(index as usize)
        .expect(&format!("Invalid module index for module part in class file {}! Expected \
            tag at index {}!", class_file_name, index));
    assert_eq!(tag, &MODULE_TAG, "Invalid module index for module part in class file {}! \
        Expected module at index {}!", class_file_name, index);
    index
}
