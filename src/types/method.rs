use bytes::{Buf, Bytes};
use java_desc::MethodType;
use crate::types::access_flags::*;
use crate::types::code::CodeBlock;
use crate::types::constant_pool::ConstantPool;
use crate::types::utils::{Generic, Nameable};

pub struct Method {
    pub name: String,
    pub descriptor: MethodType,
    pub generic_signature: Option<String>,
    pub access_flags: u16,
    pub parameters: Vec<MethodParameter>,
    pub code: Option<CodeBlock>,
    pub checked_exception_indices: Vec<u16>,
    pub other_flags: u8
}

// These aren't part of the spec, this is just the best way I could think of compactly storing extra flags.
pub const METHOD_IS_CONSTRUCTOR: u8 = 0x01;
pub const METHOD_IS_STATIC_INITIALIZER: u8 = 0x02;

impl Method {
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

impl Accessible for Method {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}

impl Nameable for Method {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Generic for Method {
    fn generic_signature(&self) -> Option<&str> {
        self.generic_signature.as_ref().map(|value| value.as_str())
    }
}

pub struct MethodParameter {
    name: String,
    pub access_flags: u16
}

impl MethodParameter {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid method parameter for method in class file {}! Expected name at \
                index {}!", class_file_name, name_index))
            .clone();
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

impl Nameable for MethodParameter {
    fn name(&self) -> &str {
        &self.name
    }
}
