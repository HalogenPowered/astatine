use bytes::{Buf, Bytes};
use java_desc::MethodType;
use crate::types::access_flags::*;
use crate::types::code::CodeBlock;
use crate::types::constant_pool::ConstantPool;

pub struct Method<'a> {
    pub name: &'a str,
    pub descriptor: MethodType,
    pub generic_signature: Option<&'a str>,
    pub access_flags: u16,
    pub parameters: Vec<MethodParameter>,
    pub code: Option<CodeBlock>,
    pub checked_exception_indices: Vec<u16>,
    pub other_flags: u8
}

// These aren't part of the spec, this is just the best way I could think of compactly storing extra flags.
pub const METHOD_IS_CONSTRUCTOR: u8 = 0x01;
pub const METHOD_IS_STATIC_INITIALIZER: u8 = 0x02;

impl Method<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_constructor(&self) -> bool {
        self.other_flags & METHOD_IS_CONSTRUCTOR != 0
    }

    pub fn is_static_initializer(&self) -> bool {
        self.other_flags & METHOD_IS_STATIC_INITIALIZER != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.access_flags & ACC_SYNCHRONIZED != 0
    }

    pub fn is_bridge(&self) -> bool {
        self.access_flags & ACC_BRIDGE != 0
    }

    pub fn is_varargs(&self) -> bool {
        self.access_flags & ACC_VARARGS != 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags & ACC_NATIVE != 0
    }

    pub fn is_strict(&self) -> bool {
        self.access_flags & ACC_STRICT != 0
    }
}

impl Accessible for Method<'_> {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}

pub struct MethodParameter {
    pub name: *const String,
    pub access_flags: u16
}

impl MethodParameter {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid method parameter for method in class file {}! Expected name at \
                index {}!", class_file_name, name_index));
        let access_flags = buf.get_u16();
        MethodParameter { name, access_flags }
    }

    pub fn is_final(&self) -> bool {
        self.access_flags & ACC_FINAL != 0
    }

    pub fn is_synthetic(&self) -> bool {
        self.access_flags & ACC_SYNTHETIC != 0
    }

    pub fn is_mandated(&self) -> bool {
        self.access_flags & ACC_MANDATED != 0
    }
}

impl Accessible for MethodParameter {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}
