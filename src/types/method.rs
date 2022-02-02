use bytes::{Buf, Bytes};
use internship::IStr;
use std::sync::Arc;
use crate::class_file::attribute_tags::*;
use crate::class_file::{ClassLoader, ClassFileVersion, parse_generic_signature};
use crate::class_file::code::CodeBlock;
use crate::objects::handles::MethodHandle;
use crate::utils::BufferExtras;
use crate::utils::constants::*;
use crate::utils::descriptors::MethodDescriptor;
use super::access_flags::*;
use super::constant_pool::ConstantPool;

#[derive(Debug)]
pub struct Method {
    name: IStr,
    descriptor: MethodDescriptor,
    generic_signature: Option<IStr>,
    access_flags: u16,
    parameters: Vec<MethodParameter>,
    code: Option<CodeBlock>,
    checked_exception_indices: Vec<u16>,
    other_flags: u8
}

// These aren't part of the spec, this is just the best way I could think of compactly storing
// extra flags.
pub const METHOD_IS_CONSTRUCTOR: u8 = 0x01;
pub const METHOD_IS_STATIC_INITIALIZER: u8 = 0x02;

impl Method {
    pub(crate) fn parse(
        loader: Arc<ClassLoader>,
        class_file_name: &str,
        pool: &ConstantPool,
        buf: &mut Bytes,
        version: &ClassFileVersion,
        class_flags: u16
    ) -> Self {
        let mut access_flags = buf.get_u16();
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid method in class file {}! Expected name index {} to be in \
                constant pool!", class_file_name, name_index))
            .clone();
        let descriptor_index = buf.get_u16();
        let descriptor = pool.get_utf8(descriptor_index as usize)
            .and_then(|value| MethodDescriptor::parse(value.as_str()))
            .expect(&format!("Invalid descriptor for method in class file {}!", class_file_name));

        let mut other_flags: u8 = 0;
        if name == CLASS_INITIALIZER_METHOD_NAME {
            assert!(descriptor.return_type().is_none(), "Invalid method descriptor {:?} for \
                static initializer ({})! Static initializer must return \
                void!", descriptor, CLASS_INITIALIZER_METHOD_NAME);
            if version >= &ClassFileVersion::RELEASE_7 {
                assert!(descriptor.parameters().is_empty(), "Invalid method descriptor {:?} for \
                    static initializer ({})! Static initializer must take no \
                    parameters!", descriptor, CLASS_INITIALIZER_METHOD_NAME);
            }
            other_flags |= METHOD_IS_STATIC_INITIALIZER;
            if version < &ClassFileVersion::RELEASE_7 {
                access_flags = ACC_STATIC;
            } else if (access_flags & ACC_STATIC) == ACC_STATIC {
                let extra_flag = if version <= &ClassFileVersion::RELEASE_16 { ACC_STRICT } else { 0 };
                access_flags &= ACC_STATIC | extra_flag;
            } else {
                panic!("Invalid static initializer method ({}) in class file {}! Must be \
                    static!", CLASS_INITIALIZER_METHOD_NAME, class_file_name);
            }
        } else {
            verify_method_flags(class_file_name, version, class_flags, access_flags, &name);
        }
        if name == OBJECT_INITIALIZER_METHOD_NAME {
            other_flags |= METHOD_IS_CONSTRUCTOR;
            assert_eq!(class_flags & ACC_INTERFACE, 0, "Invalid class file {}! Interface cannot \
                have a constructor!", class_file_name);
        }

        let attribute_count = buf.get_u16();
        let (code, checked_exception_indices, parameters, generic_signature) = parse_attributes(
            loader,
            class_file_name,
            pool,
            buf,
            version,
            access_flags,
            attribute_count
        );
        if access_flags & ACC_ABSTRACT == 0 && access_flags & ACC_NATIVE == 0 {
            assert!(code.is_some(), "Non-abstract and non-native methods must have code \
                attributes!");
        } else {
            assert!(code.is_none(), "Abstract and native methods must not have code attributes!");
        }
        Method {
            name,
            descriptor,
            generic_signature,
            access_flags,
            parameters,
            code,
            checked_exception_indices,
            other_flags
        }
    }

