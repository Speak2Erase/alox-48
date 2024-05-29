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

use darling::{
    util::{Flag, Override},
    FromDeriveInput,
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, LitStr, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(marshal))]
#[darling(supports(struct_any, enum_any))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<VariantReciever, FieldReciever>,

    alox_crate_path: Option<Path>,

    class: Option<String>,
    deny_unknown_fields: Flag,
    enforce_class: Flag,
    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,
    #[darling(rename = "from")]
    from_type: Option<Type>,
    #[darling(rename = "try_from")]
    try_from_type: Option<Type>,
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
    skip_deserializing: Flag,

    #[darling(rename = "deserialize_with")]
    deserialize_with_fn: Option<Path>,
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

pub fn derive_inner(input: syn::DeriveInput) -> TokenStream {
    let reciever = match TypeReciever::from_derive_input(&input) {
        Ok(reciever) => reciever,
        Err(e) => return e.write_errors(),
    };
    let deserialization_impl = parse_reciever(&reciever);

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
            #alox_crate_path;
            use _alox_48::{
                ArrayAccess, Deserialize, DeserializerTrait, DeError, HashAccess,
                InstanceAccess, IvarAccess, Visitor, VisitorOption, DeResult, Sym,
                de::Unexpected,
            };
            #deserialization_impl
        };
    }
}

