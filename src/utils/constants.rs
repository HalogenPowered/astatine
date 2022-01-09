pub const JAVA_CLASS_FILE_MAGIC: u32 = 0xCAFEBABE;

// All of the version constants that we support
pub const JAVA_VERSION_1_1: u16 = 45;
pub const JAVA_VERSION_1_2: u16 = 46;
pub const JAVA_VERSION_1_3: u16 = 47;
pub const JAVA_VERSION_1_4: u16 = 48;
pub const JAVA_VERSION_1_5: u16 = 49;
pub const JAVA_VERSION_6: u16 = 50;
pub const JAVA_VERSION_7: u16 = 51;
pub const JAVA_VERSION_8: u16 = 52;
pub const JAVA_VERSION_9: u16 = 53;
pub const JAVA_VERSION_10: u16 = 54;
pub const JAVA_VERSION_11: u16 = 55;
pub const JAVA_VERSION_12: u16 = 56;
pub const JAVA_VERSION_13: u16 = 57;
pub const JAVA_VERSION_14: u16 = 58;
pub const JAVA_VERSION_15: u16 = 59;
pub const JAVA_VERSION_16: u16 = 60;
pub const JAVA_VERSION_17: u16 = 61;

// Some other version constants
pub const JAVA_MINIMUM_SUPPORTED_VERSION: u16 = JAVA_VERSION_1_1;
pub const JAVA_MAXIMUM_SUPPORTED_VERSION: u16 = JAVA_VERSION_17;
pub const JAVA_PREVIEW_MINOR_VERSION: u16 = 65535;

pub const CLASS_INITIALIZER_METHOD_NAME: &str = "<clinit>";
pub const OBJECT_INITIALIZER_METHOD_NAME: &str = "<init>";

pub fn is_valid_version(major_version: u16, minor_version: u16) -> bool {
    if major_version < JAVA_MINIMUM_SUPPORTED_VERSION || major_version > JAVA_MAXIMUM_SUPPORTED_VERSION {
        return false;
    }
    if major_version >= JAVA_VERSION_12 && (minor_version != 0 && minor_version != JAVA_PREVIEW_MINOR_VERSION) {
        return false;
    }
    return true;
}
