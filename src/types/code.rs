use bytes::{Buf, Bytes};

pub struct CodeBlock {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_handlers: ExceptionHandlerTable,
    pub line_number_table: Option<LineNumberTable>
}

pub type ExceptionHandlerTable = Vec<ExceptionHandlerBlock>;

pub struct ExceptionHandlerBlock {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16
}

impl ExceptionHandlerBlock {
    pub fn parse(buf: &mut Bytes) -> Self {
        let start_pc = buf.get_u16();
        let end_pc = buf.get_u16();
        let handler_pc = buf.get_u16();
        let catch_type = buf.get_u16();
        ExceptionHandlerBlock { start_pc, end_pc, handler_pc, catch_type }
    }
}

pub type LineNumberTable = Vec<LineNumberEntry>;

pub struct LineNumberEntry {
    pub start_pc: u16,
    pub line_number: u16
}

impl LineNumberEntry {
    pub fn parse(buf: &mut Bytes) -> Self {
        let start_pc = buf.get_u16();
        let line_number = buf.get_u16();
        LineNumberEntry { start_pc, line_number }
    }
}
