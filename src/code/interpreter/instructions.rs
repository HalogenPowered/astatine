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
use std::sync::Arc;
use crate::class_file::code::CodeBlock;
use crate::code::StackFrame;
use crate::objects::*;
use crate::types::Class;
use crate::utils::vm_types::ArrayType;
use super::{CodeParser, MethodResult};
use super::constants::*;

macro_rules! load_store_array_primitive {
    ($name:ident, $instruction_prefix:literal, $expected:literal, $array_type:pat) => {
        paste! {
            pub(super) fn [<load_array_ $name>](heap: &HeapSpace, frame: &mut StackFrame) {
                array_primitive(
                    heap,
                    frame,
                    &format!("{}ALOAD", $instruction_prefix), $expected,
                    |array_type| matches!(array_type, $array_type),
                    |frame, array, index| frame.[<push_ $name _op>](array.[<get_ $name>](index))
                );
            }

            pub(super) fn [<store_array_ $name>](heap: &HeapSpace, frame: &mut StackFrame) {
                array_primitive(
                    heap,
                    frame,
                    &format!("{}ASTORE", $instruction_prefix), $expected,
                    |array_type| matches!(array_type, $array_type),
                    |frame, array, index| array.[<set_ $name>](index, frame.[<pop_ $name _op>]())
                );
            }
        }
    };
}

pub(super) fn load_array_ref(heap: &HeapSpace, frame: &mut StackFrame) {
    let array_ref = frame.pop_ref_array_op(heap)
        .expect("Invalid array reference on operand stack!");
    let index = frame.pop_int_op();
    let value = array_ref.get(index as usize).expect("Invalid array index on operand stack!");
    frame.push_ref_op(value.offset() as u32);
}

pub(super) fn store_array_ref(heap: &HeapSpace, frame: &mut StackFrame) {
    let array_ref = frame.pop_ref_array_op(heap)
        .expect("Invalid array reference on operand stack!");
    let index = frame.pop_int_op();
    let value = frame.pop_ref_op(heap)
        .expect("Invalid array value on operand stack!");
    array_ref.set(index as usize, value);
}

pub(super) fn load_ref(heap: &HeapSpace, frame: &mut StackFrame, index: u8) {
    let reference = frame.get_local_ref(index as usize, heap)
        .expect(&format!("Invalid reference index {}!", index));
    frame.push_ref_op(reference.offset() as u32);
}

pub(super) fn array_length(heap: &HeapSpace, frame: &mut StackFrame) {
    let array_ref = frame.pop_ref_array_op(heap)
        .expect("Invalid array reference on operand stack!");
    frame.push_int_op(array_ref.len() as i32);
}

pub(super) fn store_ref(heap: &HeapSpace, frame: &mut StackFrame, index: u8) {
    let reference = frame.pop_ref_op(heap)
        .expect("Invalid reference on operand stack! Reference cannot be null!");
    frame.set_local_ref(index as usize, reference.offset() as u32);
}

pub(super) fn throw(
    heap: &HeapSpace,
    code: &CodeBlock,
    frame: &mut StackFrame,
    parser: &mut CodeParser
) -> Option<MethodResult> {
    let exception = frame.pop_ref_op(heap)
        .expect("Invalid exception on operand stack! Reference cannot be null!");
    let handler = code.exception_handlers().get_handler(exception.class());
    match handler {
        Some(value) => {
            parser.seek(value.start_pc() as usize);
            None
        },
        None => Some(MethodResult::Exception)
    }
}

pub(super) fn load_array_byte(heap: &HeapSpace, frame: &mut StackFrame) {
    common_array_primitive(heap, frame, |frame, array, array_type, index| {
        match array_type {
            ArrayType::Byte => frame.push_byte_op(array.get_byte(index)),
            ArrayType::Boolean => frame.push_bool_op(array.get_bool(index)),
            _ => panic!("Invalid type of array for BASTORE! Expected array to be of type \
                byte or boolean, was {}!", array_type)
        }
    })
}

pub(super) fn store_array_byte(heap: &HeapSpace, frame: &mut StackFrame) {
    common_array_primitive(heap, frame, |frame, array, array_type, index| {
        match array_type {
            ArrayType::Byte => array.set_byte(index, frame.pop_byte_op()),
            ArrayType::Boolean => array.set_bool(index, frame.pop_bool_op()),
            _ => panic!("Invalid type of array for BASTORE! Expected array to be of type \
                byte or boolean, was {}!", array_type)
        }
    })
}

