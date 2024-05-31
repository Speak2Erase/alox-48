// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use darling::{util::Override, FromDeriveInput};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, LitStr};

use super::{FieldReciever, TypeReciever, VariantReciever};

pub fn derive_inner(input: &syn::DeriveInput) -> TokenStream {
    let reciever = match TypeReciever::from_derive_input(input) {
        Ok(reciever) => reciever,
        Err(e) => return e.write_errors(),
    };
    let deserialization_impl = parse_reciever(&reciever);

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
    if reciever
        .generics
        .lifetimes()
        .any(|l| l.lifetime.ident == "de")
    {
        return quote! {
            compile_error!("Cannot use 'de as a lifetime in the Deserialize derive macro")
        };
    }

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

#[allow(clippy::too_many_lines)]
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
    let ty_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let ty_lifetimes = quote! { <#( #ty_lifetimes ),*> };
    let visitor_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let visitor_lifetimes = quote! { <'de, #( #visitor_lifetimes ),*> };

    let lifetimes_iter = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let de_lifetime = quote! { 'de: #( #lifetimes_iter )+* };
    // i have no idea why we need to specify this here but rust gets *really* unhappy if we don't
    let lifetimes_iter = reciever.generics.lifetimes().cloned().map(|mut l| {
        l.bounds.push(syn::Lifetime::new("'de", l.span()));
        l
    });
    let impl_lifetimes = quote! { <#de_lifetime, #( #lifetimes_iter ),*> };

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
        impl #impl_lifetimes Deserialize<'de> for #ty #ty_lifetimes {
            fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
            where
                D: DeserializerTrait<'de>
            {
                const __FIELDS: &[&Sym] = &[
                    #( #field_const ),*
                ];

                struct __Visitor #impl_lifetimes {
                    _marker: std::marker::PhantomData<#ty #ty_lifetimes >,
                    _phantom: std::marker::PhantomData<&'de ()>,
                }

                impl #impl_lifetimes Visitor<'de> for __Visitor #visitor_lifetimes {
                    type Value = #ty #ty_lifetimes;

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

                deserializer.deserialize(__Visitor { _marker: std::marker::PhantomData, _phantom: std::marker::PhantomData })
            }
        }
    }
}

fn parse_newtype_struct(reciever: &TypeReciever) -> TokenStream {
    let ty = reciever.ident.clone();
    let ty_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let ty_lifetimes = quote! { <#( #ty_lifetimes ),*> };
    let visitor_lifetimes = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let visitor_lifetimes = quote! { <'de, #( #visitor_lifetimes ),*> };

    let lifetimes_iter = reciever.generics.lifetimes().map(|l| &l.lifetime);
    let de_lifetime = quote! { 'de: #( #lifetimes_iter )+* };
    // i have no idea why we need to specify this here but rust gets *really* unhappy if we don't
    let lifetimes_iter = reciever.generics.lifetimes().cloned().map(|mut l| {
        l.bounds.push(syn::Lifetime::new("'de", l.span()));
        l
    });
    let impl_lifetimes = quote! { <#de_lifetime, #( #lifetimes_iter ),*> };

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
        impl #impl_lifetimes Deserialize<'de> for #ty #ty_lifetimes {
            fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
            where
                D: DeserializerTrait<'de>
            {

                struct __Visitor #impl_lifetimes {
                    _marker: std::marker::PhantomData<#ty #ty_lifetimes >,
                    _phantom: std::marker::PhantomData<&'de ()>,
                }

                impl #impl_lifetimes Visitor<'de> for __Visitor #visitor_lifetimes {
                    type Value = #ty #ty_lifetimes;

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

                deserializer.deserialize(__Visitor { _marker: std::marker::PhantomData, _phantom: std::marker::PhantomData })
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
        .map_or_else(|| field_ident.to_string(), syn::LitStr::value);
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

    let instantiate_default = match field.default_fn.as_ref() {
        Some(Override::Explicit(default_fn)) => {
            Some(quote! { #let_var_ident.unwrap_or(#default_fn()) })
        }
        Some(_) => Some(quote! { #let_var_ident.unwrap_or(<#field_ty as Default>::default()) }),
        None if skip => Some(quote! { <#field_ty as Default>::default() }),
        None if reciever_has_default => Some(quote! {
            #let_var_ident.unwrap_or(default.#field_ident)
        }),
        None => None,
    };

    let instantiate_field = if let Some(instantiate_default) = instantiate_default {
        quote! {
            #field_ident: #instantiate_default
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
