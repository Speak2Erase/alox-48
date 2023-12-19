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
#[darling(supports(struct_named, struct_tuple))]
struct TypeReciever {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldReciever>,

    class: Option<String>,
    #[darling(default)]
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
        #[allow(non_upper_case_globals, non_snake_case, unused_attributes, unused_qualifications, no_effect_underscore_binding)]
        const _: () = {
            extern crate alox_48 as _alox_48;
            use _alox_48::{
                ArrayAccess, Deserialize, DeserializerTrait, DeserializerTrait, DeError, HashAccess,
                InstanceAccess, IvarAccess, Visitor, VisitorOption, DeResult, Sym,
                de::Unexpected,
            };
            #deserialization_impl
        };
    }
}

fn parse_fields(reciever: TypeReciever) -> proc_macro2::TokenStream {}