load_store_array_primitive!(char, "C", "char", ArrayType::Char);
load_store_array_primitive!(double, "D", "double", ArrayType::Double);
load_store_array_primitive!(float, "F", "float", ArrayType::Float);
load_store_array_primitive!(int, "I", "int", ArrayType::Int);
load_store_array_primitive!(long, "L", "long", ArrayType::Long);
load_store_array_primitive!(short, "S", "short", ArrayType::Short);

pub(super) fn check_cast(
    heap: &HeapSpace,
    class: &Class,
    frame: &mut StackFrame,
    parser: &mut CodeParser
) {
    let reference = frame.pop_ref_op(heap);
    if matches!(reference, Reference::Null) {
        return;
    }
    let reference = reference.unwrap();

    let class_index = ((parser.next() as u16) << 8) | (parser.next() as u16);
    let class = class.constant_pool().get_class(class_index as usize)
        .expect(&format!("Invalid cast check! Expected index {} to be in constant \
            pool!", class_index));
    assert!(reference.class().is_subclass(Arc::clone(&class)), "Cannot cast {} to {}!",
        reference.class().name(), Arc::clone(&class).name());
    frame.push_ref_op(reference.offset() as u32);
}

#[inline]
fn common_array_primitive(
    heap: &HeapSpace,
    frame: &mut StackFrame,
    mapper: impl Fn(&mut StackFrame, Arc<TypeArrayObject>, ArrayType, usize)
) {
    let array_ref = frame.pop_type_array_op(heap)
        .expect("Invalid array reference on operand stack! Reference cannot be null!");
    let array_type = array_ref.array_type();
    let index = frame.pop_int_op() as usize;
    mapper(frame, array_ref, array_type, index)
}

#[inline]
fn array_primitive(
    heap: &HeapSpace,
    frame: &mut StackFrame,
    instruction: &str,
    expected_type: &str,
    checker: impl Fn(ArrayType) -> bool,
    mapper: impl Fn(&mut StackFrame, Arc<TypeArrayObject>, usize)
) {
    common_array_primitive(heap, frame, |frame, array, array_type, index| {
        if checker(array_type) {
            mapper(frame, array, index)
        }
        panic!("Invalid type of array for {}! Expected array to be of type {}, was {}!",
            instruction, expected_type, array_type)
    })
}

pub(super) fn pop(frame: &mut StackFrame, double: bool) {
    frame.pop_op();
    if double {
        frame.pop_op();
    }
}

pub(super) fn dup(frame: &mut StackFrame) {
    let value = frame.get_op(0);
    frame.push_op(value);
}

pub(super) fn dup_x1(frame: &mut StackFrame) {
    let first = frame.get_op(0);
    let second = frame.get_op(1);
    frame.set_op(1, first);
    frame.set_op(0, second);
    frame.push_op(first);
}

pub(super) fn dup_x2(frame: &mut StackFrame) {
    let first = frame.get_op(0);
    let third = frame.get_op(2);
    frame.set_op(2, first);
    frame.push_op(first);
    let second = frame.get_op(2);
    frame.set_op(2, third);
    frame.set_op(1, second);
}

pub(super) fn dup2(frame: &mut StackFrame) {
    let high = frame.get_op(1);
    frame.push_op(high);
    let low = frame.get_op(1);
    frame.push_op(low);
}

pub(super) fn dup2_x1(frame: &mut StackFrame) {
    let first = frame.get_op(0);
    let second = frame.get_op(1);
    frame.push_op(second);
    frame.push_op(first);
    frame.set_op(3, first);
    let third = frame.get_op(4);
    frame.set_op(2, third);
    frame.set_op(4, second);
}

pub(super) fn dup2_x2(frame: &mut StackFrame) {
    let first = frame.get_op(0);
    let second = frame.get_op(1);
    frame.push_op(second);
    frame.push_op(first);
    let third = frame.get_op(4);
    frame.set_op(2, third);
    frame.set_op(4, first);
    let fourth = frame.get_op(5);
    let fifth = frame.get_op(3);
    frame.set_op(3, fourth);
    frame.set_op(5, fifth);
}

pub(super) fn swap(frame: &mut StackFrame) {
    let first = frame.get_op(1);
    let second = frame.get_op(0);
    frame.set_op(0, first);
    frame.set_op(1, second);
}

pub(super) fn branch<'a>(frame: &mut StackFrame, parser: &mut CodeParser<'a>, op: u8) {
    let value = frame.pop_int_op();
    let success = (op == IFEQ && value == 0) ||
        (op == IFNE && value != 0) ||
        (op == IFLT && value < 0) ||
        (op == IFLE && value <= 0) ||
        (op == IFGT && value > 0) ||
        (op == IFGE && value >= 0);
    if success {
        branch_seek(parser);
    }
}

