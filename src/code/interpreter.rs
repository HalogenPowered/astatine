mod primitive_ops;
use primitive_ops::*;

use std::rc::Rc;
use paste::paste;
use super::stack_frame::StackFrame;
use crate::class_file::class_loader::ClassLoader;
use crate::class_file::code::CodeBlock;
use crate::objects::heap::HeapSpace;
use crate::objects::object::*;
use crate::objects::reference::Reference;
use crate::types::class::Class;
use crate::types::utils::Nameable;
use crate::utils::vm_types::ArrayType;
use crate::types::access_flags::*;

pub struct Interpreter {
    _singleton: ()
}

macro_rules! load_store_array_primitive {
    ($name:ident, $instruction_prefix:literal, $expected:literal, $array_type:pat) => {
        paste! {
            fn [<load_array_ $name>](context: &InterpreterContext, frame: &mut StackFrame) {
                Interpreter::array_primitive(context, frame,
                                             &format!("{}ALOAD", $instruction_prefix), $expected,
                                             |array_type| matches!(array_type, $array_type),
                                             |frame, array, index| frame.[<push_ $name _op>](array.[<get_ $name>](index)));
            }

            fn [<store_array_ $name>](context: &InterpreterContext, frame: &mut StackFrame) {
                Interpreter::array_primitive(context, frame,
                                             &format!("{}ASTORE", $instruction_prefix), $expected,
                                             |array_type| matches!(array_type, $array_type),
                                             |frame, array, index| array.[<put_ $name>](index, frame.[<pop_ $name _op>]()));
            }
        }
    };
}

