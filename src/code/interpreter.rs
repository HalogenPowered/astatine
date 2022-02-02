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

mod constants;
mod instructions;
mod primitive_ops;

use paste::paste;
use constants::*;
use instructions::*;
use primitive_ops::*;
use crate::class_file::code::CodeBlock;
use crate::objects::*;
use crate::types::Class;

pub struct Interpreter {
    _singleton: ()
}

impl Interpreter {
    pub fn execute(
        heap: &HeapSpace,
        class: &Class,
        code: &CodeBlock,
        parameters: &[u32]
    ) -> MethodResult {
        let mut frame = code.new_stack_frame();
        for parameter in parameters {
            frame.push_op(*parameter);
        }

        let mut parser = CodeParser::new(code.code());
        while !parser.is_empty() {
            let op = parser.next();
            match op {
                NOP => {},
                ACONST_NULL => frame.push_null_op(),
                ICONST_M1..=ICONST_5 => frame.push_int_op((op as i32) - (ICONST_0 as i32)),
                LCONST_0..=LCONST_1 => frame.push_long_op((op as i64) - (LCONST_0 as i64)),
                FCONST_0..=FCONST_2 => frame.push_float_op((op as f32) - (FCONST_0 as f32)),
                DCONST_0..=DCONST_1 => frame.push_double_op((op as f64) - (DCONST_0 as f64)),
                BIPUSH => frame.push_byte_op(parser.next() as i8),
                SIPUSH => frame.push_short_op((((parser.next() as i32) << 8) | (parser.next() as i32)) as i16),
                // TODO: LDC, LDC_W, and LDC2_W
                ILOAD => jvm_load_int(&mut frame, parser.next()),
                LLOAD => jvm_load_long(&mut frame, parser.next()),
                FLOAD => jvm_load_float(&mut frame, parser.next()),
                DLOAD => jvm_load_double(&mut frame, parser.next()),
                ALOAD => load_ref(heap, &mut frame, parser.next()),
                ILOAD_0..=ILOAD_3 => jvm_load_int(&mut frame, iload_index(op)),
                LLOAD_0..=LLOAD_3 => jvm_load_long(&mut frame, lload_index(op)),
                FLOAD_0..=FLOAD_3 => jvm_load_float(&mut frame, fload_index(op)),
                DLOAD_0..=DLOAD_3 => jvm_load_double(&mut frame, dload_index(op)),
                ALOAD_0..=ALOAD_3 => load_ref(heap, &mut frame, aload_index(op)),
                IALOAD => load_array_int(heap, &mut frame),
                LALOAD => load_array_long(heap, &mut frame),
                FALOAD => load_array_float(heap, &mut frame),
                DALOAD => load_array_double(heap, &mut frame),
                AALOAD => load_array_ref(heap, &mut frame),
                BALOAD => load_array_byte(heap, &mut frame),
                CALOAD => load_array_char(heap, &mut frame),
                SALOAD => load_array_short(heap, &mut frame),
                ISTORE => jvm_store_int(&mut frame, parser.next()),
                LSTORE => jvm_store_long(&mut frame, parser.next()),
                FSTORE => jvm_store_float(&mut frame, parser.next()),
                DSTORE => jvm_store_double(&mut frame, parser.next()),
                ASTORE => store_ref(heap, &mut frame, parser.next()),
                ISTORE_0..=ISTORE_3 => jvm_store_int(&mut frame, istore_index(op)),
                LSTORE_0..=LSTORE_3 => jvm_store_long(&mut frame, lstore_index(op)),
                FSTORE_0..=FSTORE_3 => jvm_store_float(&mut frame, fstore_index(op)),
                DSTORE_0..=DSTORE_3 => jvm_store_double(&mut frame, dstore_index(op)),
                ASTORE_0..=ASTORE_3 => store_ref(heap, &mut frame, astore_index(op)),
                IASTORE => store_array_int(heap, &mut frame),
                LASTORE => store_array_long(heap, &mut frame),
                FASTORE => store_array_float(heap, &mut frame),
                DASTORE => store_array_double(heap, &mut frame),
                AASTORE => store_array_ref(heap, &mut frame),
                BASTORE => store_array_byte(heap, &mut frame),
                CASTORE => store_array_char(heap, &mut frame),
                SASTORE => store_array_short(heap, &mut frame),
                POP => pop(&mut frame, false),
                POP2 => pop(&mut frame, true),
                DUP => dup(&mut frame),
                DUP_X1 => dup_x1(&mut frame),
                DUP_X2 => dup_x2(&mut frame),
                DUP2 => dup2(&mut frame),
                DUP2_X1 => dup2_x1(&mut frame),
                DUP2_X2 => dup2_x2(&mut frame),
                SWAP => swap(&mut frame),
                IADD => jvm_int_add(&mut frame),
                LADD => jvm_long_add(&mut frame),
                FADD => jvm_float_add(&mut frame),
                DADD => jvm_double_add(&mut frame),
                ISUB => jvm_int_sub(&mut frame),
                LSUB => jvm_long_sub(&mut frame),
                FSUB => jvm_float_sub(&mut frame),
                DSUB => jvm_double_sub(&mut frame),
                IMUL => jvm_int_mul(&mut frame),
                LMUL => jvm_long_mul(&mut frame),
                FMUL => jvm_float_mul(&mut frame),
                DMUL => jvm_double_mul(&mut frame),
                IDIV => jvm_int_div(&mut frame),
                LDIV => jvm_long_div(&mut frame),
                FDIV => jvm_float_div(&mut frame),
                DDIV => jvm_double_div(&mut frame),
                IREM => jvm_int_rem(&mut frame),
                LREM => jvm_long_rem(&mut frame),
                FREM => jvm_float_rem(&mut frame),
                DREM => jvm_double_rem(&mut frame),
                INEG => jvm_int_neg(&mut frame),
                LNEG => jvm_long_neg(&mut frame),
                FNEG => jvm_float_neg(&mut frame),
                DNEG => jvm_double_neg(&mut frame),
                ISHL => jvm_int_shl(&mut frame),
                LSHL => jvm_long_shl(&mut frame),
                ISHR => jvm_int_shr(&mut frame),
                LSHR => jvm_long_shr(&mut frame),
                IUSHR => jvm_int_ushr(&mut frame),
                LUSHR => jvm_long_ushr(&mut frame),
                IAND => jvm_int_and(&mut frame),
                LAND => jvm_long_and(&mut frame),
                IOR => jvm_int_or(&mut frame),
                LOR => jvm_long_or(&mut frame),
                IXOR => jvm_int_xor(&mut frame),
                LXOR => jvm_long_xor(&mut frame),
                IINC => jvm_int_inc(&mut frame),
                I2L => jvm_int_to_long(&mut frame),
                I2F => jvm_int_to_float(&mut frame),
                I2D => jvm_int_to_double(&mut frame),
                L2I => jvm_long_to_int(&mut frame),
                L2F => jvm_long_to_float(&mut frame),
                L2D => jvm_long_to_double(&mut frame),
                F2I => jvm_float_to_int(&mut frame),
                F2L => jvm_float_to_long(&mut frame),
                F2D => jvm_float_to_double(&mut frame),
                D2I => jvm_double_to_int(&mut frame),
                D2L => jvm_double_to_long(&mut frame),
                D2F => jvm_double_to_float(&mut frame),
                I2B => jvm_int_to_byte(&mut frame),
                I2C => jvm_int_to_char(&mut frame),
                I2S => jvm_int_to_short(&mut frame),
                LCMP => jvm_cmp_long(&mut frame),
                FCMPL => jvm_cmp_float(&mut frame, false),
                FCMPG => jvm_cmp_float(&mut frame, true),
                DCMPL => jvm_cmp_double(&mut frame, false),
                DCMPG => jvm_cmp_double(&mut frame, true),
                IFEQ | IFNE | IFLT | IFGE | IFGT | IFLE => branch(&mut frame, &mut parser, op),
                IF_ICMPEQ..=IF_ICMPLE => int_branch(&mut frame, &mut parser, op),
                IF_ACMPEQ | IF_ACMPNE => ref_branch(heap, &mut frame, &mut parser, op),
                GOTO => branch_seek(&mut parser),
                JSR => jump_subroutine(&mut frame, &mut parser, false),
                // TODO: RET, TABLESWITCH, LOOKUPSWITCH
                IRETURN => return MethodResult::Integer(frame.pop_int_op()),
                LRETURN => return MethodResult::Long(frame.pop_long_op()),
                FRETURN => return MethodResult::Float(frame.pop_float_op()),
                DRETURN => return MethodResult::Double(frame.pop_double_op()),
                ARETURN => return MethodResult::Reference(frame.pop_ref_op(heap)),
                RETURN => return MethodResult::Void,
                // TODO: GETSTATIC, PUTSTATIC, GETFIELD, PUTFIELD, INVOKEVIRTUAL, INVOKESPECIAL,
                //  INVOKESTATIC, INVOKEINTERFACE, INVOKEDYNAMIC
                NEW => new_ref(heap, class, &mut frame, &mut parser),
                NEWARRAY => new_type_array(heap, &mut frame, &mut parser),
                ANEWARRAY => new_ref_array(heap, class, &mut frame, &mut parser),
                ARRAYLENGTH => array_length(heap, &mut frame),
                ATHROW => {
                    if let Some(result) = throw(heap, code, &mut frame, &mut parser) {
                        return result;
                    }
                },
                CHECKCAST => check_cast(heap, class, &mut frame, &mut parser),
                INSTANCEOF => instanceof(heap, class, &mut frame, &mut parser),
                // TODO: MONITORENTER, MONITOREXIT, WIDE, MULTIANEWARRAY
                IFNULL => branch_null(heap, &mut frame, &mut parser, true),
                IFNONNULL => branch_null(heap, &mut frame, &mut parser, false),
                GOTO_W => branch_seek_wide(&mut parser),
                JSR_W => jump_subroutine(&mut frame, &mut parser, true),
                _ => panic!("Unrecognised bytecode {}!", op)
            }
        }
        panic!("Method should have returned by this point!");
    }
}

