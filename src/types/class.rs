use bytes::{Buf, Bytes};
use crate::types::constant_pool::ConstantPool;
use crate::types::field::Field;
use crate::types::method::Method;
use crate::types::record::RecordComponent;

pub struct Class<'a> {
    pub minor_version: u16,
    pub major_version: u16,
    pub access_flags: u16,
    pub constant_pool: ConstantPool,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field<'a>>,
    pub methods: Vec<Method<'a>>,
    pub source_file_name: Option<&'a str>,
    pub inner_classes: Option<Vec<InnerClassInfo<'a>>>,
    pub record_components: Option<Vec<RecordComponent<'a>>>,
}

pub struct InnerClassInfo<'a> {
    pub index: u16,
    pub name: &'a str,
    pub access_flags: u16,
    pub outer_index: u16
}

impl<'a> InnerClassInfo<'a> {
    pub fn parse(class_file_name: &str, pool: &'a ConstantPool, buf: &mut Bytes) -> Self {
        let index = buf.get_u16();
        let outer_index = buf.get_u16();
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid inner class for class file {}! Expected name at \
                index {}!", class_file_name, name_index));
        let access_flags = buf.get_u16();
        InnerClassInfo { index, name, access_flags, outer_index }
    }
}
