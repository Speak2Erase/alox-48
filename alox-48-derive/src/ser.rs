// Copyright (C) 2023 Lily Lyons
//
// This file is part of alox-48.
//
// alox-48 is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// alox-48 is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with alox-48.  If not, see <http://www.gnu.org/licenses/>.

use darling::{util::Flag, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, LitInt, LitStr, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(marshal))]
#[darling(supports(struct_any, enum_any))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<VariantReciever, FieldReciever>,

    alox_crate_path: Option<Path>,

    class: Option<String>,
    #[darling(rename = "into")]
    into_type: Option<Type>,
    #[darling(rename = "try_into")]
    try_into_type: Option<Type>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(marshal))]
struct FieldReciever {
    ident: Option<Ident>,
    ty: Type,

    rename: Option<LitStr>,

    skip: Flag,
    skip_serializing: Flag,

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

pub fn derive_inner(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let reciever = match TypeReciever::from_derive_input(&input) {
        Ok(reciever) => reciever,
        Err(e) => return e.write_errors(),
    };
    let serialization_impl = parse_reciever(&reciever);

    let alox_crate_path = reciever
        .alox_crate_path
        .as_ref()
        .map(|path| {
            quote! { use #path as _alox_48; }
        })
        .unwrap_or_else(|| {
            quote! { extern crate alox_48 as _alox_48; }
        });

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
    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());

    let field_impls = fields.iter().map(parse_field);
    let fields_len = format!("{}_usize", fields.len());
    let fields_len = LitInt::new(&fields_len, ty.span());

    quote! {
        #[automatically_derived]
        impl Serialize for #ty {
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
    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());

    quote! {
        #[automatically_derived]
        impl Serialize for #ty {
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
    let skip = field.skip.is_present() || field.skip_serializing.is_present();
    if skip {
        return quote! {};
    }

    let field_ident = field.ident.as_ref().unwrap();
    let field_ty = field.ty.clone();

    let serialize_str = field
        .rename
        .as_ref()
        .map(|r| r.value())
        .unwrap_or_else(|| field_ident.to_string());
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