fn parse_reciever(reciever: &TypeReciever) -> TokenStream {
    let ty = &reciever.ident;

    if reciever.try_from_type.is_some() && reciever.from_type.is_some() {
        return quote! { compile_error!("Cannot specify both `from` and `try_from`") };
    }

    if let Some(into_ty) = reciever.from_type.as_ref() {
        return quote! {
            #[automatically_derived]
            impl<'de> Deserialize<'de> for #ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
                where
                    D: DeserializerTrait<'de>
                {
                    #into_ty::deserialize(deserializer).map(Into::into)
                }
            }
        };
    }

    if let Some(try_into_ty) = reciever.try_from_type.as_ref() {
        return quote! {
            #[automatically_derived]
            impl<'de> Deserialize<'de> for #ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
                where
                    D: DeserializerTrait<'de>
                {
                    #try_into_ty::deserialize(deserializer)?.try_into().map_err(DeError::custom)
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

    let (field_const, field_lets, field_match, instantiate_fields): ParseUnpack = fields
        .iter()
        .map(|field| parse_field(reciever.default_fn.is_some(), field))
        .multiunzip();

    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());
    let enforce_class = if reciever.enforce_class.is_present() {
        let classname_lit = LitStr::new(&classname, ty.span());
        quote! {
            if class != Sym::new(#classname_lit) {
                return Err(DeError::invalid_type(Unexpected::Class(class), &self));
            }
        }
    } else {
        quote! {}
    };

    let unknown_fields = if reciever.deny_unknown_fields.is_present() {
        quote! {
            _f => return Err(DeError::unknown_field(Sym::new(_f), __FIELDS))
        }
    } else {
        quote! {
            _ => {}
        }
    };
    let default = reciever.default_fn.as_ref().map(|d| {
        if let Some(p) = d.as_ref().explicit() {
            quote! { let default = #p(); }
        } else {
            quote! { let default = <#ty as Default>::default(); }
        }
    });

    let expecting_text = reciever
        .expecting
        .clone()
        .unwrap_or_else(|| format!("an instance of {classname}",));
    let expecting_lit = LitStr::new(&expecting_text, ty.span());

    quote! {
        #[automatically_derived]
        impl<'de> Deserialize<'de> for #ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
            where
                D: DeserializerTrait<'de>
            {
                const __FIELDS: &[&Sym] = &[
                    #( #field_const ),*
                ];

                struct __Visitor;

                impl<'de> Visitor<'de> for __Visitor {
                    type Value = #ty;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str(#expecting_lit)
                    }

                    fn visit_object<A>(self, class: &'de Sym, mut _instance_variables: A) -> Result<Self::Value, DeError>
                    where
                        A: IvarAccess<'de>,
                    {
                        #enforce_class

                        #( #field_lets );*

                        while let Some(f) = _instance_variables.next_ivar()? {
                            match f.to_rust_field_name().unwrap_or(f).as_str() {
                                #( #field_match ),*
                                #unknown_fields
                            }
                        }

                        #default

                        Ok(#ty {
                            #( #instantiate_fields ),*
                        })
                    }
                }

                deserializer.deserialize(__Visitor)
            }
        }
    }
}

fn parse_newtype_struct(reciever: &TypeReciever) -> TokenStream {
    let ty = reciever.ident.clone();

    let classname = reciever.class.clone().unwrap_or_else(|| ty.to_string());
    let enforce_class = if reciever.enforce_class.is_present() {
        let classname_lit = LitStr::new(&classname, ty.span());
        quote! {
            if class != Sym::new(#classname_lit) {
                return Err(DeError::invalid_type(Unexpected::Class(class), &self));
            }
        }
    } else {
        quote! {}
    };

    let expecting_text = reciever
        .expecting
        .clone()
        .unwrap_or_else(|| format!("an instance of {classname}"));
    let expecting_lit = LitStr::new(&expecting_text, ty.span());

    quote! {
        #[automatically_derived]
        impl<'de> Deserialize<'de> for #ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
            where
                D: DeserializerTrait<'de>
            {

                struct __Visitor;

                impl<'de> Visitor<'de> for __Visitor {
                    type Value = #ty;

                    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value, DeError>
                    where
                        D: DeserializerTrait<'de>
                    {
                        #enforce_class

                        Ok(#ty(Deserialize::deserialize(deserializer)?))
                    }

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str(#expecting_lit)
                    }
                }

                deserializer.deserialize(__Visitor)
            }
        }
    }
}

type ParseTuple<T> = (
    // const field
    T,
    // let field
    T,
    // match field
    T,
    // instantiate field
    T,
);
type ParseResult = ParseTuple<TokenStream>;
type ParseUnpack = ParseTuple<Vec<TokenStream>>;

fn parse_field(reciever_has_default: bool, field: &FieldReciever) -> ParseResult {
    let field_ident = field.ident.as_ref().unwrap();
    let field_str = format!("__field_{field_ident}");
    let field_ty = field.ty.clone();
    let let_var_ident = Ident::new(&field_str, field_ident.span());

    let field_lit = field
        .rename
        .as_ref()
        .map(|r| r.value())
        .unwrap_or_else(|| field_ident.to_string());
    let field_lit_str = LitStr::new(&field_lit, field_ident.span());
    let const_sym = quote! { Sym::new(#field_lit_str) };

    let let_field = quote! { let mut #let_var_ident: Option<#field_ty> = None; };

    let deserialize_with_fn = field.deserialize_with_fn.clone().or_else(|| {
        field.with_module.clone().map(|mut module| {
            module
                .segments
                .push(Ident::new("deserialize_with", module.span()).into());
            module
        })
    });

    let skip = field.skip.is_present() || field.skip_deserializing.is_present();

    let match_field = if skip {
        quote! {
            #field_lit_str => {
                // skipped
            }
        }
    } else if let Some(with_fn) = deserialize_with_fn {
        quote! {
            #field_lit_str => {
                struct __DeserializeField(#field_ty);
                impl<'de> Deserialize<'de> for __DeserializeField {
                    fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
                    where
                        D: DeserializerTrait<'de>
                    {
                        #with_fn(deserializer).map(Self)
                    }
                }
                let __v = _instance_variables.next_value::<__DeserializeField>()?.0;
                #let_var_ident = Some(__v);
            }
        }
    } else {
        quote! {
            #field_lit_str => {
                let __v = _instance_variables.next_value::<#field_ty>()?;
                #let_var_ident = Some(__v);
            }
        }
    };

    let instantiate_field = if let Some(default_fn) = field.default_fn.as_ref() {
        if let Some(p) = default_fn.as_ref().explicit() {
            quote! { #field_ident: #let_var_ident.unwrap_or(#p()) }
        } else {
            quote! { #field_ident: #let_var_ident.unwrap_or(<#field_ty as Default>::default()) }
        }
    } else if reciever_has_default {
        quote! {
            #field_ident: #let_var_ident.unwrap_or(default.#field_ident)
        }
    } else {
        quote! {
            #field_ident: #let_var_ident.ok_or_else(|| {
                DeError::missing_field(Sym::new(#field_lit_str))
            })?
        }
    };

    (const_sym, let_field, match_field, instantiate_field)
}

fn parse_enum(_reciever: &TypeReciever, _variants: &[VariantReciever]) -> TokenStream {
    quote! {
        compile_error!("Derive macro does not currently automatic deserialize impls for enums!")
    }
}
