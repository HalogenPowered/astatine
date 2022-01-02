use std::ops::Range;
use bytes::{Buf, Bytes};

pub struct StackMapFrame {
    pub frame_type: u8,
    pub data: Option<dyn ExtraFrameData>
}

impl StackMapFrame {
    pub fn read_from(mut buf: &Bytes) -> Self {
        let frame_type = buf.get_u8();
        let data: Option<dyn ExtraFrameData> = match frame_type {
            0..=63 => Option::None,
            64..=127 => Option::Some(SameLocalsOneStackFrameData::read_from(frame_type, buf)),
            247 => Option::Some(SameLocalsOneStackExtendedFrameData::read_from(frame_type, buf)),
            248..=251 => Option::Some(OffsetDeltaFrameData::read_from(frame_type, buf)),
            252..=254 => Option::Some(AppendFrameData::read_from(frame_type, buf)),
            255 => Option::Some(FullFrameData::read_from(frame_type, buf)),
            _ => Option::None
        };
        StackMapFrame { frame_type, data }
    }
}

pub trait ExtraFrameData {
    fn read_from(frame_type: u8, mut buf: &Bytes) -> Self;
}

pub struct SameLocalsOneStackFrameData {
    pub stack: VerificationTypeInfo
}

impl ExtraFrameData for SameLocalsOneStackFrameData {
    fn read_from(_: u8, mut buf: &Bytes) -> Self {
        let stack = VerificationTypeInfo::read_from(buf);
        SameLocalsOneStackFrameData { stack }
    }
}

pub struct SameLocalsOneStackExtendedFrameData {
    pub offset_delta: u16,
    pub stack: VerificationTypeInfo
}

impl ExtraFrameData for SameLocalsOneStackExtendedFrameData {
    fn read_from(_: u8, mut buf: &Bytes) -> Self {
        let offset_delta = buf.get_u16();
        let stack = VerificationTypeInfo::read_from(buf);
        SameLocalsOneStackExtendedFrameData { offset_delta, stack }
    }
}

pub struct OffsetDeltaFrameData {
    pub offset_delta: u16
}

impl ExtraFrameData for OffsetDeltaFrameData {
    fn read_from(_: u8, mut buf: &Bytes) -> Self {
        let offset_data = buf.get_u16();
        OffsetDeltaFrameData { offset_delta }
    }
}

pub struct AppendFrameData {
    pub offset_delta: u16,
    pub locals: Vec<VerificationTypeInfo>
}

impl ExtraFrameData for AppendFrameData {
    fn read_from(frame_type: u8, mut buf: &Bytes) -> Self {
        let offset_delta = buf.get_u16();
        let locals_count = frame_type - 251;
        let mut locals = Vec::with_capacity(locals_count as usize);
        for _ in 0..locals_count {
            locals.push(VerificationTypeInfo::read_from(buf))
        }
        AppendFrameData { offset_delta, locals }
    }
}

pub struct FullFrameData {
    pub offset_delta: u16,
    pub locals: Vec<VerificationTypeInfo>,
    pub stack: Vec<VerificationTypeInfo>
}

impl ExtraFrameData for FullFrameData {
    fn read_from(frame_type: u8, mut buf: &Bytes) -> Self {
        let offset_delta = buf.get_u16();
        let number_of_locals = buf.get_u16();
        let mut locals = Vec::with_capacity(number_of_locals as usize);
        for _ in 0..number_of_locals {
            locals.push(VerificationTypeInfo::read_from(buf));
        }
        let number_of_stack_items = buf.get_u16();
        let mut stack = Vec::with_capacity(number_of_stack_items as usize);
        for _ in 0..number_of_stack_items {
            stack.push(VerificationTypeInfo::read_from(buf));
        }
        FullFrameData { offset_delta, locals, stack }
    }
}

const SAME_FRAME_TYPE: Range<u8> = 0..64;
const SAME_LOCALS_ONE_STACK_FRAME_TYPE: Range<u8> = 64..128;
const SAME_LOCALS_ONE_STACK_FRAME_EXTENDED_TYPE: u8 = 247;
const CHOP_FRAME_TYPE: Range<u8> = 248..251;
const SAME_FRAME_EXTENDED_TYPE: u8 = 251;
const APPEND_FRAME_TYPE: Range<u8> = 252..255;
const FULL_FRAME_TYPE: u8 = 255;

pub struct VerificationTypeInfo {
    pub tag: u8,
    pub extra: Option<u16>
}

impl VerificationTypeInfo {
    fn read_from(mut buf: &Bytes) -> Self {
        let tag = buf.get_u8();
        let extra: Option<u16> = match tag {
            OBJECT_VARIABLE_INFO | UNINITIALIZED_VARIABLE_INFO => Option::Some(buf.get_u16()),
            _ => Option::None
        };
        VerificationTypeInfo { tag, extra }
    }
}

pub trait ExtraVariableData {
    fn read_from(mut buf: &Bytes) -> Self;
}

pub struct ObjectVariableData {
    pub cpool_index: u16
}

impl ExtraVariableData for ObjectVariableData {
    fn read_from(mut buf: &Bytes) -> Self {
        let cpool_index = buf.get_u16();
        ObjectVariableData { cpool_index }
    }
}

pub struct UninitializedVariableData {
    pub offset: u16
}

impl ExtraVariableData for UninitializedVariableData {
    fn read_from(mut buf: &Bytes) -> Self {
        let offset = buf.get_u16();
        UninitializedVariableData { offset }
    }
}

const TOP_VARIABLE_INFO_TYPE: u8 = 0;
const INTEGER_VARIABLE_INFO_TYPE: u8 = 1;
const FLOAT_VARIABLE_INFO_TYPE: u8 = 2;
const DOUBLE_VARIABLE_INFO_TYPE: u8 = 3;
const LONG_VARIABLE_INFO: u8 = 4;
const NULL_VARIABLE_INFO_TYPE: u8 = 5;
const UNINITIALIZED_THIS_VARIABLE_INFO_TYPE: u8 = 6;
const OBJECT_VARIABLE_INFO: u8 = 7;
const UNINITIALIZED_VARIABLE_INFO: u8 = 8;
