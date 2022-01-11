use bytes::{Buf, Bytes};

pub fn parse_stack_map_table(buf: &mut Bytes) -> StackMapTable {
    let entry_count = buf.get_u16();
    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        entries.push(StackMapFrame::parse(buf));
    }
    entries
}

pub type StackMapTable = Vec<StackMapFrame>;

pub struct StackMapFrame {
    pub frame_type: u8,
    pub offset_delta: u16,
    stack: Option<Vec<VerificationType>>,
    locals: Option<Vec<VerificationType>>
}

impl StackMapFrame {
    pub fn parse(buf: &mut Bytes) -> Self {
        let frame_type = buf.get_u8();
        let offset_delta;
        let mut stack = None;
        let mut locals = None;

        match frame_type {
            0..=63 => offset_delta = frame_type as u16,
            64..=127 => {
                offset_delta = (frame_type - 64) as u16;
                stack = Some(StackMapFrame::parse_types(1, buf));
            }
            247 => {
                offset_delta = buf.get_u16();
                stack = Some(StackMapFrame::parse_types(1, buf));
            }
            248..=250 => offset_delta = buf.get_u16(),
            251 => offset_delta = buf.get_u16(),
            252..=254 => {
                offset_delta = buf.get_u16();
                locals = Some(StackMapFrame::parse_types((frame_type - 251) as usize, buf));
            }
            255 => {
                offset_delta = buf.get_u16();
                locals = Some(StackMapFrame::parse_types(buf.get_u16() as usize, buf));
                stack = Some(StackMapFrame::parse_types(buf.get_u16() as usize, buf));
            }
            _ => panic!("Invalid stack map frame type {}!", frame_type)
        }
        StackMapFrame { frame_type, offset_delta, stack, locals }
    }

    fn parse_types(count: usize, buf: &mut Bytes) -> Vec<VerificationType> {
        let mut types = Vec::with_capacity(count);
        for _ in 0..count {
            types.push(VerificationType::parse(buf));
        }
        types
    }

    pub fn get_stack_type(&self, index: usize) -> Option<&VerificationType> {
        self.stack.as_ref().and_then(|stack| stack.get(index))
    }

    pub fn get_local_type(&self, index: usize) -> Option<&VerificationType> {
        self.locals.as_ref().and_then(|stack| stack.get(index))
    }
}

const ITEM_TOP: u8 = 0;
const ITEM_INTEGER: u8 = 1;
const ITEM_FLOAT: u8 = 2;
const ITEM_DOUBLE: u8 = 3;
const ITEM_LONG: u8 = 4;
const ITEM_NULL: u8 = 5;
const ITEM_UNINITIALIZED_THIS: u8 = 6;
const ITEM_OBJECT: u8 = 7;
const ITEM_UNINITIALIZED: u8 = 8;

pub enum VerificationType {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,
    UninitializedThis,
    Object { constant_pool_index: u16 },
    Uninitialized { offset: u16 }
}

impl VerificationType {
    pub fn parse(buf: &mut Bytes) -> Self {
        let tag = buf.get_u8();
        match tag {
            ITEM_TOP => VerificationType::Top,
            ITEM_INTEGER => VerificationType::Integer,
            ITEM_FLOAT => VerificationType::Float,
            ITEM_DOUBLE => VerificationType::Double,
            ITEM_LONG => VerificationType::Long,
            ITEM_NULL => VerificationType::Null,
            ITEM_UNINITIALIZED_THIS => VerificationType::UninitializedThis,
            ITEM_OBJECT => VerificationType::Object { constant_pool_index: buf.get_u16() },
            ITEM_UNINITIALIZED => VerificationType::Uninitialized { offset: buf.get_u16() },
            _ => panic!("Could not parse verification type with tag {}!", tag)
        }
    }
}
