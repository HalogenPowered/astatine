use bytes::{Buf, Bytes};
use crate::attributes::attributes::Attribute;
use crate::attributes::exceptions::ExceptionTableEntry;

pub struct CodeAttribute {
    pub attribute_name: str,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes: Vec<dyn Attribute>
}

impl Attribute for CodeAttribute {
    fn name(&self) -> &str {
        &self.attribute_name
    }

    fn read_from_internal(attribute_name_index: u16, attribute_length: u32, mut buf: &Bytes) -> Self {
        let max_stack = buf.get_u16();
        let max_locals = buf.get_u16();
        let code_length = buf.get_u32();
        let code = buf.copy_to_bytes(code_length as usize).to_vec();
        let exception_table_length = buf.get_u16();
        let mut exception_table = Vec::with_capacity(exception_table_length as usize);
        for _ in 0..exception_table_length {
            exception_table.push(ExceptionTableEntry::read_from(buf));
        }
        let attributes_count =
    }
}
