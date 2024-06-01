// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use darling::FromDeriveInput;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, LitInt, LitStr};

use super::{FieldReciever, TypeReciever, VariantReciever};

pub fn derive_inner(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let reciever = match TypeReciever::from_derive_input(input) {
        Ok(reciever) => reciever,
        Err(e) => return e.write_errors(),
    };
    let serialization_impl = parse_reciever(&reciever);

    let alox_crate_path = reciever.alox_crate_path.as_ref().map_or_else(
        || {
            quote! { extern crate alox_48 as _alox_48; }
        },
        |path| {
            quote! { use #path as _alox_48; }
        },
    );

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, non_snake_case, unused_attributes, unused_qualifications, no_effect_underscore_binding, non_camel_case_types)]
        const _: () = {
            #alox_crate_path
            use _alox_48::{
                Serialize, SerializeArray, SerializeHash, SerializeIvars, SerializerTrait, ser::Error as SerError, Sym
            };
            #serialization_impl
        };
    }
}

fn parse_reciever(reciever: &TypeReciever) -> TokenStream {
    let ty = &reciever.ident;

    if reciever.try_into_type.is_some() && reciever.into_type.is_some() {
        return quote! { compile_error!("Cannot specify both `from` and `try_from`") };
    }

    if let Some(into_ty) = reciever.into_type.as_ref() {
        return quote! {
            #[automatically_derived]
            impl Serialize for #ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, SerError>
                    where S: SerializerTrait
                {
                    <Self as Into<#into_ty>>::into(self.clone()).serialize(serializer)
                }
            }
        };
    }

    if let Some(try_into_ty) = reciever.try_into_type.as_ref() {
        return quote! {
            #[automatically_derived]
            impl Serialize for #ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, SerError>
                    where S: SerializerTrait
                {
                    <Self as TryInto<#try_into_ty>>::into(self.clone()).map_err(SerError::custom)?.serialize(serializer)
                }
            }
        };
    }

    match &reciever.data {
        darling::ast::Data::Enum(e) => parse_enum(reciever, e),
        darling::ast::Data::Struct(f) => parse_struct(reciever, f),
    }
}

fn parse_struct(
    reciever: &TypeReciever,
    fields: &darling::ast::Fields<FieldReciever>,
) -> TokenStream {
    // handle tuple and newtype structs
    if fields.iter().next().is_some_and(|f| f.ident.is_none()) {
        return if fields.len() > 1 {
            quote! {
                compile_error!("Derive macro does not currently automatic deserialize impls for tuple structs!")
            }
        } else {
            parse_newtype_struct(reciever)
        };
    }

    let ty = reciever.ident.clone();
    let impl_lifetimes = reciever.generics.lifetimes();
    let ty_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);

    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());

    let field_impls = fields
        .iter()
        .filter(|field| !(field.skip.is_present() || field.skip_serializing.is_present()))
        .map(parse_field)
        .collect_vec();
    let fields_len = format!("{}_usize", field_impls.len());
    let fields_len = LitInt::new(&fields_len, ty.span());

    quote! {
        #[automatically_derived]
        impl < #( #impl_lifetimes ),* > Serialize for #ty < #( #ty_lifetimes ),* > {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, SerError>
                where S: SerializerTrait
            {
                let mut serialize_ivars = serializer.serialize_object(&Sym::new(#classname), #fields_len)?;
                #(#field_impls)*
                serialize_ivars.end()
            }
        }
    }
}

fn parse_newtype_struct(reciever: &TypeReciever) -> TokenStream {
    let ty = reciever.ident.clone();
    let impl_lifetimes = reciever.generics.lifetimes();
    let ty_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);

    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());

    quote! {
        #[automatically_derived]
        impl < #( #impl_lifetimes ),* > Serialize for #ty < #( #ty_lifetimes ),* > {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, SerError>
                where S: SerializerTrait
            {
                let class = Sym::new(#classname);
                serializer.serialize_user_class(class, &self.0)
            }
        }
    }
}

type ParseResult = TokenStream;
fn parse_field(field: &FieldReciever) -> ParseResult {
    let field_ident = field.ident.as_ref().unwrap();
    let field_ty = field.ty.clone();

    let serialize_str = field
        .rename
        .as_ref()
        .map_or_else(|| field_ident.to_string(), syn::LitStr::value);
    let serialize_str = LitStr::new(&serialize_str, field_ident.span());

    let serialize_with_fn = field.serialize_with_fn.clone().or_else(|| {
        field.with_module.clone().map(|mut module| {
            module
                .segments
                .push(Ident::new("serialize_with", module.span()).into());
            module
        })
    });

    if let Some(with_fn) = serialize_with_fn {
        quote! {
            {
                struct __SerializeField<'a>(&'a #field_ty);
                impl Serialize for __SerializeField<'_> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, SerError>
                    where S: SerializerTrait
                    {
                        #with_fn(self.0, serializer)
                    }
                }
                let field = Sym::new(#serialize_str).to_ivar();
                serialize_ivars.serialize_entry(&field, &__SerializeField(&self.#field_ident))?;
            }
        }
    } else if field.byte_string.is_present() {
        quote! {
            let field = Sym::new(#serialize_str).to_ivar();
            let ty = _alox_48::SerializeByteString(self.#field_ident.as_ref());
            serialize_ivars.serialize_entry(&field, &ty)?;
        }
    } else {
        quote! {
            let field = Sym::new(#serialize_str).to_ivar();
            serialize_ivars.serialize_entry(&field, &self.#field_ident)?;
        }
    }
}

fn parse_enum(_reciever: &TypeReciever, _variants: &[VariantReciever]) -> TokenStream {
    quote! {
        compile_error!("Derive macro does not currently automatic deserialize impls for enums!")
    }
}
