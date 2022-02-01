use bytes::{Buf, Bytes};
use internship::IStr;
use std::fs;
use std::sync::Arc;
use crate::class_file::attribute_tags::*;
use crate::class_file::class_loader::ClassLoader;
use crate::class_file::version::ClassFileVersion;
use crate::types::method::BootstrapMethod;
use crate::utils::buffer::BufferExtras;
use crate::utils::constants::JAVA_LANG_OBJECT_NAME;
use super::access_flags::*;
use super::constant_pool::ConstantPool;
use super::field::Field;
use super::method::Method;
use super::record::RecordComponent;

#[derive(Debug)]
pub struct Class {
    loader: Arc<ClassLoader>,
    version: ClassFileVersion,
    access_flags: u16,
    constant_pool: ConstantPool,
    name: IStr,
    super_class: Option<Arc<Class>>,
    interfaces: Vec<Arc<Class>>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    source_file_name: Option<IStr>,
    inner_classes: Vec<InnerClassInfo>,
    record_components: Vec<RecordComponent>,
    bootstrap_methods: Vec<Arc<BootstrapMethod>>
}

const MAGIC_CLASS_FILE_VERSION: u32 = 0xCAFEBABE;

impl Class {
    pub(crate) fn parse(loader: Arc<ClassLoader>, file_name: &str) -> Self {
        let contents = fs::read(file_name)
            .expect(&format!("Class file name {} could not be read!", file_name));
        let mut buf = Bytes::from(contents);
        let magic = buf.get_u32();
        if magic != MAGIC_CLASS_FILE_VERSION {
            panic!("Invalid class file {}! Expected magic header {}, got {}!", file_name,
                   MAGIC_CLASS_FILE_VERSION, magic);
        }

        let minor_version = buf.get_u16();
        let major_version = buf.get_u16();
        let version = ClassFileVersion::from(major_version, minor_version);
        let constant_pool = ConstantPool::parse(&mut buf);

        let mut access_flags = if version >= ClassFileVersion::RELEASE_9 {
            buf.get_u16() & ALL_CLASS_MODIFIERS_J9
        } else {
            buf.get_u16() & ALL_CLASS_MODIFIERS
        };
        if access_flags & ACC_INTERFACE != 0 && version < ClassFileVersion::RELEASE_6 {
            // Set abstract flag for backwards compatibility
            access_flags |= ACC_ABSTRACT;
        }
        verify_modifiers(file_name, &version, access_flags);

        let this_class = buf.get_u16();
        let name = constant_pool.get_class_name(this_class as usize)
            .expect(&format!("Invalid name for class file {}! Expected index {} to be in \
                constant pool!", file_name, this_class));
        let super_class = resolve_superclass(Arc::clone(&loader), file_name, name.as_str(),
                                             &constant_pool, buf.get_u16(), access_flags);

        let interfaces = buf.get_generic_u16_array(|buf| {
            let index = buf.get_u16();
            constant_pool.get_class_no_holder(index as usize, Arc::clone(&loader))
                .expect(&format!("Invalid class file {}! Expected super interface index {} to be \
                    in constant pool!", file_name, index))
        });
        let fields = buf.get_generic_u16_array(|buf| {
            Field::parse(file_name, &constant_pool, buf, &version, access_flags)
        });
        let methods = buf.get_generic_u16_array(|buf| {
            Method::parse(Arc::clone(&loader), file_name, &constant_pool, buf, &version, access_flags)
        });

        let attribute_count = buf.get_u16();
        let (source_file_name, inner_classes, record_components, bootstrap_methods) = parse_attributes(
            file_name,
            &constant_pool,
            &mut buf,
            attribute_count,
            constant_pool.has_dynamic()
        );

        assert_eq!(buf.remaining(), 0, "Extra bytes found in class file {}!", file_name);
        Class {
            loader,
            version,
            access_flags,
            constant_pool,
            name,
            super_class,
            interfaces,
            fields,
            methods,
            source_file_name,
            inner_classes,
            record_components,
            bootstrap_methods
        }
    }

    pub fn new(
        loader: Arc<ClassLoader>,
        version: ClassFileVersion,
        access_flags: u16,
        constant_pool: ConstantPool,
        name: &str,
        super_class: Option<Arc<Class>>,
        interfaces: Vec<Arc<Class>>,
        fields: Vec<Field>,
        methods: Vec<Method>,
        source_file_name: Option<&str>,
        inner_classes: Vec<InnerClassInfo>,
        record_components: Vec<RecordComponent>,
        bootstrap_methods: Vec<Arc<BootstrapMethod>>
    ) -> Self {
        Class {
            loader,
            version,
            access_flags,
            constant_pool,
            name: IStr::new(name),
            super_class,
            interfaces,
            fields,
            methods,
            source_file_name: source_file_name.map(|value| IStr::new(value)),
            inner_classes,
            record_components,
            bootstrap_methods
        }
    }

