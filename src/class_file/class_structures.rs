use bytes::{Buf, Bytes};

pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<ElementInfo>,
    pub methods: Vec<ElementInfo>,
    pub attributes: Vec<AttributeInfo>,
}

pub type ConstantPool = Vec<ConstantPoolEntry>;

pub struct ConstantPoolEntry {
    pub tag: u8,
    pub info: ConstantPoolInfo
}

// Any data with a name ending in "index" refers to an index somewhere else in the constant pool
#[repr(u8)]
pub enum ConstantPoolInfo {
    Utf8 { value: String } = 1,
    Integer { value: u32 } = 3,
    Float { value: f32 } = 4,
    Long { value: u64 } = 5,
    Double { value: f64 } = 6,
    Class { name_index: u16 } = 7,
    String { string_index: u16 } = 8,
    FieldRef { class_index: u16, name_and_type_index: u16 } = 9,
    MethodRef { class_index: u16, name_and_type_index: u16 } = 10,
    InterfaceMethodRef { class_index: u16, name_and_type_index: u16 } = 11,
    NameAndType { name_index: u16, descriptor_index: u16 } = 12,
    MethodHandle { reference_kind: u8, reference_index: u16 } = 15,
    MethodType { descriptor_index: u16 } = 16,
    Dynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 } = 17,
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 } = 18,
    Module { name_index: u16 } = 19,
    Package { name_index: u16 } = 20
}

// Used for both field_info and method_info.
// They have the same fields in the same order with the same types, there's no point repeating ourselves.
pub struct ElementInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>
}

impl ElementInfo {
    fn read_from(mut buf: &Bytes) -> Self {
        let access_flags = buf.get_u16();
        let name_index = buf.get_u16();
        let descriptor_index = buf.get_u16();
        let attributes_count = buf.get_u16();
    }
}

pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub info: Vec<u8>
}