struct CodeParser<'a> {
    bytes: &'a [u8],
    index: u16
}

impl<'a> CodeParser<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        CodeParser { bytes, index: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn next(&mut self) -> u8 {
        let next = self.bytes[self.index as usize];
        self.index += 1;
        next
    }

    pub fn next_index(&self) -> u32 {
        (self.index + 1) as u32
    }

    pub fn seek(&mut self, index: usize) {
        self.index = index as u16;
    }

    pub fn seek_relative(&mut self, offset: usize) {
        self.index += offset as u16;
    }
}

pub enum MethodResult {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(Reference<InstanceObject>),
    Void,
    Exception
}

pub const NUMBER_OF_JAVA_OP_CODES: u8 = 203;

macro_rules! generate_load_store_index {
    ($name:ident, $prefix:ident) => {
        paste! {
            fn [<$name _index>](op: u8) -> u8 {
                if op < [<$prefix _0>] || op > [<$prefix _3>] {
                    panic!("{} called with op < {} or > {}! Op was {}!", "[<$name _index>]",
                        "[<$prefix _0>]", "[<$prefix _3>]", op);
                }
                op - [<$prefix _0>]
            }
        }
    };
    ($prefix:ident) => {
        paste! {
            generate_load_store_index!([<$prefix load>], [<$prefix:upper LOAD>]);
            generate_load_store_index!([<$prefix store>], [<$prefix:upper STORE>]);
        }
    }
}

generate_load_store_index!(a);
generate_load_store_index!(d);
generate_load_store_index!(f);
generate_load_store_index!(i);
generate_load_store_index!(l);
