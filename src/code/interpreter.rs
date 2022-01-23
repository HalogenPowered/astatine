use crate::class_file::class_loader::ClassLoader;
use crate::class_file::code::CodeBlock;
use crate::objects::heap::HeapSpace;
use crate::objects::object::{HeapObject, InstanceObject, ReferenceArrayObject, TypeArrayObject};
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

use super::stack_frame::StackFrame;

pub struct Interpreter {
    _singleton: ()
}

macro_rules! load_array_primitive {
    ($name:ident, $instruction:literal, $expected:literal, $array_type:pat, $push_name:ident, $get_name:ident) => {
        fn $name(context: &InterpreterContext, frame: &mut StackFrame) {
            Interpreter::array_primitive(context, frame, $instruction, $expected,
                                         |array_type| matches!(array_type, $array_type),
                                         |array, index| frame.$push_name(array.$get_name(index)));
        }
    };
}

macro_rules! store_array_primitive {
    ($name:ident, $instruction:literal, $expected:literal, $array_type:pat, $pop_name:ident, $put_name:ident) => {
        fn $name(context: &InterpreterContext, frame: &mut StackFrame) {
            Interpreter::array_primitive(context, frame, $instruction, $expected,
                                         |array_type| matches!(array_type, $array_type),
                                         |array, index| array.$put_name(index, frame.$pop_name()));
        }
    };
}

impl Interpreter {
    pub fn execute(context: &mut InterpreterContext, code: &CodeBlock, parameters: &[u32]) -> MethodResult {
        let mut frame = code.new_stack_frame();
        for parameter in parameters {
            frame.push_op(*parameter);
        }

        let mut parser = CodeParser::new(code.code());
        while !parser.is_empty() {
            let op = parser.next();
            match op {
                AALOAD => Interpreter::load_array_ref(context, &mut frame),
                AASTORE => Interpreter::store_array_ref(context, &mut frame),
                ACONST_NULL => frame.push_null_op(),
                ALOAD => Interpreter::load_ref(context, &mut frame, parser.next()),
                ALOAD_0..=ALOAD_3 => Interpreter::load_ref(context, &mut frame, aload_index(op)),
                ANEWARRAY => Interpreter::new_array(context, &mut frame, &mut parser),
                ARRAYLENGTH => Interpreter::array_length(context, &mut frame),
                ASTORE => Interpreter::store_ref(context, &mut frame, parser.next()),
                ASTORE_0..=ASTORE_3 => Interpreter::store_ref(context, &mut frame, astore_index(op)),
                ATHROW => {
                    if let Some(result) = Interpreter::throw(context, &mut frame, &mut parser) {
                        return result;
                    }
                },
                BALOAD => Interpreter::load_array_byte(context, &mut frame),
                BASTORE => Interpreter::store_array_byte(context, &mut frame),
                BIPUSH => frame.push_byte_op(parser.next() as i8),
                CALOAD => Interpreter::load_array_char(context, &mut frame),
                CASTORE => Interpreter::store_array_char(context, &mut frame),
                DALOAD => Interpreter::load_array_double(context, &mut frame),
                DASTORE => Interpreter::store_array_double(context, &mut frame),
                FALOAD => Interpreter::load_array_float(context, &mut frame),
                FASTORE => Interpreter::store_array_float(context, &mut frame),
                IALOAD => Interpreter::load_array_int(context, &mut frame),
                IASTORE => Interpreter::store_array_int(context, &mut frame),
                LALOAD => Interpreter::load_array_long(context, &mut frame),
                LASTORE => Interpreter::store_array_long(context, &mut frame),
                SALOAD => Interpreter::load_array_short(context, &mut frame),
                SASTORE => Interpreter::store_array_short(context, &mut frame),
                _ => panic!("Unrecognised bytecode {}!", op)
            }
        }
        panic!("Method should have returned by this point!");
    }