    pub fn new(
        name: &str,
        descriptor: MethodDescriptor,
        generic_signature: Option<&str>,
        access_flags: u16,
        parameters: Vec<MethodParameter>,
        code: Option<CodeBlock>,
        checked_exception_indices: Vec<u16>,
        other_flags: u8
    ) -> Self {
        Method {
            name: IStr::new(name),
            descriptor,
            generic_signature: generic_signature.map(|value| IStr::new(value)),
            access_flags,
            parameters,
            code,
            checked_exception_indices,
            other_flags
        }
    }

    // TODO: Procedural macros
    named!();
    describable!(MethodDescriptor);
    flagged_final!();
    flagged_public!();
    flagged_abstract!();
    flagged_private_protected_static!();

    pub fn parameters(&self) -> &[MethodParameter] {
        self.parameters.as_slice()
    }

    pub fn code(&self) -> Option<&CodeBlock> {
        self.code.as_ref()
    }

    pub fn is_constructor(&self) -> bool {
        self.other_flags & METHOD_IS_CONSTRUCTOR != 0
    }

    pub fn is_static_initializer(&self) -> bool {
        self.other_flags & METHOD_IS_STATIC_INITIALIZER != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.access_flags & ACC_SYNCHRONIZED != 0
    }

    pub fn is_bridge(&self) -> bool {
        self.access_flags & ACC_BRIDGE != 0
    }

    pub fn is_varargs(&self) -> bool {
        self.access_flags & ACC_VARARGS != 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags & ACC_NATIVE != 0
    }

    pub fn is_strict(&self) -> bool {
        self.access_flags & ACC_STRICT != 0
    }
}

impl_accessible!(Method);

#[derive(Debug)]
pub struct BootstrapMethod {
    handle: Arc<MethodHandle>,
    arguments: Vec<u16>
}

impl BootstrapMethod {
    pub(crate) fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let handle_index = buf.get_u16();
        let handle = pool.get_method_handle(handle_index as usize)
            .expect(&format!("Invalid bootstrap method in class file {}! Expected index {} to be \
                in constant pool!", class_file_name, handle_index));
        BootstrapMethod::new(handle, buf.get_u16_array())
    }

    pub const fn new(handle: Arc<MethodHandle>, arguments: Vec<u16>) -> Self {
        BootstrapMethod { handle, arguments }
    }

    pub fn handle(&self) -> Arc<MethodHandle> {
        Arc::clone(&self.handle)
    }

    pub fn arguments(&self) -> &[u16] {
        self.arguments.as_slice()
    }
}

#[derive(Debug)]
pub struct MethodParameter {
    name: IStr,
    access_flags: u16
}

impl MethodParameter {
    pub(crate) fn parse(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let name = pool.get_string(name_index as usize)
            .expect(&format!("Invalid method parameter for method in class file {}! Expected name \
                at index {}!", class_file_name, name_index));
        let access_flags = buf.get_u16();
        MethodParameter { name, access_flags }
    }

    pub fn new(name: &str, access_flags: u16) -> Self {
        MethodParameter { name: IStr::from(name), access_flags }
    }

    named!();
    flagged_final!();
    flagged_mandated!();
}

impl_accessible!(MethodParameter);

