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

use paste::paste;
use crate::objects::*;

macro_rules! get_pop_ref {
    ($name:ident, $ty:ty) => {
        paste! {
            pub fn [<get_local_ $name>](&self, index: usize, heap: &HeapSpace) -> Reference<$ty> {
                StackFrame::get_ref(self.get_local(index), |index| heap.[<get_ $name>](index))
            }

            pub fn [<pop_ $name _op>](&mut self, heap: &HeapSpace) -> Reference<$ty> {
                StackFrame::get_ref(self.pop_op(), |index| heap.[<get_ $name>](index))
            }
        }
    }
}

macro_rules! set_local_push_op {
    ($name:ident, $ty:ty) => {
        paste! {
            pub fn [<push_ $name _op>](&mut self, value: $ty) {
                self.push_op(value as u32)
            }

            pub fn [<set_local_ $name>](&mut self, index: usize, value: $ty) {
                self.set_local(index, value as u32);
            }
        }
    }
}

pub struct StackFrame {
    local_variables: Vec<u32>,
    operand_stack: Vec<u32>
}

impl StackFrame {
    pub fn new(max_stack: u16, max_locals: u16) -> StackFrame {
        let local_variables = Vec::with_capacity(max_locals as usize);
        let operand_stack = Vec::with_capacity(max_stack as usize);
        StackFrame { local_variables, operand_stack }
    }

    pub fn get_local_bool(&self, index: usize) -> bool {
        self.get_local(index) != 0
    }

    pub fn get_local_byte(&self, index: usize) -> i8 {
        (self.get_local(index) & 255) as i8
    }

    pub fn get_local_char(&self, index: usize) -> char {
        char::from_u32(self.get_local(index))
            .expect(&format!("Invalid character at local variable index {}!", index))
    }

    pub fn get_local_short(&self, index: usize) -> i16 {
        (self.get_local(index) & 65535) as i16
    }

    pub fn get_local_int(&self, index: usize) -> i32 {
        self.get_local(index) as i32
    }

    pub fn get_local_float(&self, index: usize) -> f32 {
        f32::from_bits(self.get_local(index))
    }

    pub fn get_local_long(&self, index: usize) -> i64 {
        parts_to_long(self.get_local(index), self.get_local(index + 1))
    }

    pub fn get_local_double(&self, index: usize) -> f64 {
        parts_to_double(self.get_local(index), self.get_local(index + 1))
    }

    get_pop_ref!(ref, InstanceObject);
    get_pop_ref!(ref_array, ReferenceArrayObject);
    get_pop_ref!(type_array, TypeArrayObject);

    pub fn get_local_return_address(&self, index: usize) -> Option<u32> {
        self.local_variables.get(index).map(|value| *value)
    }

    fn get_local(&self, index: usize) -> u32 {
        *self.local_variables.get(index).expect(&format!("Invalid local variable index {}!", index))
    }

    set_local_push_op!(bool, bool);
    set_local_push_op!(byte, i8);
    set_local_push_op!(char, char);
    set_local_push_op!(short, i16);
    set_local_push_op!(int, i32);
    set_local_push_op!(float, f32);

    pub fn set_local_long(&mut self, index: usize, value: i64) {
        self.set_local(index, (value >> 32) as u32);
        self.set_local(index, value as u32);
    }

    pub fn set_local_double(&mut self, index: usize, value: f64) {
        let bits = value.to_bits();
        self.set_local(index, (bits >> 32) as u32);
        self.set_local(index, bits as u32);
    }

    pub fn set_local_ref(&mut self, index: usize, value: u32) {
        self.set_local(index, value);
    }

    fn set_local(&mut self, index: usize, value: u32) {
        self.local_variables.insert(index, value);
    }

    pub fn push_long_op(&mut self, value: i64) {
        self.push_op((value >> 32) as u32);
        self.push_op(value as u32);
    }

    pub fn push_double_op(&mut self, value: f64) {
        let bits = value.to_bits();
        self.push_op((bits >> 32) as u32);
        self.push_op(bits as u32);
    }

    pub fn push_ref_op(&mut self, offset: u32) {
        self.push_op(offset);
    }

    pub fn push_null_op(&mut self) {
        self.push_op(0);
    }

    pub fn push_op(&mut self, value: u32) {
        self.operand_stack.push(value);
    }

    pub fn set_op(&mut self, offset: usize, value: u32) {
        self.operand_stack.insert(self.operand_stack.len() - 1 - offset, value)
    }

    pub fn pop_bool_op(&mut self) -> bool {
        self.pop_op() != 0
    }

    pub fn pop_byte_op(&mut self) -> i8 {
        self.pop_op() as i8
    }

    pub fn pop_char_op(&mut self) -> char {
        char::from_u32(self.pop_op()).expect("Invalid character found in operand stack!")
    }

    pub fn pop_short_op(&mut self) -> i16 {
        self.pop_op() as i16
    }

    pub fn pop_int_op(&mut self) -> i32 {
        self.pop_op() as i32
    }

    pub fn pop_float_op(&mut self) -> f32 {
        f32::from_bits(self.pop_op())
    }

    pub fn pop_long_op(&mut self) -> i64 {
        parts_to_long(self.pop_op(), self.pop_op())
    }

    pub fn pop_double_op(&mut self) -> f64 {
        parts_to_double(self.pop_op(), self.pop_op())
    }

    pub fn pop_op(&mut self) -> u32 {
        self.operand_stack.pop().expect("Nothing left to pop on the stack! If verification \
            succeeded, this should be impossible!")
    }

    pub fn peek_op(&self) -> u32 {
        *self.operand_stack.last().expect("Nothing left to pop on the stack! If verification \
            succeeded, this should be impossible!")
    }

    pub fn get_op(&self, offset: usize) -> u32 {
        if offset == 0 {
            return self.peek_op();
        }
        *self.operand_stack.get(self.operand_stack.len() - 1 - offset)
            .expect(&format!("Could not pop element at offset {} from end of stack!", offset))
    }

    #[inline]
    fn get_ref<T>(offset: u32, mapper: impl Fn(usize) -> Reference<T>) -> Reference<T> {
        let ref_index = (offset + 1) as usize;
        match offset {
            0 => Reference::Null,
            _ => mapper(ref_index)
        }
    }
}

fn parts_to_long(most: u32, least: u32) -> i64 {
    (((most as u64) << 32) | (least as u64)) as i64
}

fn parts_to_double(most: u32, least: u32) -> f64 {
    f64::from_bits(((most as u64) << 32) | (least as u64))
}