    pub(crate) fn initialize(self: Arc<Class>) -> Arc<Class> {
        self.constant_pool.set_holder(Arc::clone(&self));
        self
    }

    pub fn loader(&self) -> Arc<ClassLoader> {
        Arc::clone(&self.loader)
    }

    pub fn version(&self) -> &ClassFileVersion {
        &self.version
    }

    pub fn major_version(&self) -> u16 {
        self.version().major()
    }

    pub fn minor_version(&self) -> u16 {
        self.version().minor()
    }

    pub fn constant_pool(&self) -> &ConstantPool {
        &self.constant_pool
    }

    pub fn super_class(&self) -> Option<Arc<Class>> {
        self.super_class.as_ref().map(|value| Arc::clone(value))
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn fields(&self) -> &[Field] {
        self.fields.as_slice()
    }

    pub fn methods(&self) -> &[Method] {
        self.methods.as_slice()
    }

    pub fn source_file_name(&self) -> Option<&str> {
        self.source_file_name.as_ref().map(|value| value.as_str())
    }

    pub fn inner_classes(&self) -> &[InnerClassInfo] {
        self.inner_classes.as_slice()
    }

    pub fn record_components(&self) -> &[RecordComponent] {
        self.record_components.as_slice()
    }

    pub fn bootstrap_methods(&self) -> &[Arc<BootstrapMethod>] {
        self.bootstrap_methods.as_slice()
    }

    pub fn is_super(&self) -> bool {
        self.access_flags & ACC_SUPER != 0
    }

    pub fn is_module(&self) -> bool {
        self.access_flags & ACC_MODULE != 0
    }

    pub fn is_subclass(&self, other: Arc<Class>) -> bool {
        if self as *const Class == other.as_ref() as *const Class {
            return true;
        }
        let mut super_class = self.super_class();
        while super_class.is_some() {
            let class = super_class.unwrap();
            if class.as_ref() as *const Class == other.as_ref() as *const Class {
                return true;
            }
            super_class = class.super_class();
        }
        false
    }
}

fn resolve_superclass(
    loader: Arc<ClassLoader>,
    class_file_name: &str,
    name: &str,
    pool: &ConstantPool,
    index: u16,
    flags: u16
) -> Option<Arc<Class>> {
    assert!(flags & ACC_INTERFACE == 0 || index != 0, "Invalid class file {}! Interfaces must \
        always have an explicit superclass!", class_file_name);
    if index == 0 {
        assert_eq!(name, JAVA_LANG_OBJECT_NAME, "Invalid class file {}! Every class other than {} \
            must have an explicit superclass of {} or one of its subclasses!", class_file_name,
            JAVA_LANG_OBJECT_NAME, JAVA_LANG_OBJECT_NAME);
        return None;
    }
    pool.get_class_no_holder(index as usize, loader)
}

impl_nameable!(Class);
impl_accessible!(Class);
impl_accessible!(Class, FinalAccessible);
impl_accessible!(Class, PublicAccessible);
impl_accessible!(Class, AbstractAccessible);
impl_accessible!(Class, EnumAccessible);
impl_accessible!(Class, InterfaceAnnotationAccessible);

#[derive(Debug)]
pub struct InnerClassInfo {
    index: u16,
    name: IStr,
    access_flags: u16,
    outer_index: u16
}

impl InnerClassInfo {
    pub(crate) fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let index = buf.get_u16();
        let outer_index = buf.get_u16();
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid inner class for class file {}! Expected name at \
                index {}!", class_file_name, name_index));
        let access_flags = buf.get_u16();
        InnerClassInfo { index, name, access_flags, outer_index }
    }

