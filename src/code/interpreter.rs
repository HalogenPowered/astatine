use crate::class_file::code::CodeBlock;
use crate::code::stack_frame::StackFrame;

pub struct Interpreter {
    _singleton: ()
}

impl Interpreter {
    pub fn execute(code: &CodeBlock, parameters: &[u32]) {
        let mut frame = StackFrame::new(code.max_stack, code.max_locals);
        for parameter in parameters {
            frame.push_op(*parameter);
        }
        for op in code.code.iter() {
            match *op {
                ACONST_NULL => frame.push_null_op(),
                _ => panic!("Unrecognised bytecode {}!", op)
            }
        }
    }
}

const AALOAD: u8 = 0x32;
const AASTORE: u8 = 0x53;
const ACONST_NULL: u8 = 0x01;
const ALOAD: u8 = 0x19;
const ALOAD_0: u8 = 0x2A;
const ALOAD_1: u8 = 0x2B;
