use bytes::{Buf, Bytes};
use crate::attributes::attributes::Attribute;

pub type ExceptionTable = Vec<ExceptionTableEntry>;

pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16
}
