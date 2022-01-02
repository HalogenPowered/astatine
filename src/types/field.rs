use crate::types::access_flags::*;

pub struct Field {
    pub name: str,
    pub descriptor: str,
    pub signature: str,
    pub access_flags: u16
}

impl Field {
    pub fn is_volatile(&self) -> bool {
        self.access_flags & ACC_VOLATILE != 0
    }

    pub fn is_transient(&self) -> bool {
        self.access_flags & ACC_TRANSIENT != 0
    }

    pub fn is_enum_constant(&self) -> bool {
        self.access_flags & ACC_ENUM != 0
    }
}

impl Accessible for Field {
    fn flags(&self) -> u16 {
        self.access_flags
    }
}
