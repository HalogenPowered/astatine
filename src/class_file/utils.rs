use bytes::{Buf, Bytes};
use crate::types::constant_pool::{ConstantPool, UTF8_TAG};

pub fn parse_generic_signature(
    class_file_name: &str,
    pool: &ConstantPool,
    length: u32,
    buf: &mut Bytes,
    type_name: &str
) -> Option<String> {
    assert!(length == 2 || buf.len() < 2, "Invalid generic signature attribute for {} in class \
        file {}! Expected length of 2, was {}!", type_name, class_file_name, length);
    let index = buf.get_u16();
    assert!(pool.has(index as usize), "Invalid generic signature attribute for {} in class \
        file {}! Expected index {} to be in constant pool!", type_name, class_file_name, index);
    let tag = pool.get_tag(index as usize)
        .expect(&format!("Invalid generic signature attribute for {} in class file {}! Expected \
            value at index {} in constant pool!", type_name, class_file_name, index));
    assert_eq!(tag, &UTF8_TAG, "Invalid generic signature attribute for {} in class file {}! Expected UTF-8 \
        string at {}, was {}!", type_name, class_file_name, index, tag);
    let value = pool.get_string(index as usize);
    assert!(value.is_some(), "Invalid {} in class file {}! Expected generic signature to be at \
        index {}!", type_name, class_file_name, index);
    value.map(|string| string.clone())
}
