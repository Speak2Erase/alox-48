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
// not sure how to handle newtypes and unit structs
// maybe userclass would make sense but that would be a bit of a stretch
// do want to add support for enums in the future though!
#[darling(supports(struct_named))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldReciever>,

    class: Option<String>,
    deny_unknown_fields: Flag,
    #[darling(rename = "default")]
    default_fn: Option<Override<Path>>,
    #[darling(rename = "from")]
    from_type: Option<Type>,
    #[darling(rename = "try_from")]
    try_from_type: Option<Type>,
    #[darling(rename = "into")]
    into_type: Option<Type>,
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

pub fn derive_inner(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let reciever = TypeReciever::from_derive_input(&input).unwrap();
    let deserialization_impl = parse_fields(reciever);

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

fn parse_fields(reciever: TypeReciever) -> proc_macro2::TokenStream {
    let ty = reciever.ident;
    let fields = reciever.data.take_struct().unwrap();

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

        let field_str = format!("__field_{field_ident}");
        let var_ident = syn::Ident::new(&field_str, field_ident.span());

        quote! {
            #field_ident: #var_ident.ok_or_else(|| {
                DeError::missing_field(Sym::new(#field_lit_str))
            })?
        }
    });

    let expecting_text = format!(
        "an instance of {}",
        reciever.class.unwrap_or_else(|| ty.to_string())
    );
    let expecting_lit = syn::LitStr::new(&expecting_text, ty.span());

    quote::quote! {
        #[automatically_derived]
        impl<'de> Deserialize<'de> for #ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, DeError>
            where
                D: DeserializerTrait<'de>
            {
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
                                _ => {}
                            }
                        }

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