impl Interpreter {
    pub fn execute(context: InterpreterContext, parameters: &[u32]) -> MethodResult {
        let mut frame = context.code.new_stack_frame();
        for parameter in parameters {
            frame.push_op(*parameter);
        }

        let mut parser = CodeParser::new(context.code.code());
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
                ALOAD => Interpreter::load_ref(&context, &mut frame, parser.next()),
                ILOAD_0..=ILOAD_3 => jvm_load_int(&mut frame, iload_index(op)),
                LLOAD_0..=LLOAD_3 => jvm_load_long(&mut frame, lload_index(op)),
                FLOAD_0..=FLOAD_3 => jvm_load_float(&mut frame, fload_index(op)),
                DLOAD_0..=DLOAD_3 => jvm_load_double(&mut frame, dload_index(op)),
                ALOAD_0..=ALOAD_3 => Interpreter::load_ref(&context, &mut frame, aload_index(op)),
                IALOAD => Interpreter::load_array_int(&context, &mut frame),
                LALOAD => Interpreter::load_array_long(&context, &mut frame),
                FALOAD => Interpreter::load_array_float(&context, &mut frame),
                DALOAD => Interpreter::load_array_double(&context, &mut frame),
                AALOAD => Interpreter::load_array_ref(&context, &mut frame),
                BALOAD => Interpreter::load_array_byte(&context, &mut frame),
                CALOAD => Interpreter::load_array_char(&context, &mut frame),
                SALOAD => Interpreter::load_array_short(&context, &mut frame),
                ISTORE => jvm_store_int(&mut frame, parser.next()),
                LSTORE => jvm_store_long(&mut frame, parser.next()),
                FSTORE => jvm_store_float(&mut frame, parser.next()),
                DSTORE => jvm_store_double(&mut frame, parser.next()),
                ASTORE => Interpreter::store_ref(&context, &mut frame, parser.next()),
                ISTORE_0..=ISTORE_3 => jvm_store_int(&mut frame, istore_index(op)),
                LSTORE_0..=LSTORE_3 => jvm_store_long(&mut frame, lstore_index(op)),
                FSTORE_0..=FSTORE_3 => jvm_store_float(&mut frame, fstore_index(op)),
                DSTORE_0..=DSTORE_3 => jvm_store_double(&mut frame, dstore_index(op)),
                ASTORE_0..=ASTORE_3 => Interpreter::store_ref(&context, &mut frame, astore_index(op)),
                IASTORE => Interpreter::store_array_int(&context, &mut frame),
                LASTORE => Interpreter::store_array_long(&context, &mut frame),
                FASTORE => Interpreter::store_array_float(&context, &mut frame),
                DASTORE => Interpreter::store_array_double(&context, &mut frame),
                AASTORE => Interpreter::store_array_ref(&context, &mut frame),
                BASTORE => Interpreter::store_array_byte(&context, &mut frame),
                CASTORE => Interpreter::store_array_char(&context, &mut frame),
                SASTORE => Interpreter::store_array_short(&context, &mut frame),
                POP => Interpreter::pop(&mut frame, false),
                POP2 => Interpreter::pop(&mut frame, true),
                DUP => Interpreter::dup(&mut frame),
                DUP_X1 => Interpreter::dup_x1(&mut frame),
                DUP_X2 => Interpreter::dup_x2(&mut frame),
                DUP2 => Interpreter::dup2(&mut frame),
                DUP2_X1 => Interpreter::dup2_x1(&mut frame),
                DUP2_X2 => Interpreter::dup2_x2(&mut frame),
                SWAP => Interpreter::swap(&mut frame),
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
                IFEQ | IFNE | IFLT | IFGE | IFGT | IFLE => Interpreter::branch(&mut frame, &mut parser, op),
                IF_ICMPEQ | IF_ICMPNE | IF_ICMPLT | IF_ICMPGE | IF_ICMPGT | IF_ICMPLE => {
                    Interpreter::int_branch(&mut frame, &mut parser, op)
                },
                IF_ACMPEQ | IF_ACMPNE => Interpreter::ref_branch(&context, &mut frame, &mut parser, op),
                GOTO => Interpreter::branch_seek(&mut parser),
                JSR => Interpreter::jump_subroutine(&mut frame, &mut parser, false),
                // TODO: RET, TABLESWITCH, LOOKUPSWITCH
                IRETURN => return MethodResult::Integer(frame.pop_int_op()),
                LRETURN => return MethodResult::Long(frame.pop_long_op()),
                FRETURN => return MethodResult::Float(frame.pop_float_op()),
                DRETURN => return MethodResult::Double(frame.pop_double_op()),
                ARETURN => return MethodResult::Reference(frame.pop_ref_op(context.heap)),
                RETURN => return MethodResult::Void,
                // TODO: GETSTATIC, PUTSTATIC, GETFIELD, PUTFIELD, INVOKEVIRTUAL, INVOKESPECIAL,
                //  INVOKESTATIC, INVOKEINTERFACE, INVOKEDYNAMIC
                NEW => Interpreter::new_ref(&context, &mut frame, &mut parser),
                NEWARRAY => Interpreter::new_type_array(&context, &mut frame, &mut parser),
                ANEWARRAY => Interpreter::new_ref_array(&context, &mut frame, &mut parser),
                ARRAYLENGTH => Interpreter::array_length(&context, &mut frame),
                ATHROW => {
                    if let Some(result) = Interpreter::throw(&context, &mut frame, &mut parser) {
                        return result;
                    }
                },
                CHECKCAST => Interpreter::check_cast(&context, &mut frame, &mut parser),
                INSTANCEOF => Interpreter::instanceof(&context, &mut frame, &mut parser),
                // TODO: MONITORENTER, MONITOREXIT, WIDE, MULTIANEWARRAY
                IFNULL => Interpreter::branch_null(&context, &mut frame, &mut parser, true),
                IFNONNULL => Interpreter::branch_null(&context, &mut frame, &mut parser, false),
                GOTO_W => Interpreter::branch_seek_wide(&mut parser),
                JSR_W => Interpreter::jump_subroutine(&mut frame, &mut parser, true),
                _ => panic!("Unrecognised bytecode {}!", op)
            }
        }
        panic!("Method should have returned by this point!");
    }

    fn load_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(Rc::clone(&context.heap))
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = array_ref.get(index as usize).expect("Invalid array index on operand stack!");
        frame.push_ref_op(value.offset() as u32);
    }

    fn store_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(Rc::clone(&context.heap))
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = frame.pop_ref_op(Rc::clone(&context.heap)).expect("Invalid array value on operand stack!");
        array_ref.set(index as usize, value);
    }

    fn load_ref(context: &InterpreterContext, frame: &mut StackFrame, index: u8) {
        let reference = frame.get_local_ref(index as usize, Rc::clone(&context.heap))
            .expect(&format!("Invalid reference index {}!", index));
        frame.push_ref_op(reference.offset() as u32);
    }

    fn array_length(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(Rc::clone(&context.heap))
            .expect("Invalid array reference on operand stack!");
        frame.push_int_op(array_ref.len() as i32);
    }

    fn store_ref(context: &InterpreterContext, frame: &mut StackFrame, index: u8) {
        let reference = frame.pop_ref_op(Rc::clone(&context.heap))
            .expect("Invalid reference on operand stack! Reference cannot be null!");
        frame.set_local_ref(index as usize, reference.offset() as u32);
    }

    fn throw(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser) -> Option<MethodResult> {
        let exception = frame.pop_ref_op(Rc::clone(&context.heap))
            .expect("Invalid exception on operand stack! Reference cannot be null!");
        let handler = context.code.exception_handlers().get_handler(exception.class());
        match handler {
            Some(value) => {
                parser.seek(value.start_pc() as usize);
                None
            },
            None => Some(MethodResult::Exception)
        }
    }

    fn load_array_byte(context: &InterpreterContext, frame: &mut StackFrame) {
        Interpreter::common_array_primitive(context, frame, |frame, array, array_type, index| {
            match array_type {
                ArrayType::Byte => frame.push_byte_op(array.get_byte(index)),
                ArrayType::Boolean => frame.push_bool_op(array.get_bool(index)),
                _ => panic!("Invalid type of array for BASTORE! Expected array to be of type \
                    byte or boolean, was {}!", array_type)
            }
        })
    }

    fn store_array_byte(context: &InterpreterContext, frame: &mut StackFrame) {
        Interpreter::common_array_primitive(context, frame, |frame, array, array_type, index| {
            match array_type {
                ArrayType::Byte => array.put_byte(index, frame.pop_byte_op()),
                ArrayType::Boolean => array.put_bool(index, frame.pop_bool_op()),
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

    fn check_cast(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser) {
        let reference = frame.pop_ref_op(Rc::clone(&context.heap));
        if matches!(reference, Reference::Null) {
            return;
        }
        let reference = reference.unwrap();
        let class_index = ((parser.next() as u16) << 8) | (parser.next() as u16);
        let class_name = context.class.constant_pool()
            .resolve_class_name(class_index as usize)
            .expect(&format!("Invalid cast check! Expected index {} to be in constant pool!", class_index));
        let class = context.loader.load_class(class_name);
        assert!(reference.class().is_subclass(Rc::clone(&class)), "Cannot cast {} to {}!",
                reference.class().name(), Rc::clone(&class).name());
        frame.push_ref_op(reference.offset() as u32);
    }

    fn common_array_primitive<F>(
        context: &InterpreterContext,
        frame: &mut StackFrame,
        mapper: F
    ) where F: Fn(&mut StackFrame, Rc<TypeArrayObject>, ArrayType, usize) {
        let array_ref = frame.pop_type_array_op(Rc::clone(&context.heap))
            .expect("Invalid array reference on operand stack! Reference cannot be null!");
        let array_type = array_ref.array_type();
        let index = frame.pop_int_op() as usize;
        mapper(frame, array_ref, array_type, index)
    }

    fn array_primitive<C, F>(
        context: &InterpreterContext,
        frame: &mut StackFrame,
        instruction: &str,
        expected_type: &str,
        checker: C,
        mapper: F
    ) where C: Fn(ArrayType) -> bool, F: Fn(&mut StackFrame, Rc<TypeArrayObject>, usize) {
        Interpreter::common_array_primitive(context, frame, |frame, array, array_type, index| {
            if checker(array_type) {
                mapper(frame, array, index)
            }
            panic!("Invalid type of array for {}! Expected array to be of type {}, was {}!",
                instruction, expected_type, array_type)
        })
    }

    fn pop(frame: &mut StackFrame, double: bool) {
        frame.pop_op();
        if double {
            frame.pop_op();
        }
    }

    fn dup(frame: &mut StackFrame) {
        let value = frame.get_op(0);
        frame.push_op(value);
    }

    fn dup_x1(frame: &mut StackFrame) {
        let first = frame.get_op(0);
        let second = frame.get_op(1);
        frame.set_op(1, first);
        frame.set_op(0, second);
        frame.push_op(first);
    }

    fn dup_x2(frame: &mut StackFrame) {
        let first = frame.get_op(0);
        let third = frame.get_op(2);
        frame.set_op(2, first);
        frame.push_op(first);
        let second = frame.get_op(2);
        frame.set_op(2, third);
        frame.set_op(1, second);
    }

    fn dup2(frame: &mut StackFrame) {
        let high = frame.get_op(1);
        frame.push_op(high);
        let low = frame.get_op(1);
        frame.push_op(low);
    }

    fn dup2_x1(frame: &mut StackFrame) {
        let first = frame.get_op(0);
        let second = frame.get_op(1);
        frame.push_op(second);
        frame.push_op(first);
        frame.set_op(3, first);
        let third = frame.get_op(4);
        frame.set_op(2, third);
        frame.set_op(4, second);
    }

    fn dup2_x2(frame: &mut StackFrame) {
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

    fn swap(frame: &mut StackFrame) {
        let first = frame.get_op(1);
        let second = frame.get_op(0);
        frame.set_op(0, first);
        frame.set_op(1, second);
    }

    fn branch<'a>(frame: &mut StackFrame, parser: &mut CodeParser<'a>, op: u8) {
        let value = frame.pop_int_op();
        let success = (op == IFEQ && value == 0) ||
            (op == IFNE && value != 0) ||
            (op == IFLT && value < 0) ||
            (op == IFLE && value <= 0) ||
            (op == IFGT && value > 0) ||
            (op == IFGE && value >= 0);
        if success {
            Interpreter::branch_seek(parser);
        }
    }

    fn branch_null<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>, null: bool) {
        match frame.pop_ref_op(Rc::clone(&context.heap)) {
            Reference::Value(_) if !null => Interpreter::branch_seek(parser),
            Reference::Null if null => Interpreter::branch_seek(parser),
            _ => {}
        }
    }

    fn instanceof<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser) {
        let reference = frame.pop_ref_op(Rc::clone(&context.heap));
        let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
        if let Reference::Null = reference {
            frame.push_int_op(0);
            return;
        }
        let reference = reference.unwrap();
        let class_name = context.class.constant_pool().resolve_class_name(index as usize)
            .expect(&format!("Invalid class for instanceof check! Expected index {} to be in constant pool!", index));
        let class = context.loader.load_class(class_name);
        let result = if reference.class().is_subclass(class) { 1 } else { 0 };
        frame.push_int_op(result);
    }

    fn ref_branch<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>, op: u8) {
        let first_ref = frame.pop_ref_op(Rc::clone(&context.heap));
        let second_ref = frame.pop_ref_op(Rc::clone(&context.heap));
        let ref_compare = first_ref.equals(second_ref);
        if (op == IF_ACMPEQ && ref_compare) || (op == IF_ACMPNE && !ref_compare) {
            Interpreter::branch_seek(parser);
        }
    }

    fn int_branch<'a>(frame: &mut StackFrame, parser: &mut CodeParser<'a>, op: u8) {
        let first = frame.pop_int_op();
        let second = frame.pop_int_op();
        let success = (op  == IF_ICMPEQ && first == second) ||
            (op == IF_ICMPNE && first != second) ||
            (op == IF_ICMPLT && first < second) ||
            (op == IF_ICMPLE && first <= second) ||
            (op == IF_ICMPGT && first > second) ||
            (op == IF_ICMPGE && first >= second);
        if success {
            Interpreter::branch_seek(parser);
        }
    }

    fn jump_subroutine<'a>(frame: &mut StackFrame, parser: &mut CodeParser<'a>, wide: bool) {
        frame.push_op(parser.next_index());
        if wide {
            Interpreter::branch_seek_wide(parser);
        } else {
            Interpreter::branch_seek(parser);
        }
    }

    fn branch_seek(parser: &mut CodeParser) {
        let index = ((parser.next() as i16) << 8) | (parser.next() as i16);
        parser.seek_relative(index as usize);
    }

    fn branch_seek_wide(parser: &mut CodeParser) {
        let index = ((parser.next() as i32) << 24) |
            ((parser.next() as i32) << 16) |
            ((parser.next() as i32) << 8) |
            (parser.next() as i32);
        parser.seek_relative(index as usize);
    }

    fn new_ref<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>) {
        let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
        let class_name = context.class.constant_pool().resolve_class_name(index as usize)
            .expect(&format!("Invalid object instantiation! Expected index {} to be in constant pool!", index));
        let class = context.loader.load_class(class_name);
        if class.is_interface() || class.is_abstract() {

        }
        let offset = context.heap.offset();
        let instance = InstanceObject::new(offset, class, 0);
        context.heap.push_ref(Rc::new(instance));
        frame.push_ref_op(offset as u32);
    }

    fn new_ref_array<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>) {
        let count = frame.pop_int_op();
        let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
        let class_type = context.class.constant_pool().resolve_class_name(index as usize)
            .expect(&format!("Invalid class type index {}!", index));
        let class = context.loader.load_class(class_type);
        let offset = context.heap.offset();
        let array = ReferenceArrayObject::new(offset, Rc::clone(&context.class), class, count as usize);
        context.heap.push_ref_array(Rc::new(array));
        frame.push_ref_op(offset as u32);
    }

    fn new_type_array<'a>(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>) {
        let array_type = ArrayType::from(parser.next());
        let count = frame.pop_int_op();
        let offset = context.heap.offset();
        let array = TypeArrayObject::new(offset, array_type, count as usize);
        context.heap.push_type_array(Rc::new(array));
        frame.push_ref_op(offset as u32);
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

