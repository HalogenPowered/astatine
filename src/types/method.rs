/*
 * Copyright (C) 2022 Callum Seabrook <callum.seabrook@prevarinite.com>
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; version 2.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program; if not, write to the Free Software Foundation, Inc., 51 Franklin
 * Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */

use astatine_macros::{Nameable, MethodDescribable, accessible};
use bytes::{Buf, Bytes};
use internship::IStr;
use std::sync::Arc;
use crate::class_file::{ClassLoader, parse_generic_signature};
use crate::class_file::code::CodeBlock;
use crate::constants::*;
use crate::objects::handles::MethodHandle;
use crate::utils::BufferExtras;
use crate::utils::descriptors::MethodDescriptor;
use super::access_flags::*;
use super::constant_pool::ConstantPool;

#[accessible(final, public, abstract, private, protected, static)]
#[derive(Debug, Nameable, MethodDescribable)]
pub struct Method {
    name: IStr,
    descriptor: MethodDescriptor,
    access_flags: AccessFlags,
    generic_signature: Option<IStr>,
    parameters: Vec<MethodParameter>,
    code: Option<CodeBlock>,
    checked_exception_indices: Vec<u16>
}

impl Method {
    pub(crate) fn parse(
        loader: Arc<ClassLoader>,
        class_file_name: &str,
        pool: &ConstantPool,
        buf: &mut Bytes,
        major_version: u16,
        class_flags: AccessFlags
    ) -> Self {
        let mut access_flags = buf.get_u16() as u32;
        let name_index = buf.get_u16();
        let name = pool.get_utf8(name_index as usize)
            .expect(&format!("Invalid method! Expected name index {} to be in constant pool!", name_index));
        let descriptor = pool.get_utf8(buf.get_u16() as usize)
            .and_then(|value| MethodDescriptor::parse(value.as_str()))
            .expect("Invalid method descriptor!");

        if name == JVM_CLASS_INITIALIZER_NAME {
            assert!(descriptor.return_type().is_none(), "Invalid method descriptor {:?} for \
                static initializer ({})! Static initializer must return \
                void!", descriptor, JVM_CLASS_INITIALIZER_NAME);
            if major_version >= JAVA_VERSION_7 {
                assert!(descriptor.parameters().is_empty(), "Invalid method descriptor {:?} for \
                    static initializer ({})! Static initializer must take no \
                    parameters!", descriptor, JVM_CLASS_INITIALIZER_NAME);
            }
            access_flags |= JVM_ACC_STATIC_INITIALIZER;
            if major_version < JAVA_VERSION_7 {
                access_flags = JVM_ACC_STATIC;
            } else if (access_flags & JVM_ACC_STATIC) == JVM_ACC_STATIC {
                let extra_flag = if major_version <= JAVA_VERSION_16 { JVM_ACC_STRICT } else { 0 };
                access_flags &= JVM_ACC_STATIC | extra_flag;
            } else {
                panic!("Invalid static initializer method ({})! Must be static!", JVM_CLASS_INITIALIZER_NAME);
            }
        } else {
            verify_method_flags(major_version, class_flags, access_flags, &name);
        }
        if name == JVM_OBJECT_INITIALIZER_NAME {
            access_flags |= JVM_ACC_CONSTRUCTOR;
            assert!(class_flags.is_interface(), "Invalid class file {}! Interface cannot have a \
                constructor!", class_file_name);
        }

        let attributes = parse_attributes(loader, pool, buf, major_version, access_flags);
        if access_flags & JVM_ACC_ABSTRACT == 0 && access_flags & JVM_ACC_NATIVE == 0 {
            assert!(attributes.0.is_some(), "Non-abstract and non-native methods must have code \
                attributes!");
        } else {
            assert!(attributes.0.is_none(), "Abstract and native methods must not have code attributes!");
        }
        let access_flags = AccessFlags::new(access_flags);
        Method {
            name,
            descriptor,
            access_flags,
            generic_signature: attributes.3,
            parameters: attributes.2.unwrap_or(Vec::new()),
            code: attributes.0,
            checked_exception_indices: attributes.1.unwrap_or(Vec::new())
        }
    }

