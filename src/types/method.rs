use crate::types::access_flags::*;

pub struct Method {
    pub name: str,
    pub descriptor: str,
    pub signature: str,
    pub access_flags: u16,
    pub parameters: Vec<MethodParameter>,
    pub code: Vec<u8>
}

impl Method {
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

pub struct MethodParameter {
    pub name: Option<str>,
    pub access_flags: u16
}

impl MethodParameter {
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
