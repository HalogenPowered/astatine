use crate::class_structures::*;
use crate::conversions::{read_u16_array, u16_to_u8_array};
use bytes::{Bytes, Buf};
use std::fs;
use std::io::Read;
use std::ptr::null;

pub fn read(class_file_name: String) -> ClassFile {
    let contents = fs::read(class_file_name)
        .expect(&*format!("Class file name {} could not be read!", class_file_name));
    let mut buf = Bytes::from(contents);
    let magic = buf.get_u32();
    let minor_version = buf.get_u16();
    let major_version = buf.get_u16();
    let constant_pool = read_constant_pool(buf);
    let access_flags = buf.get_u16();
    let this_class = buf.get_u16();
    let super_class = buf.get_u16();
    let interfaces = read_u16_array(&buf);

}

fn read_constant_pool(mut buf: Bytes) -> Vec<ConstantPoolEntry> {
    let count = buf.get_u16();
    let mut pool = Vec::with_capacity(count as usize);
    for _ in 0..count {
        pool.push(**read_constant_pool_entry(&buf))
    }
    pool
}

fn read_constant_pool_entry(mut buf: &Bytes) -> Box<ConstantPoolEntry> {
    let tag = buf.get_u8();
    let info: dyn ConstantPoolInfo = match tag {
        UTF8_TAG => Utf8ConstantInfo::read_from(UTF8_TAG, buf),
        INTEGER_TAG | FLOAT_TAG => SinglePrimitiveConstantInfo::read_from(tag, buf),
        LONG_TAG | DOUBLE_TAG => DoublePrimitiveConstantInfo::read_from(tag, buf),
        CLASS_TAG | MODULE_TAG | PACKAGE_TAG => StructureConstantInfo::read_from(tag, buf),
        STRING_TAG => StringConstantInfo::read_from(tag, buf),
        FIELD_REF_TAG | METHOD_REF_TAG | INTERFACE_METHOD_REF_TAG => ElementRefConstantInfo::read_from(tag, buf),
        NAME_AND_TYPE_TAG => NameAndTypeConstantInfo::read_from(tag, buf),
        METHOD_HANDLE_TAG => MethodHandleConstantInfo::read_from(tag, buf),
        METHOD_TYPE_TAG => MethodTypeConstantInfo::read_from(tag, buf),
        DYNAMIC_TAG | INVOKE_DYNAMIC_TAG => DynamicConstantInfo::read_from(tag, buf),
        _ => panic!(format!("Unexpected tag {}!", tag))
    };
    Box::new(ConstantPoolEntry { tag, info })
}

fn read_elements(mut buf: &Bytes) -> Vec<ElementInfo> {
    let count = buf.get_u16();
}

fn read_element(mut buf: &Bytes) -> ElementInfo {
    let access_flags
}
