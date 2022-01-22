#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct ClassFileVersion {
    major: u16,
    minor: u16
}

const PREVIEW_MINOR: u16 = 65535;

impl ClassFileVersion {
    pub const RELEASE_1_1: ClassFileVersion = ClassFileVersion::new(45, 0);
    pub const RELEASE_1_2: ClassFileVersion = ClassFileVersion::new(46, 0);
    pub const RELEASE_1_3: ClassFileVersion = ClassFileVersion::new(47, 0);
    pub const RELEASE_1_4: ClassFileVersion = ClassFileVersion::new(48, 0);
    pub const RELEASE_1_5: ClassFileVersion = ClassFileVersion::new(49, 0);
    pub const RELEASE_6: ClassFileVersion = ClassFileVersion::new(50, 0);
    pub const RELEASE_7: ClassFileVersion = ClassFileVersion::new(51, 0);
    pub const RELEASE_8: ClassFileVersion = ClassFileVersion::new(52, 0);
    pub const RELEASE_9: ClassFileVersion = ClassFileVersion::new(53, 0);
    pub const RELEASE_10: ClassFileVersion = ClassFileVersion::new(54, 0);
    pub const RELEASE_11: ClassFileVersion = ClassFileVersion::new(55, 0);
    pub const RELEASE_12: ClassFileVersion = ClassFileVersion::new(56, 0);
    pub const RELEASE_13: ClassFileVersion = ClassFileVersion::new(57, 0);
    pub const RELEASE_14: ClassFileVersion = ClassFileVersion::new(58, 0);
    pub const RELEASE_15: ClassFileVersion = ClassFileVersion::new(59, 0);
    pub const RELEASE_16: ClassFileVersion = ClassFileVersion::new(60, 0);
    pub const RELEASE_17: ClassFileVersion = ClassFileVersion::new(61, 0);

    pub const MINIMUM: ClassFileVersion = ClassFileVersion::RELEASE_1_1;
    pub const LATEST: ClassFileVersion = ClassFileVersion::RELEASE_17;

    pub fn from(major: u16, minor: u16) -> ClassFileVersion {
        if major >= 46 && major <= 55 && minor > 0 {
            panic!("Invalid class file version! Majors between 46 (Java 1.2) and 55 (Java 11) \
                did not allow major versions!");
        }
        if major >= 56 && minor != 0 && minor != PREVIEW_MINOR {
            panic!("Invalid class file version! Majors 56 (Java 12) and above only support a \
                minor of 0 for release and {} for preview! The given minor was {}!", PREVIEW_MINOR, minor);
        }
        if major >= 56 && minor != 0 {
            panic!("Astatine does not support preview Java versions! Given major was {}", major);
        }
        match major {
            45 => {
                if major != 0 {
                    ClassFileVersion::new(45, minor)
                } else {
                    ClassFileVersion::RELEASE_1_1
                }
            },
            46 => ClassFileVersion::RELEASE_1_2,
            47 => ClassFileVersion::RELEASE_1_3,
            48 => ClassFileVersion::RELEASE_1_4,
            49 => ClassFileVersion::RELEASE_1_5,
            50 => ClassFileVersion::RELEASE_6,
            51 => ClassFileVersion::RELEASE_7,
            52 => ClassFileVersion::RELEASE_8,
            53 => ClassFileVersion::RELEASE_9,
            54 => ClassFileVersion::RELEASE_10,
            55 => ClassFileVersion::RELEASE_11,
            56 => ClassFileVersion::RELEASE_12,
            57 => ClassFileVersion::RELEASE_13,
            58 => ClassFileVersion::RELEASE_14,
            59 => ClassFileVersion::RELEASE_15,
            60 => ClassFileVersion::RELEASE_16,
            61 => ClassFileVersion::RELEASE_17,
            _ => panic!("Unsupported major {}!", major)
        }
    }

    pub const fn new(major: u16, minor: u16) -> ClassFileVersion {
        ClassFileVersion { major, minor }
    }

    pub fn major(&self) -> u16 {
        self.major
    }

    pub fn minor(&self) -> u16 {
        self.minor
    }
}