pub(super) fn branch_null(
    heap: &HeapSpace,
    frame: &mut StackFrame,
    parser: &mut CodeParser,
    null: bool
) {
    match frame.pop_ref_op(heap) {
        Reference::Value(_) if !null => branch_seek(parser),
        Reference::Null if null => branch_seek(parser),
        _ => {}
    }
}

pub(super) fn instanceof(
    heap: &HeapSpace,
    class: &Class,
    frame: &mut StackFrame,
    parser: &mut CodeParser
) {
    let reference = frame.pop_ref_op(heap);
    let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
    if let Reference::Null = reference {
        frame.push_int_op(0);
        return;
    }
    let reference = reference.unwrap();

    let class = class.constant_pool().get_class(index as usize)
        .expect(&format!("Invalid class for instanceof check! Expected index {} to be in \
            constant pool!", index));
    let result = if reference.class().is_subclass(class) { 1 } else { 0 };
    frame.push_int_op(result);
}

pub(super) fn ref_branch(heap: &HeapSpace, frame: &mut StackFrame, parser: &mut CodeParser, op: u8) {
    let first_ref = frame.pop_ref_op(heap);
    let second_ref = frame.pop_ref_op(heap);
    let ref_compare = first_ref.equals(second_ref);
    if (op == IF_ACMPEQ && ref_compare) || (op == IF_ACMPNE && !ref_compare) {
        branch_seek(parser);
    }
}

pub(super) fn int_branch(frame: &mut StackFrame, parser: &mut CodeParser, op: u8) {
    let first = frame.pop_int_op();
    let second = frame.pop_int_op();
    let success = (op  == IF_ICMPEQ && first == second) ||
        (op == IF_ICMPNE && first != second) ||
        (op == IF_ICMPLT && first < second) ||
        (op == IF_ICMPLE && first <= second) ||
        (op == IF_ICMPGT && first > second) ||
        (op == IF_ICMPGE && first >= second);
    if success {
        branch_seek(parser);
    }
}

pub(super) fn jump_subroutine(frame: &mut StackFrame, parser: &mut CodeParser, wide: bool) {
    frame.push_op(parser.next_index());
    if wide {
        branch_seek_wide(parser);
    } else {
        branch_seek(parser);
    }
}

pub(super) fn branch_seek(parser: &mut CodeParser) {
    let index = ((parser.next() as i16) << 8) | (parser.next() as i16);
    parser.seek_relative(index as usize);
}

pub(super) fn branch_seek_wide(parser: &mut CodeParser) {
    let index = ((parser.next() as i32) << 24) |
        ((parser.next() as i32) << 16) |
        ((parser.next() as i32) << 8) |
        (parser.next() as i32);
    parser.seek_relative(index as usize);
}

pub(super) fn new_ref(
    heap: &HeapSpace,
    class: &Class,
    frame: &mut StackFrame,
    parser: &mut CodeParser
) {
    let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
    let class = class.constant_pool().get_class(index as usize)
        .expect(&format!("Invalid object instantiation! Expected index {} to be in constant \
            pool!", index));
    if class.is_interface() || class.is_abstract() {
        panic!("Attempted to instantiate an interface or abstract class!");
    }

    let offset = heap.len(); // Index of next element will be the current length
    let field_count = class.field_count();
    let instance = InstanceObject::new(offset, Arc::clone(&class), field_count);
    for i in 0..field_count {
        // Everything gets initialised to default values. For primitives, this is 0.
        // For references, this is null, but the offset of null references is 0.
        instance.set(i, 0);
    }

    heap.push_ref(Arc::new(instance));
    frame.push_ref_op(offset as u32);
}

pub(super) fn new_ref_array(
    heap: &HeapSpace,
    class: &Class,
    frame: &mut StackFrame,
    parser: &mut CodeParser
) {
    let count = frame.pop_int_op();
    let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
    let class = class.constant_pool().get_class(index as usize)
        .expect(&format!("Invalid class type index {}!", index));

    let offset = heap.len(); // Index of next element will be the current length
    let array = ReferenceArrayObject::new(offset, Arc::clone(&class), class, count as usize);
    heap.push_ref_array(Arc::new(array));
    frame.push_ref_op(offset as u32);
}

pub(super) fn new_type_array(heap: &HeapSpace, frame: &mut StackFrame, parser: &mut CodeParser) {
    let array_type = ArrayType::from(parser.next()).expect("Invalid array type!");
    let count = frame.pop_int_op();
    let offset = heap.len(); // Index of next element will be the current length
    let array = TypeArrayObject::new(offset, array_type, count as usize);
    heap.push_type_array(Arc::new(array));
    frame.push_ref_op(offset as u32);
}