    pub fn new(index: u16, name: &str, access_flags: u16, outer_index: u16) -> Self {
        InnerClassInfo { index, name: IStr::new(name), access_flags, outer_index }
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn outer_index(&self) -> u16 {
        self.outer_index
    }
}

impl_nameable!(InnerClassInfo);
impl_accessible!(InnerClassInfo);
impl_accessible!(InnerClassInfo, FinalAccessible);
impl_accessible!(InnerClassInfo, PublicAccessible);
impl_accessible!(InnerClassInfo, AbstractAccessible);
impl_accessible!(InnerClassInfo, EnumAccessible);
impl_accessible!(InnerClassInfo, PrivateProtectedStaticAccessible);
impl_accessible!(InnerClassInfo, InterfaceAnnotationAccessible);

fn parse_attributes(
    class_file_name: &str,
    pool: &ConstantPool,
    buf: &mut Bytes,
    mut attribute_count: u16,
    bootstrap_methods_required: bool
) -> (Option<IStr>, Vec<InnerClassInfo>, Vec<RecordComponent>, Vec<Arc<BootstrapMethod>>) {
    let mut source_file_name = None;
    let mut inner_classes = None;
    let mut record_components = None;
    let mut bootstrap_methods = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated class attributes for class file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid class attribute index {} for class file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_SOURCE_FILE {
            assert_eq!(attribute_length, 2, "Invalid source file attribute for class file {}! \
                Expected length of 2, was {}!", class_file_name, attribute_length);
            assert!(source_file_name.is_none(), "Duplicate source file attribute found for class \
                file {}!", class_file_name);
            let source_file_index = buf.get_u16();
            let source_file = pool.get_string(source_file_index as usize)
                .expect(&format!("Invalid source file attribute for class file {}! Expected name \
                    index {} to be in constant pool!", class_file_name, source_file_index));
            source_file_name = Some(source_file);
        } else if attribute_name == TAG_INNER_CLASSES {
            assert!(inner_classes.is_none(), "Duplicate inner classes attribute found for class \
                file {}!", class_file_name);
            let number_of_classes = buf.get_u16();
            let mut classes = Vec::with_capacity(number_of_classes as usize);
            for _ in 0..number_of_classes {
                classes.push(InnerClassInfo::parse(class_file_name, pool, buf));
            }
            inner_classes = Some(classes);
        } else if attribute_name == TAG_RECORD {
            assert!(record_components.is_none(), "Duplicate record attribute found for class \
                file {}!", class_file_name);
            let components_count = buf.get_u16();
            let mut components = Vec::with_capacity(components_count as usize);
            for _ in 0..components_count {
                components.push(RecordComponent::parse(class_file_name, pool, buf));
            }
            record_components = Some(components);
        } else if attribute_name == TAG_BOOTSTRAP_METHODS {
            assert!(bootstrap_methods.is_none(), "Duplicate bootstrap methods attribute found for \
                class file {}!", class_file_name);
            let methods_count = buf.get_u16();
            let mut methods = Vec::with_capacity(methods_count as usize);
            for _ in 0..methods_count {
                methods.push(Arc::new(BootstrapMethod::parse(class_file_name, pool, buf)));
            }
            bootstrap_methods = Some(methods)
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }

    if bootstrap_methods_required {
        assert!(bootstrap_methods.is_some(), "Invalid attributes for class file {}! Bootstrap \
            methods must be present if the class file has a Dynamic or InvokeDynamic constant in \
            the constant pool!", class_file_name);
    }
    (
        source_file_name,
        inner_classes.unwrap_or(Vec::new()),
        record_components.unwrap_or(Vec::new()),
        bootstrap_methods.unwrap_or(Vec::new())
    )
}

fn verify_modifiers(class_file_name: &str, version: &ClassFileVersion, flags: u16) {
    let is_module = flags & ACC_MODULE != 0;
    assert!(version >= &ClassFileVersion::RELEASE_9 || !is_module, "Invalid class \
        file {}! Module flag should not be set for classes before Java 9!", class_file_name);
    assert!(!is_module, "Cannot load class file {} as it is a module!", class_file_name);

    let is_final = flags & ACC_FINAL != 0;
    let is_super = flags & ACC_SUPER != 0;
    let is_interface = flags & ACC_INTERFACE != 0;
    let is_abstract = flags & ACC_ABSTRACT != 0;
    let is_annotation = flags & ACC_ANNOTATION != 0;
    let is_enum = flags & ACC_ENUM != 0;
    let major_1_5_or_above = version >= &ClassFileVersion::RELEASE_1_5;

    let is_illegal = (is_abstract && is_final) ||
        (is_interface && !is_abstract) ||
        (is_interface && major_1_5_or_above && (is_super || is_enum)) ||
        (!is_interface && major_1_5_or_above && is_annotation);
    assert!(!is_illegal, "Invalid class file {}! Illegal class modifiers {}!", class_file_name, flags);
}
