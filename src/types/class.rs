use crate::class_file::attributes::Attribute;
use crate::types::constant_pool::{ConstantPool, PoolConstant};
use crate::types::field::Field;
use crate::types::method::Method;

pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub access_flags: u16,
    pub constant_pool: ConstantPool,
    pub this_class: PoolConstant::Class,
    pub super_class: PoolConstant::Class,
    pub interfaces: Vec<PoolConstant::Class>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub source_file_name: String,
}

// TODO: Add annotations
pub struct RecordComponent {
    pub name: String,
    pub descriptor: String,
    pub signature: String,
}
