use crate::objects::heap::HeapSpace;
use crate::objects::object::*;
use crate::objects::reference::Reference;
use crate::utils::vm_types::ReturnAddress;

macro_rules! get_ref {
    ($name:ident, $ty:ty, $getter_name: ident) => {
        pub fn $name<'a, 'b>(&'a self, index: usize, heap: &'b Box<HeapSpace>) -> Reference<&'b Box<$ty>> {
            StackFrame::get_ref(self.get_local(index), |index| heap.$getter_name(index))
        }
    }
}

macro_rules! pop_ref_op {
    ($name:ident, $ty:ty, $getter_name:ident) => {
        pub fn $name<'a, 'b>(&'a mut self, heap: &'b Box<HeapSpace>) -> Reference<&'b Box<$ty>> {
            StackFrame::get_ref(self.pop_op(), |index| heap.$getter_name(index))
        }
    }
}

macro_rules! push_op {
    ($name:ident, $ty:ty) => {
        pub fn $name(&mut self, value: $ty) {
            self.push_op(value as u32)
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

    get_ref!(get_local_ref, InstanceObject, get_ref);
    get_ref!(get_local_ref_array, ReferenceArrayObject, get_ref_array);
    get_ref!(get_local_type_array, TypeArrayObject, get_type_array);

    pub fn get_local_return_address(&self, index: usize) -> Option<ReturnAddress> {
        self.local_variables.get(index).map(|value| *value)
    }

    fn get_local(&self, index: usize) -> u32 {
        *self.local_variables.get(index).expect(&format!("Invalid local variable index {}!", index))
    }

    push_op!(push_bool_op, bool);
    push_op!(push_byte_op, i8);
    push_op!(push_char_op, char);
    push_op!(push_short_op, i16);
    push_op!(push_int_op, i32);
    push_op!(push_float_op, f32);

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

    pop_ref_op!(pop_ref_op, InstanceObject, get_ref);
    pop_ref_op!(pop_ref_array_op, ReferenceArrayObject, get_ref_array);
    pop_ref_op!(pop_type_array_op, TypeArrayObject, get_type_array);

    fn pop_op(&mut self) -> u32 {
        self.operand_stack.pop().expect("Nothing left to pop on the stack! If verification \
            succeeded, this should be impossible!")
    }

    fn get_ref<T, F>(offset: u32, f: F) -> Reference<T> where F : Fn(usize) -> Option<T> {
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
