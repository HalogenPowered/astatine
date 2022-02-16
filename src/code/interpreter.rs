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

mod instructions;
mod primitive_ops;

use paste::paste;
use instructions::*;
use primitive_ops::*;
use crate::class_file::code::CodeBlock;
use crate::constants::*;
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
                JVM_OPCODE_NOP => {}
                JVM_OPCODE_ACONST_NULL => frame.push_null_op(),
                JVM_OPCODE_ICONST_M1..=JVM_OPCODE_ICONST_5 => {
                    frame.push_int_op((op as i32) - (JVM_OPCODE_ICONST_0 as i32))
                }
                JVM_OPCODE_LCONST_0 | JVM_OPCODE_LCONST_1 => {
                    frame.push_long_op((op as i64) - (JVM_OPCODE_LCONST_0 as i64))
                }
                JVM_OPCODE_FCONST_0..=JVM_OPCODE_FCONST_2 => {
                    frame.push_float_op((op as f32) - (JVM_OPCODE_FCONST_0 as f32))
                }
                JVM_OPCODE_DCONST_0 | JVM_OPCODE_DCONST_1 => {
                    frame.push_double_op((op as f64) - (JVM_OPCODE_DCONST_0 as f64))
                }
                JVM_OPCODE_BIPUSH => frame.push_byte_op(parser.next() as i8),
                JVM_OPCODE_SIPUSH => {
                    frame.push_short_op((((parser.next() as i32) << 8) | (parser.next() as i32)) as i16)
                }
                // TODO: LDC, LDC_W, and LDC2_W
                JVM_OPCODE_ILOAD => jvm_load_int(&mut frame, parser.next()),
                JVM_OPCODE_LLOAD => jvm_load_long(&mut frame, parser.next()),
                JVM_OPCODE_FLOAD => jvm_load_float(&mut frame, parser.next()),
                JVM_OPCODE_DLOAD => jvm_load_double(&mut frame, parser.next()),
                JVM_OPCODE_ALOAD => load_ref(heap, &mut frame, parser.next()),
                JVM_OPCODE_ILOAD_0..=JVM_OPCODE_ILOAD_3 => jvm_load_int(&mut frame, iload_index(op)),
                JVM_OPCODE_LLOAD_0..=JVM_OPCODE_LLOAD_3 => jvm_load_long(&mut frame, lload_index(op)),
                JVM_OPCODE_FLOAD_0..=JVM_OPCODE_FLOAD_3 => jvm_load_float(&mut frame, fload_index(op)),
                JVM_OPCODE_DLOAD_0..=JVM_OPCODE_DLOAD_3 => jvm_load_double(&mut frame, dload_index(op)),
                JVM_OPCODE_ALOAD_0..=JVM_OPCODE_ALOAD_3 => load_ref(heap, &mut frame, aload_index(op)),
                JVM_OPCODE_IALOAD => load_array_int(heap, &mut frame),
                JVM_OPCODE_LALOAD => load_array_long(heap, &mut frame),
                JVM_OPCODE_FALOAD => load_array_float(heap, &mut frame),
                JVM_OPCODE_DALOAD => load_array_double(heap, &mut frame),
                JVM_OPCODE_AALOAD => load_array_ref(heap, &mut frame),
                JVM_OPCODE_BALOAD => load_array_byte(heap, &mut frame),
                JVM_OPCODE_CALOAD => load_array_char(heap, &mut frame),
                JVM_OPCODE_SALOAD => load_array_short(heap, &mut frame),
                JVM_OPCODE_ISTORE => jvm_store_int(&mut frame, parser.next()),
                JVM_OPCODE_LSTORE => jvm_store_long(&mut frame, parser.next()),
                JVM_OPCODE_FSTORE => jvm_store_float(&mut frame, parser.next()),
                JVM_OPCODE_DSTORE => jvm_store_double(&mut frame, parser.next()),
                JVM_OPCODE_ASTORE => store_ref(heap, &mut frame, parser.next()),
                JVM_OPCODE_ISTORE_0..=JVM_OPCODE_ISTORE_3 => jvm_store_int(&mut frame, istore_index(op)),
                JVM_OPCODE_LSTORE_0..=JVM_OPCODE_LSTORE_3 => jvm_store_long(&mut frame, lstore_index(op)),
                JVM_OPCODE_FSTORE_0..=JVM_OPCODE_FSTORE_3 => jvm_store_float(&mut frame, fstore_index(op)),
                JVM_OPCODE_DSTORE_0..=JVM_OPCODE_DSTORE_3 => jvm_store_double(&mut frame, dstore_index(op)),
                JVM_OPCODE_ASTORE_0..=JVM_OPCODE_ASTORE_3 => store_ref(heap, &mut frame, astore_index(op)),
                JVM_OPCODE_IASTORE => store_array_int(heap, &mut frame),
                JVM_OPCODE_LASTORE => store_array_long(heap, &mut frame),
                JVM_OPCODE_FASTORE => store_array_float(heap, &mut frame),
                JVM_OPCODE_DASTORE => store_array_double(heap, &mut frame),
                JVM_OPCODE_AASTORE => store_array_ref(heap, &mut frame),
                JVM_OPCODE_BASTORE => store_array_byte(heap, &mut frame),
                JVM_OPCODE_CASTORE => store_array_char(heap, &mut frame),
                JVM_OPCODE_SASTORE => store_array_short(heap, &mut frame),
                JVM_OPCODE_POP => pop(&mut frame, false),
                JVM_OPCODE_POP2 => pop(&mut frame, true),
                JVM_OPCODE_DUP => dup(&mut frame),
                JVM_OPCODE_DUP_X1 => dup_x1(&mut frame),
                JVM_OPCODE_DUP_X2 => dup_x2(&mut frame),
                JVM_OPCODE_DUP2 => dup2(&mut frame),
                JVM_OPCODE_DUP2_X1 => dup2_x1(&mut frame),
                JVM_OPCODE_DUP2_X2 => dup2_x2(&mut frame),
                JVM_OPCODE_SWAP => swap(&mut frame),
                JVM_OPCODE_IADD => jvm_int_add(&mut frame),
                JVM_OPCODE_LADD => jvm_long_add(&mut frame),
                JVM_OPCODE_FADD => jvm_float_add(&mut frame),
                JVM_OPCODE_DADD => jvm_double_add(&mut frame),
                JVM_OPCODE_ISUB => jvm_int_sub(&mut frame),
                JVM_OPCODE_LSUB => jvm_long_sub(&mut frame),
                JVM_OPCODE_FSUB => jvm_float_sub(&mut frame),
                JVM_OPCODE_DSUB => jvm_double_sub(&mut frame),
                JVM_OPCODE_IMUL => jvm_int_mul(&mut frame),
                JVM_OPCODE_LMUL => jvm_long_mul(&mut frame),
                JVM_OPCODE_FMUL => jvm_float_mul(&mut frame),
                JVM_OPCODE_DMUL => jvm_double_mul(&mut frame),
                JVM_OPCODE_IDIV => jvm_int_div(&mut frame),
                JVM_OPCODE_LDIV => jvm_long_div(&mut frame),
                JVM_OPCODE_FDIV => jvm_float_div(&mut frame),
                JVM_OPCODE_DDIV => jvm_double_div(&mut frame),
                JVM_OPCODE_IREM => jvm_int_rem(&mut frame),
                JVM_OPCODE_LREM => jvm_long_rem(&mut frame),
                JVM_OPCODE_FREM => jvm_float_rem(&mut frame),
                JVM_OPCODE_DREM => jvm_double_rem(&mut frame),
                JVM_OPCODE_INEG => jvm_int_neg(&mut frame),
                JVM_OPCODE_LNEG => jvm_long_neg(&mut frame),
                JVM_OPCODE_FNEG => jvm_float_neg(&mut frame),
                JVM_OPCODE_DNEG => jvm_double_neg(&mut frame),
                JVM_OPCODE_ISHL => jvm_int_shl(&mut frame),
                JVM_OPCODE_LSHL => jvm_long_shl(&mut frame),
                JVM_OPCODE_ISHR => jvm_int_shr(&mut frame),
                JVM_OPCODE_LSHR => jvm_long_shr(&mut frame),
                JVM_OPCODE_IUSHR => jvm_int_ushr(&mut frame),
                JVM_OPCODE_LUSHR => jvm_long_ushr(&mut frame),
                JVM_OPCODE_IAND => jvm_int_and(&mut frame),
                JVM_OPCODE_LAND => jvm_long_and(&mut frame),
                JVM_OPCODE_IOR => jvm_int_or(&mut frame),
                JVM_OPCODE_LOR => jvm_long_or(&mut frame),
                JVM_OPCODE_IXOR => jvm_int_xor(&mut frame),
                JVM_OPCODE_LXOR => jvm_long_xor(&mut frame),
                JVM_OPCODE_IINC => jvm_int_inc(&mut frame),
                JVM_OPCODE_I2L => jvm_int_to_long(&mut frame),
                JVM_OPCODE_I2F => jvm_int_to_float(&mut frame),
                JVM_OPCODE_I2D => jvm_int_to_double(&mut frame),
                JVM_OPCODE_L2I => jvm_long_to_int(&mut frame),
                JVM_OPCODE_L2F => jvm_long_to_float(&mut frame),
                JVM_OPCODE_L2D => jvm_long_to_double(&mut frame),
                JVM_OPCODE_F2I => jvm_float_to_int(&mut frame),
                JVM_OPCODE_F2L => jvm_float_to_long(&mut frame),
                JVM_OPCODE_F2D => jvm_float_to_double(&mut frame),
                JVM_OPCODE_D2I => jvm_double_to_int(&mut frame),
                JVM_OPCODE_D2L => jvm_double_to_long(&mut frame),
                JVM_OPCODE_D2F => jvm_double_to_float(&mut frame),
                JVM_OPCODE_I2B => jvm_int_to_byte(&mut frame),
                JVM_OPCODE_I2C => jvm_int_to_char(&mut frame),
                JVM_OPCODE_I2S => jvm_int_to_short(&mut frame),
                JVM_OPCODE_LCMP => jvm_cmp_long(&mut frame),
                JVM_OPCODE_FCMPL => jvm_cmp_float(&mut frame, false),
                JVM_OPCODE_FCMPG => jvm_cmp_float(&mut frame, true),
                JVM_OPCODE_DCMPL => jvm_cmp_double(&mut frame, false),
                JVM_OPCODE_DCMPG => jvm_cmp_double(&mut frame, true),
                JVM_OPCODE_IFEQ..=JVM_OPCODE_IFLE => branch(&mut frame, &mut parser, op),
                JVM_OPCODE_IF_ICMPEQ..=JVM_OPCODE_IF_ICMPLE => int_branch(&mut frame, &mut parser, op),
                JVM_OPCODE_IF_ACMPEQ | JVM_OPCODE_IF_ACMPNE => ref_branch(heap, &mut frame, &mut parser, op),
                JVM_OPCODE_GOTO => branch_seek(&mut parser),
                JVM_OPCODE_JSR => jump_subroutine(&mut frame, &mut parser, false),
                // TODO: RET, TABLESWITCH, LOOKUPSWITCH
                JVM_OPCODE_IRETURN => return MethodResult::Integer(frame.pop_int_op()),
                JVM_OPCODE_LRETURN => return MethodResult::Long(frame.pop_long_op()),
                JVM_OPCODE_FRETURN => return MethodResult::Float(frame.pop_float_op()),
                JVM_OPCODE_DRETURN => return MethodResult::Double(frame.pop_double_op()),
                JVM_OPCODE_ARETURN => return MethodResult::Reference(frame.pop_ref_op(heap)),
                JVM_OPCODE_RETURN => return MethodResult::Void,
                // TODO: GETSTATIC, PUTSTATIC, GETFIELD, PUTFIELD, INVOKEVIRTUAL, INVOKESPECIAL,
                //  INVOKESTATIC, INVOKEINTERFACE, INVOKEDYNAMIC
                JVM_OPCODE_NEW => new_ref(heap, class, &mut frame, &mut parser),
                JVM_OPCODE_NEWARRAY => new_type_array(heap, &mut frame, &mut parser),
                JVM_OPCODE_ANEWARRAY => new_ref_array(heap, class, &mut frame, &mut parser),
                JVM_OPCODE_ARRAYLENGTH => array_length(heap, &mut frame),
                JVM_OPCODE_ATHROW => {
                    if let Some(result) = throw(heap, code, &mut frame, &mut parser) {
                        return result;
                    }
                }
                JVM_OPCODE_CHECKCAST => check_cast(heap, class, &mut frame, &mut parser),
                JVM_OPCODE_INSTANCEOF => instanceof(heap, class, &mut frame, &mut parser),
                // TODO: MONITORENTER, MONITOREXIT, WIDE, MULTIANEWARRAY
                JVM_OPCODE_IFNULL => branch_null(heap, &mut frame, &mut parser, true),
                JVM_OPCODE_IFNONNULL => branch_null(heap, &mut frame, &mut parser, false),
                JVM_OPCODE_GOTO_W => branch_seek_wide(&mut parser),
                JVM_OPCODE_JSR_W => jump_subroutine(&mut frame, &mut parser, true),
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
            generate_load_store_index!([<$prefix load>], [<JVM_OPCODE_ $prefix:upper LOAD>]);
            generate_load_store_index!([<$prefix store>], [<JVM_OPCODE_ $prefix:upper STORE>]);
        }
    }
}

generate_load_store_index!(a);
generate_load_store_index!(d);
generate_load_store_index!(f);
generate_load_store_index!(i);
generate_load_store_index!(l);
