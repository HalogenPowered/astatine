use bytes::{Buf, Bytes};
use crate::attributes::attributes::Attribute;

pub struct ExceptionsAttribute {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub exception_index_table: Vec<u16>
}

impl Attribute for ExceptionsAttribute {
    fn read_from_internal(attribute_name_index: u16, attribute_length: u32, mut buf: &Bytes) -> Self {
        let number_of_exceptions = buf.get_u16();
        let mut exception_index_table = Vec::with_capacity(number_of_exceptions as usize);
        for _ in 0..number_of_exceptions {
            exception_index_table.push(buf.get_u16());
        }
        ExceptionsAttribute { attribute_name_index, attribute_length, exception_index_table }
    }
}

pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16
}

impl ExceptionTableEntry {
    fn read_from(mut buf: &Bytes) -> Self {
        let start_pc = buf.get_u16();
        let end_pc = buf.get_u16();
        let handler_pc = buf.get_u16();
        let catch_type = buf.get_u16();
        ExceptionTableEntry { start_pc, end_pc, handler_pc, catch_type }
    }
}
