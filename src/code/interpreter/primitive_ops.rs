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
use crate::code::stack_frame::StackFrame;

macro_rules! primitive_op {
    ($name:ident, $op_name:ident, $op:tt) => {
        paste! {
            pub fn [<jvm_ $name _ $op_name>](frame: &mut StackFrame) {
                let first = frame.[<pop_ $name _op>]();
                let second = frame.[<pop_ $name _op>]();
                frame.[<push_ $name _op>](first $op second);
            }
        }
    };
    ($name:ident, $op_name:ident, $op:tt, $second:literal) => {
        paste! {
            pub fn [<jvm_ $name _ $op_name>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $name _op>]();
                frame.[<push_ $name _op>](value $op $second);
            }
        }
    }
}

macro_rules! primitive_negate {
    ($name:ident) => {
        paste! {
            pub fn [<jvm_ $name _neg>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $name _op>]();
                frame.[<push_ $name _op>](-value);
            }
        }
    }
}

macro_rules! primitive_ushr {
    ($name:ident, $primitive:ty, $unsigned:ty) => {
        paste! {
            pub fn [<jvm_ $name _ushr>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $name _op>]();
                let amount = frame.[<pop_ $name _op>]();
                frame.[<push_ $name _op>](((value as $unsigned) >> amount) as $primitive);
            }
        }
    }
}

macro_rules! primitive_conversion {
    ($from:ident, $to:ident, $target:ty) => {
        paste! {
            pub fn [<jvm_ $from _to_ $to>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $from _op>]();
                frame.[<push_ $to _op>](value as $target);
            }
        }
    }
}

macro_rules! floating_compare {
    ($primitive:ident) => {
        paste! {
            pub fn [<jvm_cmp_ $primitive>](frame: &mut StackFrame, greater: bool) {
                let first = frame.[<pop_ $primitive _op>]();
                let second = frame.[<pop_ $primitive _op>]();
                let result = if first > second {
                    1
                } else if second > first {
                    -1
                } else if second == first {
                    0
                } else {
                    if greater {
                        1
                    } else {
                        -1
                    }
                };
                frame.push_int_op(result);
            }
        }
    }
}

macro_rules! primitive_load_store {
    ($name:ident, $primitive:ty) => {
        paste! {
            pub fn [<jvm_load_ $name>](frame: &mut StackFrame, index: u8) {
                let value = frame.[<get_local_ $name>](index as usize);
                frame.[<push_ $name _op>](value as $primitive);
            }

            pub fn [<jvm_store_ $name>](frame: &mut StackFrame, index: u8) {
                let value = frame.[<pop_ $name _op>]();
                frame.[<set_local_ $name>](index as usize, value);
            }
        }
    }
}

macro_rules! to_from_floating {
    ($name:ident, $primitive:ty) => {
        paste! {
            pub fn [<jvm_ $name _to_float>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $name _op>]();
                frame.push_float_op(f32::from_bits(value as u32));
            }

            pub fn [<jvm_ $name _to_double>](frame: &mut StackFrame) {
                let value = frame.[<pop_ $name _op>]();
                frame.push_double_op(f64::from_bits(value as u64));
            }

            pub fn [<jvm_float_to_ $name>](frame: &mut StackFrame) {
                let value = frame.pop_float_op();
                frame.[<push_ $name _op>](value.to_bits() as $primitive);
            }

            pub fn [<jvm_double_to_ $name>](frame: &mut StackFrame) {
                let value = frame.pop_double_op();
                frame.[<push_ $name _op>](value.to_bits() as $primitive);
            }
        }
    }
}

macro_rules! generate_shared_functions {
    ($name:ident, $primitive:ty) => {
        primitive_op!($name, add, +);
        primitive_op!($name, div, /);
        primitive_op!($name, mul, *);
        primitive_negate!($name);
        primitive_op!($name, rem, %);
        primitive_op!($name, sub, -);
        primitive_load_store!($name, $primitive);
    }
}

macro_rules! generate_int_long_functions {
    ($name:ident, $primitive:ty, $unsigned:ty) => {
        generate_shared_functions!($name, $primitive);
        primitive_op!($name, and, &);
        primitive_op!($name, or, |);
        primitive_op!($name, shl, <<);
        primitive_op!($name, shr, >>);
        primitive_ushr!($name, $primitive, $unsigned);
        primitive_op!($name, xor, ^);
    }
}

generate_int_long_functions!(int, i32, u32);
generate_int_long_functions!(long, i64, u64);
generate_shared_functions!(float, f32);
generate_shared_functions!(double, f64);

pub fn jvm_int_inc(frame: &mut StackFrame) {
    let value = frame.pop_int_op();
    frame.push_int_op(value + 1);
}

// Integer conversion
primitive_conversion!(int, byte, i8);
to_from_floating!(int, i32);
primitive_conversion!(int, long, i64);
primitive_conversion!(int, short, i16);

// Long conversion
to_from_floating!(long, i64);
primitive_conversion!(long, int, i32);

pub fn jvm_int_to_char(frame: &mut StackFrame) {
    let value = frame.pop_int_op();
    frame.push_char_op(char::from_u32(value as u32).expect(&format!("{} is not a valid unicode code point!", value)));
}

primitive_conversion!(double, float, f32);

pub fn jvm_float_to_double(frame: &mut StackFrame) {
    let value = frame.pop_float_op();
    frame.push_double_op(value.into());
}

// Comparisons
floating_compare!(double);
floating_compare!(float);

pub fn jvm_cmp_long(frame: &mut StackFrame) {
    let first = frame.pop_long_op();
    let second = frame.pop_long_op();
    let result = if first > second { 1 } else if second > first { -1 } else { 0 };
    frame.push_int_op(result);
}