    fn load_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = array_ref.get(index as usize).expect("Invalid array index on operand stack!");
        frame.push_ref_op(value.offset() as u32);
    }

    fn store_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = frame.pop_ref_op(&context.heap).expect("Invalid array value on operand stack!");
        array_ref.set(index as usize, value);
    }

    fn load_ref(context: &InterpreterContext, frame: &mut StackFrame, index: u8) {
        let reference = frame.get_local_ref(index as usize, &context.heap)
            .expect(&format!("Invalid reference index {}!", index));
        frame.push_ref_op(reference.offset() as u32);
    }

    fn new_array<'a>(context: &mut InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser<'a>) {
        let count = frame.pop_int_op();
        let index = ((parser.next() as u16) << 8) | (parser.next() as u16);
        let class_type = context.class.constant_pool().resolve_class_name(index as usize)
            .expect(&format!("Invalid class type index {}!", index));
        let class = context.loader.load_class(class_type);
        let offset = context.heap.get_offset();
        let array = ReferenceArrayObject::new(offset, context.class, class, count as usize);
        context.heap.push_ref_array(Box::new(array));
        frame.push_ref_op(offset as u32);
    }

    fn array_length(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
        frame.push_int_op(array_ref.len() as i32);
    }

    fn store_ref(context: &InterpreterContext, frame: &mut StackFrame, index: u8) {
        let reference = frame.pop_ref_op(&context.heap)
            .expect("Invalid reference on operand stack! Reference cannot be null!");
        frame.set_local_ref(index as usize, reference.offset() as u32);
    }

    fn throw(context: &InterpreterContext, frame: &mut StackFrame, parser: &mut CodeParser) -> Option<MethodResult> {
        let exception = frame.pop_ref_op(&context.heap)
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
        Interpreter::common_array_primitive(context, frame, |array, array_type, index| {
            match array_type {
                ArrayType::Byte => frame.push_byte_op(array.get_byte(index)),
                ArrayType::Boolean => frame.push_bool_op(array.get_bool(index)),
                _ => panic!("Invalid type of array for BASTORE! Expected array to be of type \
                    byte or boolean, was {}!", array_type)
            }
        })
    }

    fn store_array_byte(context: &InterpreterContext, frame: &mut StackFrame) {
        Interpreter::common_array_primitive(context, frame, |array, array_type, index| {
            match array_type {
                ArrayType::Byte => array.put_byte(index, frame.pop_byte_op()),
                ArrayType::Boolean => array.put_bool(index, frame.pop_bool_op()),
                _ => panic!("Invalid type of array for BASTORE! Expected array to be of type \
                    byte or boolean, was {}!", array_type)
            }
        });
    }

    load_array_primitive!(load_array_char, "CALOAD", "char", ArrayType::Char, push_char_op, get_char);
    store_array_primitive!(store_array_char, "CASTORE", "char", ArrayType::Char, pop_char_op, put_char);
    load_array_primitive!(load_array_double, "DALOAD", "double", ArrayType::Double, push_double_op, get_double);
    store_array_primitive!(store_array_double, "DASTORE", "double", ArrayType::Double, pop_double_op, put_double);
    load_array_primitive!(load_array_float, "FALOAD", "float", ArrayType::Float, push_float_op, get_float);
    store_array_primitive!(store_array_float, "FASTORE", "float", ArrayType::Float, pop_float_op, put_float);
    load_array_primitive!(load_array_int, "IALOAD", "int", ArrayType::Int, push_int_op, get_int);
    store_array_primitive!(store_array_int, "IASTORE", "int", ArrayType::Int, pop_int_op, put_int);
    load_array_primitive!(load_array_long, "LALOAD", "long", ArrayType::Long, push_long_op, get_long);
    store_array_primitive!(store_array_long, "LASTORE", "long", ArrayType::Long, pop_long_op, put_long);
    load_array_primitive!(load_array_short, "SALOAD", "short", ArrayType::Short, push_short_op, get_short);
    store_array_primitive!(store_array_short, "SASTORE", "short", ArrayType::Short, pop_short_op, put_short);

    fn common_array_primitive<F>(
        context: &InterpreterContext,
        frame: &mut StackFrame,
        mapper: F
    ) where F: Fn(&Box<TypeArrayObject>, ArrayType, usize) {
        let array_ref = frame.pop_type_array_op(&context.heap)
            .expect("Invalid array reference on operand stack! Reference cannot be null!");
        let array_type = array_ref.array_type();
        let index = frame.pop_int_op() as usize;
        mapper(array_ref, array_type, index)
    }

    fn array_primitive<C, F>(
        context: &InterpreterContext,
        frame: &mut StackFrame,
        instruction: &str,
        expected_type: &str,
        checker: C,
        mapper: F
    ) where C: Fn(ArrayType) -> bool, F: Fn(&Box<TypeArrayObject>, usize) {
        Interpreter::common_array_primitive(context, frame, |array, array_type, index| {
            if checker(array_type) {
                mapper(array, index)
            }
            panic!("Invalid type of array for {}! Expected array to be of type {}, was {}!",
                instruction, expected_type, array_type)
        });
    }
}

struct CodeParser<'a> {
    bytes: &'a [u8],
    index: usize
}

impl<'a> CodeParser<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        CodeParser { bytes, index: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn next(&mut self) -> u8 {
        let next = self.bytes[self.index];
        self.index += 1;
        next
    }

    pub fn seek(&mut self, index: usize) {
        self.index = index;
    }
}

pub struct InterpreterContext<'a> {
    pub heap: Box<HeapSpace>,
    pub loader: Box<ClassLoader>,
    pub class: &'a Class,
    pub code: &'a CodeBlock
}

pub enum MethodResult {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(Box<InstanceObject>),
    Exception
}

const AALOAD: u8 = 0x32;
const AASTORE: u8 = 0x53;
const ACONST_NULL: u8 = 0x01;
const ALOAD: u8 = 0x19;
const ALOAD_0: u8 = 0x2A;
const ALOAD_3: u8 = 0x2D;
const ANEWARRAY: u8 = 0xBD;
const ARETURN: u8 = 0xB0;
const ARRAYLENGTH: u8 = 0xBE;
const ASTORE: u8 = 0x3A;
const ASTORE_0: u8 = 0x4B;
const ASTORE_3: u8 = 0x4E;
const ATHROW: u8 = 0xBF;
const BALOAD: u8 = 0x33;
const BASTORE: u8 = 0x54;
const BIPUSH: u8 = 0x10;
const CALOAD: u8 = 0x34;
const CASTORE: u8 = 0x55;
const DALOAD: u8  = 0x31;
const DASTORE: u8 = 0x52;
const FALOAD: u8 = 0x30;
const FASTORE: u8 = 0x51;
const IALOAD: u8 = 0x2E;
const IASTORE: u8 = 0x4F;
const LALOAD: u8 = 0x2F;
const LASTORE: u8 = 0x7F;
const SALOAD: u8 = 0x35;
const SASTORE: u8 = 0x56;

fn aload_index(op: u8) -> u8 {
    if op > ALOAD_3 {
        panic!("aload_index called with op higher than ALOAD_3! Op was {}!", op);
    }
    op - ALOAD_0
}

fn astore_index(op: u8) -> u8 {
    if op > ASTORE_3 {
        panic!("astore_index called with op higher than ASTORE_3! Op was {}!", op);
    }
    op - ASTORE_0
}
