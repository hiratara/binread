#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]

mod codegen;
mod parser;

use codegen::generate_impl;
use parser::{is_binread_attr, Input};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    derive_binread_internal(parse_macro_input!(input as DeriveInput)).into()
}

fn derive_binread_internal(input: DeriveInput) -> proc_macro2::TokenStream {
    let binread_input = Input::from_input(&input);
    generate_impl(&input, &binread_input)
}

#[proc_macro_attribute]
pub fn derive_binread(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    let binread_input = Input::from_input(&derive_input);
    let generated_impl = generate_impl(&derive_input, &binread_input);
    let binread_input = binread_input.ok();

    clean_struct_attrs(&mut derive_input.attrs);

    match &mut derive_input.data {
        syn::Data::Struct(input_struct) => {
            clean_field_attrs(&binread_input, 0, &mut input_struct.fields);
        }
        syn::Data::Enum(input_enum) => {
            for (index, variant) in input_enum.variants.iter_mut().enumerate() {
                clean_struct_attrs(&mut variant.attrs);
                clean_field_attrs(&binread_input, index, &mut variant.fields);
            }
        }
        syn::Data::Union(union) => {
            for field in union.fields.named.iter_mut() {
                clean_struct_attrs(&mut field.attrs);
            }
        }
    }

    quote!(
        #derive_input
        #generated_impl
    )
    .into()
}

fn clean_field_attrs(
    binread_input: &Option<Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let Some(binread_input) = binread_input {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if binread_input.is_temp_field(variant_index, index) {
                    None
                } else {
                    let mut value = value.clone();
                    clean_struct_attrs(&mut value.attrs);
                    Some(value)
                }
            })
            .collect();
    }
}

fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binread_attr(attr));
}

#[cfg(test)]
mod tests {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    use std::{env, fs};

    #[test]
    fn derive_code_coverage() {
        let derive_tests_folder = env::current_dir()
            .unwrap()
            .join("..")
            .join("binread")
            .join("tests")
            .join("derive");

        let mut run_success = true;
        for entry in fs::read_dir(derive_tests_folder).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            let file = fs::File::open(entry.path()).unwrap();
            let is_ok =
                emulate_derive_expansion_fallible(file, "BinRead", super::derive_binread_internal)
                    .is_ok();
            run_success &= is_ok;
        }

        assert!(run_success)
    }
}
