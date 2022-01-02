use std::ops::Range;
use bytes::{Buf, Bytes};

pub struct StackMapFrame {
    pub frame_type: u8,
    pub offset: u16,
    pub stack: Vec<VerificationTypeInfo>,
    pub locals: Vec<VerificationTypeInfo>
}

#[repr(u8)]
pub enum VerificationTypeInfo {
    Top = 0,
    Integer = 1,
    Float = 2,
    Double = 3,
    Long = 4,
    Null = 5,
    UninitializedThis = 6,
    Object { constant_pool_index: u16 } = 7,
    Uninitialized { offset: u16 } = 8,
}
