use bytes::{Buf, Bytes};
use crate::class_file::class_loader::ClassLoader;
use crate::class_file::code::CodeBlock;
use crate::code::stack_frame::StackFrame;
use crate::objects::heap::HeapSpace;
use crate::objects::object::HeapObject;
use crate::types::class::Class;

pub struct Interpreter {
    _singleton: ()
}

impl Interpreter {
    pub fn execute(context: InterpreterContext, code: &CodeBlock, parameters: &[u32]) {
        let mut frame = StackFrame::new(code.max_stack, code.max_locals);
        for parameter in parameters {
            frame.push_op(*parameter);
        }

        let mut bytes = Bytes::copy_from_slice(code.code.as_slice());
        while !bytes.is_empty() {
            let op = bytes.get_u8();
            match op {
                AALOAD => Interpreter::load_array_ref(&context, &mut frame),
                //AASTORE => Interpreter::store_array_ref(&context, &mut frame),
                ACONST_NULL => frame.push_null_op(),
                ALOAD => Interpreter::load_ref(&context, &mut frame, bytes.get_u8()),
                ALOAD_0..=ALOAD_3 => Interpreter::load_ref(&context, &mut frame, aload_index(op)),
                //ANEWARRAY => Interpreter::new_array(&mut context, &mut frame, &mut bytes),
                _ => panic!("Unrecognised bytecode {}!", op)
            }
        }
    }

    fn load_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = array_ref.get(index as usize).expect("Invalid array index on operand stack!");
        frame.push_ref_op(value.offset() as u32);
    }

    /*
    fn store_array_ref(context: &InterpreterContext, frame: &mut StackFrame) {
        let mut array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
        let index = frame.pop_int_op();
        let value = frame.pop_ref_op(&context.heap).expect("Invalid array value on operand stack!");
        array_ref.set(index as usize, value)
    }
    */

    fn load_ref(context: &InterpreterContext, frame: &mut StackFrame, index: u8) {
        let reference = frame.get_local_ref(index as usize, &context.heap)
            .expect(&format!("Invalid reference index {}!", index));
        frame.push_ref_op(reference.offset() as u32);
    }

    /*
    fn new_array(context: &mut InterpreterContext, frame: &mut StackFrame, buf: &mut Bytes) {
        let count = frame.pop_int_op();
        let index = ((buf.get_u8() as u16) << 8) | (buf.get_u8() as u16);
        let class_type = context.class.constant_pool.resolve_class_name(index as usize)
            .expect(&format!("Invalid class type index {}!", index));
        let class = context.loader.load_class(class_type);
        let offset = context.heap.get_offset();
        let array = ReferenceArrayObject::new(offset, context.class, class, count as usize);
        context.heap.push_ref_array(Box::new(array));
        frame.push_ref_op(offset as u32);
    }
    */

    /*
    fn array_length(context: &InterpreterContext, frame: &mut StackFrame) {
        let array_ref = frame.pop_ref_array_op(&context.heap)
            .expect("Invalid array reference on operand stack!");
    }
    */
}

pub struct InterpreterContext {
    pub heap: Box<HeapSpace>,
    pub loader: Box<ClassLoader>,
    pub class: Box<Class>,
}

const AALOAD: u8 = 0x32;
//const AASTORE: u8 = 0x53;
const ACONST_NULL: u8 = 0x01;
const ALOAD: u8 = 0x19;
const ALOAD_0: u8 = 0x2A;
const ALOAD_3: u8 = 0x2D;
//const ANEWARRAY: u8 = 0xBD;

fn aload_index(op: u8) -> u8 {
    op - ALOAD_0
}
