use bytes::{Buf, Bytes};
use crate::types::constant_pool::ConstantPool;
use crate::types::field::Field;
use crate::types::method::Method;
use crate::types::record::RecordComponent;
use crate::types::utils::Nameable;

pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub access_flags: u16,
    pub constant_pool: ConstantPool,
    pub name: String,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub source_file_name: Option<String>,
    pub inner_classes: Option<Vec<InnerClassInfo>>,
    pub record_components: Option<Vec<RecordComponent>>,
}

impl Nameable for Class {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct InnerClassInfo {
    pub index: u16,
    name: String,
    pub access_flags: u16,
    pub outer_index: u16
}

impl InnerClassInfo {
    pub fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let index = buf.get_u16();
        let outer_index = buf.get_u16();
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid inner class_file for class_file file {}! Expected name at \
                index {}!", class_file_name, name_index))
            .clone();
        let access_flags = buf.get_u16();
        InnerClassInfo { index, name, access_flags, outer_index }
    }
}

impl Nameable for InnerClassInfo {
    fn name(&self) -> &str {
        &self.name
    }
}
