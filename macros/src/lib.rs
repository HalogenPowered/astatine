extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, ItemStruct, parse_macro_input};

#[proc_macro_derive(Nameable)]
pub fn derive_nameable(input: TokenStream) -> TokenStream {
    gen_impl(input, quote! {
        fn name(&self) -> &str {
            self.name.as_str()
        }
    })
}

#[proc_macro_derive(FieldDescribable)]
pub fn derive_field_describable(input: TokenStream) -> TokenStream {
    gen_impl(input, quote! {
        fn descriptor(&self) -> &crate::utils::descriptors::FieldDescriptor {
            &self.descriptor
        }
    })
}

#[proc_macro_derive(MethodDescribable)]
pub fn derive_method_describable(input: TokenStream) -> TokenStream {
    gen_impl(input, quote! {
        fn descriptor(&self) -> &crate::utils::descriptors::MethodDescriptor {
            &self.descriptor
        }
    })
}

#[proc_macro_derive(Generic)]
pub fn derive_generic(input: TokenStream) -> TokenStream {
    gen_impl(input, quote! {
        fn generic_signature(&self) -> Option<&str> {
            self.generic_signature.as_ref().map(|value| value.as_str())
        }
    })
}

#[proc_macro_derive(Versioned)]
pub fn derive_versioned(input: TokenStream) -> TokenStream {
    gen_impl(input, quote! {
        fn version(&self) -> Option<&str> {
            self.version.as_ref().map(|value| value.as_str())
        }
    })
}

fn gen_impl(input: TokenStream, function: TokenStream2) -> TokenStream {
    let name = parse_macro_input!(input as DeriveInput).ident;
    TokenStream::from(quote! {
        impl #name {
            pub #function
        }
    })
}

#[proc_macro_attribute]
pub fn accessible(attribute: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = input.ident.clone();
    let functions = attribute.to_string()
        .replace("\n", " ")
        .split(", ")
        .map(gen_accessible_function)
        .collect::<Vec<TokenStream2>>();
    TokenStream::from(quote! {
        #input
        impl crate::types::access_flags::Accessible for #name {
            fn flags(&self) -> u16 {
                self.access_flags
            }
        }

        impl #name {
            #(#functions)*
        }
    })
}

fn gen_accessible_function(name: &str) -> TokenStream2 {
    let function_name = format_ident!("is_{}", name);
    let acc_name = format_ident!("ACC_{}", name.to_uppercase());
    quote! {
        pub fn #function_name(&self) -> bool {
            self.flags() & #acc_name != 0
        }
    }
}
