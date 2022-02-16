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

use enum_as_inner::EnumAsInner;
use internship::IStr;
use std::fmt::Debug;
use std::sync::Arc;
use astatine_macros::{Nameable, FieldDescribable, MethodDescribable};
use crate::constants::*;
use crate::types::{Class, ConstantPool};
use crate::utils::descriptors::{FieldDescriptor, MethodDescriptor};

#[derive(Debug)]
pub struct MethodHandle {
    kind: u8,
    reference: MethodHandleRef
}

impl MethodHandle {
    pub(crate) fn parse(
        pool: &ConstantPool,
        kind: u8,
        reference_index: u16,
        major_version: u16
    ) -> Self {
        assert!(kind >= JVM_REF_GET_FIELD && kind <= JVM_REF_INVOKE_INTERFACE, "Invalid method \
            handle kind {}!", kind);
        assert!(pool.has(reference_index as usize), "Invalid method handle reference index {}!", reference_index);
        if kind <= JVM_REF_PUT_STATIC { // FieldRef
            let reference = pool.get_field_ref(reference_index as usize)
                .expect(&format!("Invalid method handle! Expected field ref index {} to be in \
                    constant pool!", reference_index));
            return MethodHandle { kind, reference: MethodHandleRef::Field(reference) }
        }
        let reference = lookup_method_ref(pool, kind, reference_index, major_version);
        MethodHandle { kind, reference: MethodHandleRef::Method(reference) }
    }

    pub fn is_field_ref(&self) -> bool {
        self.kind <= JVM_REF_PUT_STATIC
    }

    pub fn is_method_ref(&self) -> bool {
        self.kind >= JVM_REF_INVOKE_VIRTUAL
    }

    pub fn field_ref(&self) -> Option<Arc<FieldRef>> {
        self.reference.as_field().map(|value| Arc::clone(value))
    }

    pub fn method_ref(&self) -> Option<Arc<MethodRef>> {
        self.reference.as_method().map(|value| Arc::clone(value))
    }
}

fn lookup_method_ref(
    pool: &ConstantPool,
    kind: u8,
    index: u16,
    major_version: u16
) -> Arc<MethodRef> {
    let reference = pool.get_method_ref(index as usize)
        .expect(&format!("Invalid method handle! Expected method ref index {} to be in constant pool!", index));
    validate_method_ref(&reference, kind, major_version);
    reference
}

fn validate_method_ref(reference: &MethodRef, kind: u8, major_version: u16) {
    if kind == JVM_REF_INVOKE_VIRTUAL || kind == JVM_REF_NEW_INVOKE_SPECIAL {
        assert!(!reference.is_interface, "Invalid method handle! Expected method reference to not \
            be an interface method reference!");
    }
    if (kind == JVM_REF_INVOKE_STATIC || kind == JVM_REF_INVOKE_SPECIAL) && major_version < JAVA_VERSION_8 {
        assert!(!reference.is_interface, "Invalid method handle! Expected method reference to not \
            be an interface method reference!");
    }
    if kind == JVM_REF_INVOKE_INTERFACE {
        assert!(reference.is_interface, "Invalid method handle! Expected method reference to be \
            an interface method reference!");
    }
    let name = reference.name();
    if (kind >= JVM_REF_INVOKE_VIRTUAL && kind <= JVM_REF_INVOKE_SPECIAL) || kind == JVM_REF_INVOKE_INTERFACE {
        assert_ne!(name, JVM_CLASS_INITIALIZER_NAME, "Invalid method reference! invokeVirtual, \
            invokeStatic, invokeSpecial, and invokeInterface references cannot reference a static \
            initializer ({})!", JVM_CLASS_INITIALIZER_NAME);
        assert_ne!(name, JVM_OBJECT_INITIALIZER_NAME, "Invalid method reference! invokeVirtual, \
            invokeStatic, invokeSpecial, and invokeInterface references cannot reference a \
            constructor ({})!", JVM_OBJECT_INITIALIZER_NAME);
    }
    if kind == JVM_REF_NEW_INVOKE_SPECIAL {
        assert_eq!(name, JVM_OBJECT_INITIALIZER_NAME, "Invalid method reference! newInvokeSpecial \
            references must reference a constructor ({})!", JVM_OBJECT_INITIALIZER_NAME);
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum MethodHandleRef {
    Field(Arc<FieldRef>),
    Method(Arc<MethodRef>)
}

macro_rules! impl_element_ref {
    ($T:ident) => {
        impl $T {
            pub fn class(&self) -> &Class {
                &self.class
            }
        }
    }
}

#[derive(Debug, Nameable, FieldDescribable)]
pub struct FieldRef {
    class: Arc<Class>,
    name: IStr,
    descriptor: FieldDescriptor
}

impl FieldRef {
    pub const fn new(class: Arc<Class>, name: IStr, descriptor: FieldDescriptor) -> Self {
        FieldRef { class, name, descriptor }
    }
}

impl_element_ref!(FieldRef);

#[derive(Debug, Nameable, MethodDescribable)]
pub struct MethodRef {
    class: Arc<Class>,
    name: IStr,
    descriptor: MethodDescriptor,
    is_interface: bool
}

impl MethodRef {
    pub const fn new(
        class: Arc<Class>,
        name: IStr,
        descriptor: MethodDescriptor,
        is_interface: bool
    ) -> Self {
        MethodRef { class, name, descriptor, is_interface }
    }

    pub fn is_interface(&self) -> bool {
        self.is_interface
    }
}

impl_element_ref!(MethodRef);
