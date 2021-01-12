pub(crate) mod sanitization;
mod read_options;

use proc_macro2::TokenStream;
use syn::Error;
use crate::meta_attrs::TopLevelAttrs;

pub fn generate(input: &syn::DeriveInput) -> syn::Result<GeneratedCode> {
    if let syn::Data::Union(ref union) = input.data {
        return Err(Error::new(union.union_token.span, "Unions are not supported"));
    }

    let tla = TopLevelAttrs::from_attrs(&input.attrs)?.finalize()?;

    Ok(GeneratedCode {
        arg_type: tla.import.types(),
        read_opt_impl: read_options::generate(input, &tla)?,
    })
}

pub struct GeneratedCode {
    pub read_opt_impl: TokenStream,
    pub arg_type: TokenStream
}