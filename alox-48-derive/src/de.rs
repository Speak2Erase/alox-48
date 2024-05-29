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
use quote::{quote, quote_spanned};
use syn::{Ident, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(marshal))]
#[darling(supports(struct_any, enum_any))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<VariantReciever, FieldReciever>,

    class: Option<String>,
    deny_unknown_fields: Flag,
    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,
    #[darling(rename = "from")]
    from_type: Option<Type>,
    #[darling(rename = "try_from")]
    try_from_type: Option<Type>,
    expecting: Option<String>,
}

#[derive(Debug, darling::FromField)]
struct FieldReciever {
    ident: Option<Ident>,
    ty: Type,

    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,

    skip: Flag,
    skip_deserializing: Flag,

    #[darling(rename = "deserialize_with")]
    deserialize_with_fn: Option<Path>,
    #[darling(rename = "with")]
    with_module: Option<Path>,
}

#[derive(Debug, darling::FromVariant)]
struct VariantReciever {
    ident: Ident,
    fields: darling::ast::Fields<FieldReciever>,

    transparent: Flag,
    class: Option<String>,
}

pub fn derive_inner(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let reciever = TypeReciever::from_derive_input(&input).unwrap();
    let deserialization_impl = parse_reciever(&reciever);

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, non_snake_case, unused_attributes, unused_qualifications, no_effect_underscore_binding, non_camel_case_types)]
        const _: () = {
            extern crate alox_48 as _alox_48;
            use _alox_48::{
                ArrayAccess, Deserialize, DeserializerTrait, DeError, HashAccess,
                InstanceAccess, IvarAccess, Visitor, VisitorOption, DeResult, Sym,
                de::Unexpected,
            };
            #deserialization_impl
        };
    }
}

fn parse_reciever(reciever: &TypeReciever) -> proc_macro2::TokenStream {
    let ty = &reciever.ident;

    if reciever.try_from_type.is_some() && reciever.from_type.is_some() {
        return quote_spanned! {
            reciever.ident.span() => compile_error!("Cannot specify both `from` and `try_from`")
        };
    }

    if let Some(into_ty) = reciever.from_type.as_ref() {
        return quote::quote! {
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
        return quote::quote! {
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
) -> proc_macro2::TokenStream {
    let ty = reciever.ident.clone();

    let field_const = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let literal = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
        quote! { Sym::new(#literal) }
    });
    let field_lets = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();

        let field_str = format!("__field_{field_ident}");
        let var_ident = syn::Ident::new(&field_str, field_ident.span());

        let field_ty = field.ty.clone();
        quote! {
            let mut #var_ident: Option<#field_ty> = None;
        }
    });
    let field_match = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_lit_str = syn::LitStr::new(&field_ident.to_string(), field_ident.span());

        let field_str = format!("__field_{field_ident}");
        let var_ident = syn::Ident::new(&field_str, field_ident.span());

        let field_ty = field.ty.clone();
        quote! {
            #field_lit_str => {
                let __v = _instance_variables.next_value::<#field_ty>()?;
                #var_ident = Some(__v);
            }
        }
    });
    let instantiate_fields = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let field_lit_str = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
        let field_ty = field.ty.clone();

        let field_str = format!("__field_{field_ident}");
        let var_ident = syn::Ident::new(&field_str, field_ident.span());

        if let Some(default_fn) = field.default_fn.as_ref() {
            if let Some(p) = default_fn.as_ref().explicit() {
                quote! { #field_ident: #p(); }
            } else {
                quote! { #field_ident: <#field_ty as Default>::default(); }
            }
        } else if reciever.default_fn.is_some() {
            quote! {
                #field_ident: #var_ident.unwrap_or(default.#field_ident)
            }
        } else {
            quote! {
                #field_ident: #var_ident.ok_or_else(|| {
                    DeError::missing_field(Sym::new(#field_lit_str))
                })?
            }
        }
    });
    let unknown_fields = if reciever.deny_unknown_fields.is_present() {
        quote::quote! {
            _f => return Err(DeError::unknown_field(Sym::new(_f), __FIELDS))
        }
    } else {
        quote::quote! {
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

    let expecting_text = reciever.expecting.clone().unwrap_or_else(|| {
        format!(
            "an instance of {}",
            reciever.class.clone().unwrap_or_else(|| ty.to_string())
        )
    });
    let expecting_lit = syn::LitStr::new(&expecting_text, ty.span());

    quote::quote! {
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

fn parse_enum(reciever: &TypeReciever, variants: &[VariantReciever]) -> proc_macro2::TokenStream {
    todo!()
}