pub struct InterpreterContext {
    pub heap: Rc<HeapSpace>,
    pub loader: Rc<ClassLoader>,
    pub class: Rc<Class>,
    pub code: Rc<CodeBlock>
}

pub enum MethodResult {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(Reference<Rc<InstanceObject>>),
    Void,
    Exception
}

const NOP: u8 = 0;
const ACONST_NULL: u8 = 1;
const ICONST_M1: u8 = 2;
const ICONST_0: u8 = 3;
const ICONST_5: u8 = 8;
const LCONST_0: u8 = 9;
const LCONST_1: u8 = 10;
const FCONST_0: u8 = 11;
const FCONST_2: u8 = 13;
const DCONST_0: u8 = 14;
const DCONST_1: u8 = 15;
const BIPUSH: u8 = 16;
const SIPUSH: u8 = 17;
//const LDC: u8 = 18;
//const LDC_W: u8 = 19;
//const LDC2_W: u8 = 20;
const ILOAD: u8 = 21;
const LLOAD: u8 = 22;
const FLOAD: u8 = 23;
const DLOAD: u8 = 24;
const ALOAD: u8 = 25;
const ILOAD_0: u8 = 26;
const ILOAD_3: u8 = 29;
const LLOAD_0: u8 = 30;
const LLOAD_3: u8 = 33;
const FLOAD_0: u8 = 34;
const FLOAD_3: u8 = 37;
const DLOAD_0: u8 = 38;
const DLOAD_3: u8 = 41;
const ALOAD_0: u8 = 42;
const ALOAD_3: u8 = 45;
const IALOAD: u8 = 46;
const LALOAD: u8 = 47;
const FALOAD: u8 = 48;
const DALOAD: u8 = 49;
const AALOAD: u8 = 50;
const BALOAD: u8 = 51;
const CALOAD: u8 = 52;
const SALOAD: u8 = 53;
const ISTORE: u8 = 54;
const LSTORE: u8 = 55;
const FSTORE: u8 = 56;
const DSTORE: u8 = 57;
const ASTORE: u8 = 58;
const ISTORE_0: u8 = 59;
const ISTORE_3: u8 = 62;
const LSTORE_0: u8 = 63;
const LSTORE_3: u8 = 66;
const FSTORE_0: u8 = 67;
const FSTORE_3: u8 = 70;
const DSTORE_0: u8 = 71;
const DSTORE_3: u8 = 74;
const ASTORE_0: u8 = 75;
const ASTORE_3: u8 = 78;
const IASTORE: u8 = 79;
const LASTORE: u8 = 80;
const FASTORE: u8 = 81;
const DASTORE: u8 = 82;
const AASTORE: u8 = 83;
const BASTORE: u8 = 84;
const CASTORE: u8 = 85;
const SASTORE: u8 = 86;
const POP: u8 = 87;
const POP2: u8 = 88;
const DUP: u8 = 89;
const DUP_X1: u8 = 90;
const DUP_X2: u8 = 91;
const DUP2: u8 = 92;
const DUP2_X1: u8 = 93;
const DUP2_X2: u8 = 94;
const SWAP: u8 = 95;
const IADD: u8 = 96;
const LADD: u8 = 97;
const FADD: u8 = 98;
const DADD: u8 = 99;
const ISUB: u8 = 100;
const LSUB: u8 = 101;
const FSUB: u8 = 102;
const DSUB: u8 = 103;
const IMUL: u8 = 104;
const LMUL: u8 = 105;
const FMUL: u8 = 106;
const DMUL: u8 = 107;
const IDIV: u8 = 108;
const LDIV: u8 = 109;
const FDIV: u8 = 110;
const DDIV: u8 = 111;
const IREM: u8 = 112;
const LREM: u8 = 113;
const FREM: u8 = 114;
const DREM: u8 = 115;
const INEG: u8 = 116;
const LNEG: u8 = 117;
const FNEG: u8 = 118;
const DNEG: u8 = 119;
const ISHL: u8 = 120;
const LSHL: u8 = 121;
const ISHR: u8 = 122;
const LSHR: u8 = 123;
const IUSHR: u8 = 124;
const LUSHR: u8 = 125;
const IAND: u8 = 126;
const LAND: u8 = 127;
const IOR: u8 = 128;
const LOR: u8 = 129;
const IXOR: u8 = 130;
const LXOR: u8 = 131;
const IINC: u8 = 132;
const I2L: u8 = 133;
const I2F: u8 = 134;
const I2D: u8 = 135;
const L2I: u8 = 136;
const L2F: u8 = 137;
const L2D: u8 = 138;
const F2I: u8 = 139;
const F2L: u8 = 140;
const F2D: u8 = 141;
const D2I: u8 = 142;
const D2L: u8 = 143;
const D2F: u8 = 144;
const I2B: u8 = 145;
const I2C: u8 = 146;
const I2S: u8 = 147;
const LCMP: u8 = 148;
const FCMPL: u8 = 149;
const FCMPG: u8 = 150;
const DCMPL: u8 = 151;
const DCMPG: u8 = 152;
const IFEQ: u8 = 153;
const IFNE: u8 = 154;
const IFLT: u8 = 155;
const IFGE: u8 = 156;
const IFGT: u8 = 157;
const IFLE: u8 = 158;
const IF_ICMPEQ: u8 = 159;
const IF_ICMPNE: u8 = 160;
const IF_ICMPLT: u8 = 161;
const IF_ICMPGE: u8 = 162;
const IF_ICMPGT: u8 = 163;
const IF_ICMPLE: u8 = 164;
const IF_ACMPEQ: u8 = 165;
const IF_ACMPNE: u8 = 166;
const GOTO: u8 = 167;
const JSR: u8 = 168;
//const RET: u8 = 169;
//const TABLESWITCH: u8 = 170;
//const LOOKUPSWITCH: u8 = 171;
const IRETURN: u8 = 172;
const LRETURN: u8 = 173;
const FRETURN: u8 = 174;
const DRETURN: u8 = 175;
const ARETURN: u8 = 176;
const RETURN: u8 = 177;
//const GETSTATIC: u8 = 178;
//const PUTSTATIC: u8 = 179;
//const GETFIELD: u8 = 180;
//const PUTFIELD: u8 = 181;
//const INVOKEVIRTUAL: u8 = 182;
//const INVOKESPECIAL: u8 = 183;
//const INVOKESTATIC: u8 = 184;
//const INVOKEINTERFACE: u8 = 185;
//const INVOKEDYNAMIC: u8 = 186;
const NEW: u8 = 187;
const NEWARRAY: u8 = 188;
const ANEWARRAY: u8 = 189;
const ARRAYLENGTH: u8 = 190;
const ATHROW: u8 = 191;
const CHECKCAST: u8 = 192;
const INSTANCEOF: u8 = 193;
//const MONITORENTER: u8 = 194;
//const MONITOREXIT: u8 = 195;
//const WIDE: u8 = 196;
//const MULTIANEWARRAY: u8 = 197;
const IFNULL: u8 = 198;
const IFNONNULL: u8 = 199;
const GOTO_W: u8 = 200;
const JSR_W: u8 = 201;
//const BREAKPOINT: u8 = 202; // Debug only

macro_rules! generate_load_store_index {
    ($name:ident, $prefix:ident) => {
        paste! {
            fn [<$name _index>](op: u8) -> u8 {
                if op < [<$prefix _0>] || op > [<$prefix _3>] {
                    panic!("{} called with op < {} or > {}! Op was {}!", "[<$name _index>]", "[<$prefix _0>]", "[<$prefix _3>]", op);
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
