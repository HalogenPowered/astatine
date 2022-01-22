use java_desc::{FieldType, MethodType};

pub trait Nameable {
    fn name(&self) -> &str;
}

macro_rules! impl_nameable {
    ($T:ident) => {
        impl crate::types::utils::Nameable for $T {
            fn name(&self) -> &str {
                self.name.as_str()
            }
        }
    }
}

pub trait FieldTyped {
    fn descriptor(&self) -> &FieldType;
}

macro_rules! impl_field {
    ($T:ident) => {
        impl_nameable!($T);
        impl_generic!($T);

        impl crate::types::utils::FieldTyped for $T {
            fn descriptor(&self) -> &FieldType {
                &self.descriptor
            }
        }
    };
    ($T:ident, marker) => {
        impl_nameable!($T);

        impl crate::types::utils::FieldTyped for $T {
            fn descriptor(&self) -> &FieldType {
                &self.descriptor
            }
        }
    }
}

pub trait MethodTyped {
    fn descriptor(&self) -> &MethodType;
}

pub trait Generic {
    fn generic_signature(&self) -> Option<&str>;
}

macro_rules! impl_generic {
    ($T:ident) => {
        impl crate::types::utils::Generic for $T {
            fn generic_signature(&self) -> Option<&str> {
                self.generic_signature.as_ref().map(|value| value.as_str())
            }
        }
    }
}

pub trait Versioned {
    fn version(&self) -> Option<&str>;
}

macro_rules! impl_versioned {
    ($T:ident) => {
        impl crate::types::utils::Versioned for $T {
            fn version(&self) -> Option<&str> {
                self.version.as_ref().map(|value| value.as_str())
            }
        }
    }
}
