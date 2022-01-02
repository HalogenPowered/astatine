use crate::attributes::attributes::Attribute;
use crate::attributes::stack_map_frames::StackMapFrame;

pub struct StackMapTableAttribute {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub entries: Vec<StackMapFrame>
}

impl Attribute for StackMapTableAttribute {
    fn read_from_internal(attribute_name_index: u16, attribute_length: u32, mut buf: &Bytes) -> Self {
        let number_of_entries = buf.get_u16();
        let mut entries = Vec::with_capacity(number_of_entries as usize);
        for _ in 0..number_of_entries {
            entries.push(StackMapFrame::read_from(buf));
        }
        StackMapTableAttribute { attribute_name_index, attribute_length, entries }
    }
}