    pub fn code(&self) -> Option<&CodeBlock> {
        self.code.as_ref()
    }

    pub fn is_constructor(&self) -> bool {
        self.access_flags.value() & JVM_ACC_CONSTRUCTOR != 0
    }

    pub fn is_static_initializer(&self) -> bool {
        self.access_flags.value() & JVM_ACC_STATIC_INITIALIZER != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.access_flags.value() & JVM_ACC_SYNCHRONIZED != 0
    }

    pub fn is_bridge(&self) -> bool {
        self.access_flags.value() & JVM_ACC_BRIDGE != 0
    }

    pub fn is_varargs(&self) -> bool {
        self.access_flags.value() & JVM_ACC_VARARGS != 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags.value() & JVM_ACC_NATIVE != 0
    }

    pub fn is_strict(&self) -> bool {
        self.access_flags.value() & JVM_ACC_STRICT != 0
    }
}

#[derive(Debug)]
pub struct BootstrapMethod {
    handle: Arc<MethodHandle>,
    arguments: Vec<u16>
}

impl BootstrapMethod {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let handle = pool.get_method_handle(buf.get_u16() as usize)
            .expect(&format!("Invalid bootstrap method! Expected index to be in constant pool!"));
        BootstrapMethod { handle, arguments: buf.get_u16_array() }
    }

    pub fn handle(&self) -> &MethodHandle {
        &self.handle
    }

    pub fn arguments(&self) -> &[u16] {
        self.arguments.as_slice()
    }
}

#[accessible(final)]
#[derive(Debug)]
pub struct MethodParameter {
    name: Option<IStr>,
    access_flags: AccessFlags
}

const ACC_MANDATED: u32 = 0x8000;

impl MethodParameter {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        assert!(name_index == 0 || pool.has(name_index as usize));
        let name = pool.get_utf8(name_index as usize);
        let access_flags = AccessFlags::from(buf.get_u16());
        MethodParameter { name, access_flags }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|value| value.as_str())
    }

    pub fn is_mandated(&self) -> bool {
        self.access_flags.value() & ACC_MANDATED != 0
    }
}

type MethodAttributes = (Option<CodeBlock>, Option<Vec<u16>>, Option<Vec<MethodParameter>>, Option<IStr>);

fn parse_attributes(
    loader: Arc<ClassLoader>,
    pool: &ConstantPool,
    buf: &mut Bytes,
    major_version: u16,
    access_flags: u32
) -> MethodAttributes {
    let mut code = None;
    let mut checked_exception_indices = None;
    let mut parameters = None;
    let mut generic_signature = None;

    let mut attribute_count = buf.get_u16();
    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated method attributes!");
        let attribute_name = pool.get_utf8(buf.get_u16() as usize).unwrap();
        let attribute_length = buf.get_u32();

        if attribute_name == JVM_ATTRIBUTE_CODE {
            assert!(code.is_none(), "Expected single code attribute for method!");
            assert!(access_flags & JVM_ACC_NATIVE == 0 && access_flags & JVM_ACC_ABSTRACT == 0, "Invalid \
                method code attribute! Abstract and native methods must not have code attributes!");
            code = Some(CodeBlock::parse(Arc::clone(&loader), pool, buf));
        } else if attribute_name == JVM_ATTRIBUTE_EXCEPTIONS {
            assert!(checked_exception_indices.is_none(), "Expected single exceptions attribute for method!");
            let number_of_exceptions = buf.get_u16();
            let mut exceptions = Vec::new();
            for _ in 0..number_of_exceptions {
                exceptions.push(buf.get_u16());
            }
            checked_exception_indices = Some(exceptions)
        } else if attribute_name == JVM_ATTRIBUTE_METHOD_PARAMETERS {
            assert!(parameters.is_none(), "Expected single method parameters attribute for method!");
            let count = buf.get_u16();
            let mut parameter_list = Vec::new();
            for _ in 0..count {
                parameter_list.push(MethodParameter::parse(pool, buf));
            }
            parameters = Some(parameter_list)
        } else if attribute_name == JVM_ATTRIBUTE_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for method!", attribute_length);
        } else if attribute_name == JVM_ATTRIBUTE_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for method!", attribute_length);
        } else if major_version >= JAVA_VERSION_1_5 && attribute_name == JVM_ATTRIBUTE_SIGNATURE {
            assert!(generic_signature.is_none(), "Duplicate generic signature attribute found for method!");
            generic_signature = parse_generic_signature(pool, buf, attribute_length, "method");
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (code, checked_exception_indices, parameters, generic_signature)
}

