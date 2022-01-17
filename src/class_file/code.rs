use std::collections::HashMap;
use bytes::{Buf, Bytes};
use java_desc::FieldType;
use crate::class_file::stack_map_table::{StackMapFrame, StackMapTable};
use crate::types::constant_pool::ConstantPool;
use crate::types::utils::Nameable;

#[derive(Debug)]
pub struct CodeBlock {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    exception_handlers: ExceptionHandlerTable,
    line_numbers: Option<HashMap<u16, u16>>,
    local_variables: Option<LocalVariableTable>,
    local_variable_types: Option<LocalVariableTable>,
    stack_map_table: Option<StackMapTable>
}

impl CodeBlock {
    pub const fn new(
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_handlers: ExceptionHandlerTable,
        line_numbers: Option<HashMap<u16, u16>>,
        local_variables: Option<LocalVariableTable>,
        local_variable_types: Option<LocalVariableTable>,
        stack_map_table: Option<StackMapTable>
    ) -> Self {
        CodeBlock { max_stack, max_locals, code, exception_handlers, line_numbers, local_variables, local_variable_types, stack_map_table }
    }

    pub fn get_code(&self, index: usize) -> Option<&u8> {
        self.code.get(index)
    }

    pub fn get_exception_handler(&self, index: usize) -> Option<&ExceptionHandlerBlock> {
        self.exception_handlers.get(index)
    }

    pub fn get_line_number(&self, code_index: u16) -> Option<&u16> {
        self.line_numbers.as_ref().and_then(|map| map.get(&code_index))
    }

    pub fn get_local_variable(&self, index: usize) -> Option<&LocalVariable> {
        self.local_variables.as_ref().and_then(|table| table.get(index))
    }

    pub fn get_local_variable_type(&self, index: usize) -> Option<&LocalVariable> {
        self.local_variable_types.as_ref().and_then(|table| table.get(index))
    }

    pub fn get_stack_map_frame(&self, index: usize) -> Option<&StackMapFrame> {
        self.stack_map_table.as_ref().and_then(|table| table.get(index))
    }
}

pub type ExceptionHandlerTable = Vec<ExceptionHandlerBlock>;

#[derive(Debug)]
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

pub type LocalVariableTable = Vec<LocalVariable>;

#[derive(Debug)]
pub struct LocalVariable {
    name: String,
    pub descriptor: FieldType,
    pub start_pc: u16,
    pub length: u16,
    pub index: u16
}

impl LocalVariable {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let start_pc = buf.get_u16();
        let length = buf.get_u16();

        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid local variable in table for method in class file {}! Expected \
                name index {} to be in constant pool!", class_file_name, name_index))
            .clone();

        let descriptor_index = buf.get_u16();
        let descriptor = pool.get_utf8(descriptor_index as usize)
            .and_then(|value| FieldType::parse(value))
            .expect(&format!("Invalid local variable in table for method in class file {}! Could \
                not parse field descriptor!", class_file_name));

        let index = buf.get_u16();
        LocalVariable { name, descriptor, start_pc, length, index }
    }
}

impl Nameable for LocalVariable {
    fn name(&self) -> &str {
        &self.name
    }
}
