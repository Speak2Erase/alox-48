#![warn(rust_2018_idioms, clippy::all, clippy::pedantic)]
use proc_macro::TokenStream;

use syn::DeriveInput;

mod de;
mod ser;
mod util;

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