fn verify_method_flags(major_version: u16, class_flags: AccessFlags, flags: u32, name: &str) {
    let is_public = (flags & JVM_ACC_PUBLIC) != 0;
    let is_private = (flags & JVM_ACC_PRIVATE) != 0;
    let is_protected = (flags & JVM_ACC_PROTECTED) != 0;
    let is_static = (flags & JVM_ACC_STATIC) != 0;
    let is_final = (flags & JVM_ACC_FINAL) != 0;
    let is_synchronized = (flags & JVM_ACC_SYNCHRONIZED) != 0;
    let is_bridge = (flags & JVM_ACC_BRIDGE) != 0;
    let is_native = (flags & JVM_ACC_NATIVE) != 0;
    let is_abstract = (flags & JVM_ACC_ABSTRACT) != 0;
    let is_strict = (flags & JVM_ACC_STRICT) != 0;
    let major_1_5_or_above = major_version >= JAVA_VERSION_1_5;
    let major_8_or_above = major_version >= JAVA_VERSION_8;
    let major_17_or_above = major_version >= JAVA_VERSION_17;
    let is_constructor = name == JVM_OBJECT_INITIALIZER_NAME;

    let is_illegal = if class_flags.is_interface() {
        if major_8_or_above {
            (is_public == is_private) || // Methods can't be both public and private
                // None of these are allowed on interface methods
                (is_native || is_protected || is_final || is_synchronized) ||
                // Interface instance methods can't be private, static, or strict
                (is_abstract && (is_private || is_static || (!major_17_or_above && is_strict)))
        } else if major_1_5_or_above {
            // Interface instance methods must be public and abstract
            !is_public || is_private || is_protected || is_static || is_final || is_synchronized |
                is_native || !is_abstract || is_strict
        } else {
            !is_public || is_static || is_final || is_native || !is_abstract
        }
    } else {
        has_illegal_visibility(flags) ||
            // Constructor methods are instance methods that must have bodies, must not be
            // generated bridge methods, and aren't final, as the class' access determines the
            // constructor's access.
            (is_constructor && (is_static || is_final || is_synchronized || is_native ||
                is_abstract || (major_1_5_or_above && is_bridge))) ||
            // Abstract methods must be overridable by subclasses, and so none of these would make
            // sense.
            (is_abstract && (is_final || is_native || is_private || is_static ||
                (major_1_5_or_above && (is_synchronized || (!major_17_or_above && is_strict)))))
    };
    assert!(!is_illegal, "Invalid method! Access modifiers {} are illegal!", flags);
}

fn has_illegal_visibility(flags: u32) -> bool {
    let is_public = (flags & JVM_ACC_PUBLIC) != 0;
    let is_protected = (flags & JVM_ACC_PROTECTED) != 0;
    let is_private = (flags & JVM_ACC_PRIVATE) != 0;
    return (is_public && is_protected) || (is_public && is_private) || (is_protected && is_private)
}
