use bytes::{Buf, Bytes};
use java_desc::FieldType;
use crate::class_file::attribute_tags::TAG_SIGNATURE;
use crate::class_file::utils::parse_generic_signature;
use crate::types::constant_pool::ConstantPool;

pub struct RecordComponent<'a> {
    pub name: &'a str,
    pub descriptor: FieldType,
    pub generic_signature: Option<&'a str>
}

impl<'a> RecordComponent<'a> {
    pub fn parse(class_file_name: &str, pool: &'a ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid record component for class file {}! Expected name at index {} \
                in constant pool!", class_file_name, name_index));
        let descriptor_index = buf.get_u16();
        let descriptor_string = pool.get_string(descriptor_index as usize)
            .expect(&format!("Invalid record component for class file {}! Expected descriptor at \
                index {} in constant pool!", class_file_name, name_index));
        let descriptor = FieldType::parse(descriptor_string)
            .expect(&format!("Invalid descriptor {} for record component in class file {}!", descriptor_string, class_file_name));

        let attribute_count = buf.get_u16();
        let mut generic_signature = None;
        parse_attributes(class_file_name, pool, attribute_count, buf, &mut generic_signature);

        RecordComponent { name, descriptor, generic_signature }
    }
}

fn parse_attributes<'a>(
    class_file_name: &str,
    pool: &'a ConstantPool,
    mut attribute_count: u16,
    buf: &mut Bytes,
    generic_signature: &mut Option<&'a str>
) {
    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated record component attributes for field in class file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid record component attribute index {} in class file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_SIGNATURE {
            parse_generic_signature(class_file_name, pool, attribute_length, buf, "record component", generic_signature);
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
}
