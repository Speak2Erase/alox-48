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

/// Derive `Deserialize` for a struct.
///
/// Does not currently support enums.
///
/// Type attributes:
/// - `alox_crate_path`: The path to the alox-48 crate.
/// - `class`: Override the class that the class enforcer checks for. By default, the class of structs is the struct name.
/// - `deny_unknown_fields`: If set, the deserializer will error if it encounters a field not in the struct.
/// - `enforce_class`: If set, the deserializer will enforce that the class matches.
/// - `default`: The default function to use for a field. Leave empty to use `Default::default`.
/// - `from`: Deserialize from a different type. That type must implement `Deserialize`.
/// - `try_from`: Deserialize from a different type. That type must implement `TryFrom`, and its error type must implement `Display`.
/// - `expecting`: The error message to use if deserialization fails.
///
/// Field attributes:
/// - `rename`: Rename the field.
/// - `default`: The default function to use for a field. Leave empty to use `Default::default`.
/// - `skip` or `skip_deserializing`: Skip deserializing the field.
/// - `deserialize_with`: Use a custom function to deserialize the field. That function must have the signature `fn(impl Deserializer<'de>) -> Result<T, DeError>`.
/// - `with`: Like `deserialize_with`, but the function is in a module.
#[proc_macro_derive(Deserialize, attributes(marshal))]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    de::derive_inner(&input).into()
}

/// Derive `Serialize` for a struct.
///
/// Does not currently support enums.
///
/// Type attributes:
/// - `alox_crate_path`: The path to the alox-48 crate.
/// - `class`: Override the class that this type is serialized as. By default, the class is the struct name.
/// - `into`: Serialize to a different type. That type must implement `Serialize`, and `Self` must impl `Into<T> + Clone`.
/// - `try_into`: Serialize to a different type. That type must implement `Serialize`, and Self must impl `TryInto<T> + Clone`.
///
/// Field attributes:
/// - `rename`: Rename the field.
/// - `skip` or `skip_serializing`: Skip serializing the field.
/// - `serialize_with`: Use a custom function to serialize the field. That function must have the signature `fn(&T, impl Serializer) -> Result<S::Ok, SerError>`.
/// - `with`: Like `serialize_with`, but the function is in a module.
#[proc_macro_derive(Serialize, attributes(marshal))]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    ser::derive_inner(&input).into()
}

// TODO tests
