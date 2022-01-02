// Global
pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_ABSTRACT: u16 = 0x0400;
pub const ACC_SYNTHETIC: u16 = 0x1000;

// Field specific
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;
pub const ACC_ENUM: u16 = 0x4000;

// Method specific
pub const ACC_SYNCHRONIZED: u16 = 0x0020;
pub const ACC_BRIDGE: u16 = 0x0040;
pub const ACC_VARARGS: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_STRICT: u16 = 0x0800;

// Class specific
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ANNOTATION: u16 = 0x2000;
pub const ACC_MODULE: u16 = 0x8000;

pub trait Accessible {
    fn flags(&self) -> u16;

    fn is_public(&self) -> bool {
        self.flags() & ACC_PUBLIC != 0
    }

    fn is_private(&self) -> bool {
        self.flags() & ACC_PRIVATE != 0
    }

    fn is_protected(&self) -> bool {
        self.flags() & ACC_PROTECTED != 0
    }

    fn is_static(&self) -> bool {
        self.flags() & ACC_STATIC != 0
    }

    fn is_final(&self) -> bool {
        self.flags() & ACC_FINAL != 0
    }

    fn is_abstract(&self) -> bool {
        self.flags() & ACC_ABSTRACT != 0
    }

    fn is_synthetic(&self) -> bool {
        self.flags() & ACC_SYNTHETIC != 0
    }
}
