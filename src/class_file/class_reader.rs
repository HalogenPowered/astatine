use crate::class_structures::*;
use crate::conversions::{read_u16_array, u16_to_u8_array};
use bytes::{Bytes, Buf};
use std::{fs, mem};
use std::io::Read;
use std::ptr::null;
use crate::class_file::attributes::Attribute;
use crate::class_file::class_structures::*;
use crate::types::constant_pool::{ConstantPool, PoolConstant, read_constant_pool};
use crate::types::field::Field;
use crate::utils::buffer_extras::read_u16_array;

const MAGIC_CLASS_FILE_VERSION: u32 = 0xCAFEBABE;

pub fn read_class_file(class_file_name: &str) -> ClassFile {
    let contents = fs::read(class_file_name)
        .expect("Class file name" + class_file_name + "could not be read!");
    let mut buf = Bytes::from(contents);
    let magic = buf.get_u32();
    if magic != MAGIC_CLASS_FILE_VERSION {
        panic!("Invalid class file {}! Expected magic header {}, got {}!", class_file_name, MAGIC_CLASS_FILE_VERSION, magic);
    }
    let minor_version = buf.get_u16();
    let major_version = buf.get_u16();
    let constant_pool = read_constant_pool(&mut buf);
    let access_flags = buf.get_u16();
    let this_class = buf.get_u16();
    let super_class = buf.get_u16();
    let interfaces = read_u16_array(&mut buf);
}

fn read_field(pool: &ConstantPool, buf: &mut Bytes) -> Field {
    let access_flags = buf.get_u16();
    let name_index = buf.get_u16();
    let descriptor_index = buf.get_u16();
}

fn read_attributes(buf: &mut Bytes) -> Vec<Attribute> {
    let count = buf.get_u16();
    let mut attributes = Vec::with_capacity(count);
    for _ in 0..count {
        attributes.push(Attribute::parse())
    }
}

fn read_attribute()
