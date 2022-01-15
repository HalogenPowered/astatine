pub type ReturnAddress = u32;

pub const T_BOOLEAN: u8 = 4;
pub const T_CHAR: u8 = 5;
pub const T_FLOAT: u8 = 6;
pub const T_DOUBLE: u8 = 7;
pub const T_BYTE: u8 = 8;
pub const T_SHORT: u8 = 9;
pub const T_INT: u8 = 10;
pub const T_LONG: u8 = 11;

#[repr(u8)]
pub enum ArrayType {
    Boolean = T_BOOLEAN,
    Char = T_CHAR,
    Float = T_FLOAT,
    Double = T_DOUBLE,
    Byte = T_BYTE,
    Short = T_SHORT,
    Int = T_INT,
    Long = T_LONG
}
