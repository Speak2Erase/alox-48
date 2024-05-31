#![warn(rust_2018_idioms, clippy::all, clippy::pedantic)]
use proc_macro::TokenStream;

use syn::DeriveInput;

mod de;
mod ser;
mod util;

use darling::{
    util::{Flag, Override},
    FromDeriveInput,
};
use syn::{Ident, LitStr, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(marshal))]
#[darling(supports(struct_any, enum_any))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<VariantReciever, FieldReciever>,

    generics: syn::Generics,

    alox_crate_path: Option<Path>,

    class: Option<String>,

    deny_unknown_fields: Flag,
    enforce_class: Flag,

    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,
    #[darling(rename = "from")]
    from_type: Option<Type>,
    #[darling(rename = "into")]
    into_type: Option<Type>,
    #[darling(rename = "try_from")]
    try_from_type: Option<Type>,
    #[darling(rename = "try_into")]
    try_into_type: Option<Type>,

    expecting: Option<String>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(marshal))]
struct FieldReciever {
    ident: Option<Ident>,
    ty: Type,

    rename: Option<LitStr>,

    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,

    skip: Flag,
    skip_serializing: Flag,
    skip_deserializing: Flag,
    byte_string: Flag,

    #[darling(rename = "deserialize_with")]
    deserialize_with_fn: Option<Path>,
    #[darling(rename = "serialize_with")]
    serialize_with_fn: Option<Path>,
    #[darling(rename = "with")]
    with_module: Option<Path>,
}

#[allow(dead_code)]
#[derive(Debug, darling::FromVariant)]
struct VariantReciever {
    ident: Ident,
    fields: darling::ast::Fields<FieldReciever>,

    transparent: Flag,
    class: Option<String>,
}

#[proc_macro_derive(Deserialize, attributes(marshal))]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    de::derive_inner(&input).into()
}

#[proc_macro_derive(Serialize, attributes(marshal))]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    ser::derive_inner(&input).into()
}

// TODO tests
