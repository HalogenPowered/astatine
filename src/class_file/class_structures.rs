use bytes::{Buf, Bytes};
use crate::class_file::attributes::Attribute;
use crate::types::constant_pool::ConstantPool;
use crate::types::field::Field;
use crate::types::method::Method;

pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}
