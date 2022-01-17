use crate::objects::heap::HeapSpace;
use crate::objects::object::{InstanceObject, ReferenceArrayObject, TypeArrayObject};
use crate::objects::reference::Reference;
use crate::utils::vm_types::ReturnAddress;

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

    pub fn get_local_ref<'a, 'b>(&'a self, index: usize, heap: &'b HeapSpace<'b>) -> Reference<&'b InstanceObject<'b>> {
        self.get_ref(self.get_local(index), |index| heap.get_ref(index))
    }

    pub fn get_local_ref_array<'a, 'b>(&'a self, index: usize, heap: &'b HeapSpace<'b>) -> Reference<&'b ReferenceArrayObject<'b, 'b>> {
        self.get_ref(self.get_local(index), |index| heap.get_ref_array(index))
    }

    pub fn get_local_type_array<'a, 'b>(&'a self, index: usize, heap: &'b HeapSpace<'b>) -> Reference<&'b TypeArrayObject<'b>> {
        self.get_ref(self.get_local(index), |index| heap.get_type_array(index))
    }

    pub fn get_local_return_address(&self, index: usize) -> Option<ReturnAddress> {
        self.local_variables.get(index).map(|value| *value)
    }

    fn get_local(&self, index: usize) -> u32 {
        *self.local_variables.get(index).expect(&format!("Invalid local variable index {}!", index))
    }

    pub fn push_bool_op(&mut self, value: bool) {
        self.push_op(value as u32);
    }

    pub fn push_byte_op(&mut self, value: i8) {
        self.push_op(value as u32);
    }

    pub fn push_char_op(&mut self, value: char) {
        self.push_op(value as u32);
    }

    pub fn push_short_op(&mut self, value: i16) {
        self.push_op(value as u32);
    }

    pub fn push_int_op(&mut self, value: i32) {
        self.push_op(value as u32);
    }

    pub fn push_float_op(&mut self, value: f32) {
        self.push_op(value as u32);
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

    pub fn pop_ref_op<'a, 'b>(&'a mut self, heap: &'b HeapSpace<'b>) -> Reference<&'b InstanceObject<'b>> {
        self.get_ref(self.pop_op(), |index| heap.get_ref(index))
    }

    pub fn pop_ref_array_op<'a, 'b>(&'a mut self, heap: &'b HeapSpace<'b>) -> Reference<&'b ReferenceArrayObject<'b, 'b>> {
        self.get_ref(self.pop_op(), |index| heap.get_ref_array(index))
    }

    pub fn pop_type_array_op<'a, 'b>(&'a mut self, heap: &'b HeapSpace<'b>) -> Reference<&'b TypeArrayObject<'b>> {
        self.get_ref(self.pop_op(), |index| heap.get_type_array(index))
    }

    fn pop_op(&mut self) -> u32 {
        self.operand_stack.pop().expect("Nothing left to pop on the stack! If verification \
            succeeded, this should be impossible!")
    }

    fn get_ref<T, F>(&self, offset: u32, f: F) -> Reference<T> where F : Fn(usize) -> Option<T> {
        let ref_index = (offset + 1) as usize;
        match offset {
            0 => Reference::Null,
            _ => Reference::Value(f(ref_index).expect("Invalid reference in stack frame!"))
        }
    }
}

fn parts_to_long(most: u32, least: u32) -> i64 {
    (((most as u64) << 32) | (least as u64)) as i64
}

fn parts_to_double(most: u32, least: u32) -> f64 {
    f64::from_bits(((most as u64) << 32) | (least as u64))
}
