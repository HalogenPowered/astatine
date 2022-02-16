/*
 * Copyright (C) 2022 Callum Seabrook <callum.seabrook@prevarinite.com>
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; version 2.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program; if not, write to the Free Software Foundation, Inc., 51 Franklin
 * Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */

use bytes::{Buf, Bytes};
use internship::IStr;
use std::collections::HashMap;
use std::sync::Arc;
use astatine_macros::{FieldDescribable, Nameable};
use crate::code::StackFrame;
use crate::constants::*;
use crate::types::{Class, ConstantPool};
use crate::utils::{BufferExtras, IdentEq};
use crate::utils::descriptors::FieldDescriptor;
use super::ClassLoader;
use super::verification::*;

#[derive(Debug)]
pub struct CodeBlock {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_handlers: ExceptionHandlerTable,
    line_numbers: Option<HashMap<u16, u16>>,
    local_variables: Option<LocalVariableTable>,
    local_variable_types: Option<LocalVariableTable>,
    stack_map_table: Option<StackMapTable>
}

const MAX_CODE_BYTES: usize = 65535;

impl CodeBlock {
    pub(crate) fn parse(loader: Arc<ClassLoader>, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let max_stack = buf.get_u16();
        let max_locals = buf.get_u16();
        let code_length = buf.get_u32() as usize;
        assert!(code_length > 0 && code_length <= MAX_CODE_BYTES, "Invalid code attribute! Code \
            length must be > 0 and < {}!", MAX_CODE_BYTES);
        let code = buf.get_u8_array(code_length);
        let exception_handlers = ExceptionHandlerTable::parse(loader, pool, buf);
        let attributes = parse_attributes(pool, buf);
        CodeBlock {
            max_stack,
            max_locals,
            code,
            exception_handlers,
            line_numbers: attributes.0,
            local_variables: attributes.1,
            local_variable_types: attributes.2,
            stack_map_table: attributes.3
        }
    }

    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }

    pub fn code(&self) -> &[u8] {
        self.code.as_slice()
    }

    pub fn exception_handlers(&self) -> &ExceptionHandlerTable {
        &self.exception_handlers
    }

    pub fn local_variables(&self) -> Option<&LocalVariableTable> {
        self.local_variables.as_ref()
    }

    pub fn local_variable_types(&self) -> Option<&LocalVariableTable> {
        self.local_variable_types.as_ref()
    }

    pub fn stack_map_table(&self) -> Option<&StackMapTable> {
        self.stack_map_table.as_ref()
    }

    pub fn new_stack_frame(&self) -> StackFrame {
        StackFrame::new(self.max_stack, self.max_locals)
    }

    pub fn new_code_reader(&self) -> Bytes {
        Bytes::copy_from_slice(self.code.as_slice())
    }
}

#[derive(Debug)]
pub struct ExceptionHandlerTable {
    handlers: Vec<ExceptionHandlerBlock>
}

impl ExceptionHandlerTable {
    pub(crate) fn parse(loader: Arc<ClassLoader>, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let handler_count = buf.get_u16();
        let mut handlers = Vec::with_capacity(handler_count as usize);
        for _ in 0..handler_count {
            handlers.push(ExceptionHandlerBlock::parse(Arc::clone(&loader), pool, buf))
        }
        ExceptionHandlerTable { handlers }
    }

    pub fn get(&self, index: usize) -> Option<&ExceptionHandlerBlock> {
        self.handlers.get(index)
    }

    pub fn get_handler(&self, exception: &Class) -> Option<&ExceptionHandlerBlock> {
        self.handlers.iter().find(|element| element.catch_type().ident_eq(exception))
    }
}

#[derive(Debug)]
pub struct ExceptionHandlerBlock {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: Arc<Class>
}

impl ExceptionHandlerBlock {
    pub(crate) fn parse(loader: Arc<ClassLoader>, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let start_pc = buf.get_u16();
        let end_pc = buf.get_u16();
        let handler_pc = buf.get_u16();
        let catch_type = pool.get_class_no_holder(buf.get_u16() as usize, loader).unwrap();
        ExceptionHandlerBlock { start_pc, end_pc, handler_pc, catch_type }
    }

    pub fn start_pc(&self) -> u16 {
        self.start_pc
    }

    pub fn end_pc(&self) -> u16 {
        self.end_pc
    }

    pub fn handler_pc(&self) -> u16 {
        self.handler_pc
    }

    pub fn catch_type(&self) -> &Class {
        &self.catch_type
    }
}

#[derive(Debug)]
pub struct LocalVariableTable {
    variables: Vec<LocalVariable>
}

impl LocalVariableTable {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let variables = buf.get_generic_u16_array(|buf| LocalVariable::parse(pool, buf));
        LocalVariableTable { variables }
    }

    pub fn get(&self, index: usize) -> Option<&LocalVariable> {
        self.variables.get(index)
    }
}

#[derive(Debug, Nameable, FieldDescribable)]
pub struct LocalVariable {
    name: IStr,
    descriptor: FieldDescriptor,
    start_pc: u16,
    length: u16,
    index: u16
}

impl LocalVariable {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let start_pc = buf.get_u16();
        let length = buf.get_u16();
        let name = pool.get_utf8(buf.get_u16() as usize)
            .expect(&format!("Invalid local variable in table for method! Expected name index to be \
                in constant pool!"));
        let descriptor = pool.get_utf8(buf.get_u16() as usize)
            .and_then(|value| FieldDescriptor::parse(value.as_str()))
            .unwrap();
        let index = buf.get_u16();
        LocalVariable { name, descriptor, start_pc, length, index }
    }

    pub fn start_pc(&self) -> u16 {
        self.start_pc
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

type CodeAttributes = (Option<HashMap<u16, u16>>, Option<LocalVariableTable>,
                       Option<LocalVariableTable>, Option<StackMapTable>);

fn parse_attributes(pool: &ConstantPool, buf: &mut Bytes) -> CodeAttributes {
    let mut line_number_table = None;
    let mut local_variable_table = None;
    let mut local_variable_type_table = None;
    let mut stack_map_table = None;

    let mut attribute_count = buf.get_u16();
    while attribute_count > 0 {
        assert!(buf.len() > 6, "Truncated code attributes!");
        let attribute_name = pool.get_utf8(buf.get_u16() as usize).unwrap();
        let attribute_length = buf.get_u32();

        if attribute_name == JVM_ATTRIBUTE_LINE_NUMBER_TABLE {
            let table_length = buf.get_u16();
            let mut table = HashMap::with_capacity(table_length as usize);
            for _ in 0..table_length {
                let start_pc = buf.get_u16();
                let line_number = buf.get_u16();
                table.insert(start_pc, line_number);
            }
            line_number_table = Some(table)
        } else if attribute_name == JVM_ATTRIBUTE_LOCAL_VARIABLE_TABLE {
            local_variable_table = Some(LocalVariableTable::parse(pool, buf));
        } else if attribute_name == JVM_ATTRIBUTE_LOCAL_VARIABLE_TYPE_TABLE {
            local_variable_type_table = Some(LocalVariableTable::parse(pool, buf));
        } else if attribute_name == JVM_ATTRIBUTE_STACK_MAP_TABLE {
            stack_map_table = Some(StackMapTable::parse(buf));
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (line_number_table, local_variable_table, local_variable_type_table, stack_map_table)
}
