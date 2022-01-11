use bytes::{Buf, Bytes};
use java_desc::FieldType;

use crate::class_file::attribute_tags::TAG_SIGNATURE;
use crate::class_file::utils::parse_generic_signature;
use crate::types::constant_pool::ConstantPool;
use crate::types::utils::{Generic, Nameable};

#[derive(Debug)]
pub struct RecordComponent {
    name: String,
    pub descriptor: FieldType,
    generic_signature: Option<String>
}

impl RecordComponent {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid record component for class_file file {}! Expected name at index {} \
                in constant pool!", class_file_name, name_index))
            .clone();
        let descriptor_index = buf.get_u16();
        let descriptor_string = pool.get_string(descriptor_index as usize)
            .expect(&format!("Invalid record component for class_file file {}! Expected descriptor at \
                index {} in constant pool!", class_file_name, name_index));
        let descriptor = FieldType::parse(descriptor_string)
            .expect(&format!("Invalid descriptor {} for record component in class_file file {}!", descriptor_string, class_file_name));

        let attribute_count = buf.get_u16();
        let generic_signature = parse_attributes(class_file_name, pool, attribute_count, buf);
        RecordComponent { name, descriptor, generic_signature }
    }
}

impl Nameable for RecordComponent {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Generic for RecordComponent {
    fn generic_signature(&self) -> Option<&str> {
        self.generic_signature.as_ref().map(|value| value.as_str())
    }
}

fn parse_attributes(
    class_file_name: &str,
    pool: &ConstantPool,
    mut attribute_count: u16,
    buf: &mut Bytes,
) -> Option<String> {
    let mut generic_signature = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated record component attributes for field in class_file file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid record component attribute index {} in class_file file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found for \
                record component in class_file file {}!", class_file_name);
            generic_signature = parse_generic_signature(class_file_name, pool, attribute_length, buf, "record component");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    generic_signature
}