fn parse_attributes(
    loader: Arc<ClassLoader>,
    class_file_name: &str,
    pool: &ConstantPool,
    buf: &mut Bytes,
    version: &ClassFileVersion,
    access_flags: u16,
    mut attribute_count: u16
) -> (Option<CodeBlock>, Vec<u16>, Vec<MethodParameter>, Option<IStr>) {
    let mut code = None;
    let mut checked_exception_indices = Vec::new();
    let mut parameters = Vec::new();
    let mut generic_signature = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated method attributes for method in class \
            file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid method attribute index {} in class file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_CODE {
            assert!(code.is_none(), "Expected single code attribute for method in class \
                file {}!", class_file_name);
            assert!(access_flags & ACC_NATIVE == 0 && access_flags & ACC_ABSTRACT == 0, "Invalid \
                code attribute for method in class file {}! Abstract and native methods must not \
                have code attributes!", class_file_name);
            code = Some(CodeBlock::parse(Arc::clone(&loader), class_file_name, pool, buf));
        } else if attribute_name == TAG_EXCEPTIONS {
            assert!(checked_exception_indices.is_empty(), "Expected single exceptions attribute \
                for method in class file {}!", class_file_name);
            let number_of_exceptions = buf.get_u16();
            for _ in 0..number_of_exceptions {
                checked_exception_indices.push(buf.get_u16());
            }
        } else if attribute_name == TAG_METHOD_PARAMETERS {
            assert!(parameters.is_empty(), "Expected single method parameters attribute for \
                method in class file {}!", class_file_name);
            let count = buf.get_u16();
            for _ in 0..count {
                parameters.push(MethodParameter::parse(class_file_name, pool, buf));
            }
        } else if attribute_name == TAG_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for method in \
                class file {}!", attribute_length, class_file_name);
        } else if attribute_name == TAG_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for method in \
                class file {}!", attribute_length, class_file_name);
        } else if version >= &ClassFileVersion::RELEASE_1_5 && attribute_name == TAG_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found \
                for method in class file {}!", class_file_name);
            generic_signature = parse_generic_signature(class_file_name, pool, buf,
                                                        attribute_length, "method");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (code, checked_exception_indices, parameters, generic_signature)
}

fn verify_method_flags(
    class_file_name: &str,
    version: &ClassFileVersion,
    class_flags: u16,
    flags: u16,
    name: &str
) {
    let is_public = (flags & ACC_PUBLIC) != 0;
    let is_private = (flags & ACC_PRIVATE) != 0;
    let is_protected = (flags & ACC_PROTECTED) != 0;
    let is_static = (flags & ACC_STATIC) != 0;
    let is_final = (flags & ACC_FINAL) != 0;
    let is_synchronized = (flags & ACC_SYNCHRONIZED) != 0;
    let is_bridge = (flags & ACC_BRIDGE) != 0;
    let is_native = (flags & ACC_NATIVE) != 0;
    let is_abstract = (flags & ACC_ABSTRACT) != 0;
    let is_strict = (flags & ACC_STRICT) != 0;
    let major_1_5_or_above = version >= &ClassFileVersion::RELEASE_1_5;
    let major_8_or_above = version >= &ClassFileVersion::RELEASE_8;
    let major_17_or_above = version >= &ClassFileVersion::RELEASE_17;
    let is_constructor = name == OBJECT_INITIALIZER_METHOD_NAME;

    let is_illegal;
    if class_flags & ACC_INTERFACE != 0 {
        if major_8_or_above {
            is_illegal = (is_public == is_private) || // Methods can't be both public and private
                // None of these are allowed on interface methods
                (is_native || is_protected || is_final || is_synchronized) ||
                // Interface instance methods can't be private, static, or strict
                (is_abstract && (is_private || is_static || (!major_17_or_above && is_strict)));
        } else if major_1_5_or_above {
            // Interface instance methods must be public and abstract
            is_illegal = !is_public || is_private || is_protected || is_static || is_final ||
                is_synchronized || is_native || !is_abstract || is_strict;
        } else {
            is_illegal = !is_public || is_static || is_final || is_native || !is_abstract;
        }
    } else {
        is_illegal = has_illegal_visibility(flags) ||
            // Constructor methods are instance methods that must have bodies, must not be
            // generated bridge methods, and aren't final, as the class' access determines the
            // constructor's access.
            (is_constructor && (is_static || is_final || is_synchronized || is_native ||
                is_abstract || (major_1_5_or_above && is_bridge))) ||
            // Abstract methods must be overridable by subclasses, and so none of these would make
            // sense.
            (is_abstract && (is_final || is_native || is_private || is_static ||
                (major_1_5_or_above && (is_synchronized || (!major_17_or_above && is_strict)))));
    }

    assert!(!is_illegal, "Invalid method in class file {}! Access modifiers {} are \
        illegal!", class_file_name, flags);
}

fn has_illegal_visibility(flags: u16) -> bool {
    let is_public = (flags & ACC_PUBLIC) != 0;
    let is_protected = (flags & ACC_PROTECTED) != 0;
    let is_private = (flags & ACC_PRIVATE) != 0;
    return (is_public && is_protected) || (is_public && is_private) || (is_protected && is_private)
}
