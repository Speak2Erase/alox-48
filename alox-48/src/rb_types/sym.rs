// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::{Borrow, Cow};

use crate::{
    de::Result as DeResult, ser::Result as SerResult, Deserialize, DeserializerTrait, Serialize,
    SerializerTrait, Symbol, Visitor,
};

/// A borrowed ruby symbol.
#[repr(transparent)]
pub struct Sym(pub(crate) str);

impl Sym {
    /// Create a new symbol from a borrowed string.
    pub const fn new(str: &str) -> &Self {
        // SAFETY: Sym is just a wrapper of str and is repr(transparent) so they have identical layouts. This should be safe.
        //
        // double checked with miri.
        // as far as I am aware (especially since this is what the stdlib does) this is only way to convert to a dst like we want.
        unsafe { std::mem::transmute(str) }
    }

    /// Fetch the inner string.
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the inner string is empty.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if the inner string starts with an '@'.
    ///
    /// (aka it's an instance variable name.)
    pub fn is_ivar(&self) -> bool {
        self.0.starts_with('@')
    }

    /// Returns a new symbol with an '@' prepended to the inner string.
    ///
    /// If the inner string already starts with an '@', this will return a borrowed reference to the original symbol.
    pub fn to_ivar(&self) -> Cow<'_, Self> {
        if self.is_ivar() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(Symbol::new(format!("@{}", self.as_str())))
        }
    }

    /// Returns a new symbol with the '@' stripped from the inner string.
    ///
    /// If the inner string does not start with an '@', this will return None.
    pub fn to_rust_field_name(&self) -> Option<&Self> {
        self.0.strip_prefix('@').map(Self::new)
    }

    /// Returns a new owned symbol.
    pub fn to_symbol(&self) -> Symbol {
        self.to_owned()
    }

    /// Returns the length of the inner string.
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

impl Borrow<str> for Sym {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Sym {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Sym {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl ToOwned for Sym {
    type Owned = Symbol;

    fn to_owned(&self) -> Self::Owned {
        Symbol(self.0.to_string())
    }
}

impl Default for &Sym {
    fn default() -> Self {
        Sym::new("")
    }
}

impl std::fmt::Display for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(":{}", &self.0))
    }
}

impl std::fmt::Debug for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sym").field(&&self.0).finish()
    }
}

impl std::hash::Hash for Sym {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> From<&'a str> for &'a Sym {
    fn from(value: &'a str) -> Self {
        Sym::new(value)
    }
}

impl<'a> From<&'a Sym> for &'a str {
    fn from(value: &'a Sym) -> Self {
        &value.0
    }
}

impl PartialEq<str> for Sym {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for Sym {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Symbol> for Sym {
    fn eq(&self, other: &Symbol) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<Sym> for Sym {
    fn eq(&self, other: &Sym) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Sym {}

struct SymVisitor;

impl<'de> Visitor<'de> for SymVisitor {
    type Value = &'de Sym;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a symbol")
    }

    fn visit_symbol(self, symbol: &'de Sym) -> DeResult<Self::Value> {
        // its that easy
        Ok(symbol)
    }
}

impl<'de> Deserialize<'de> for &'de Sym {
    fn deserialize<D>(deserializer: D) -> DeResult<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(SymVisitor)
    }
}

impl Serialize for Sym {
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_symbol(self)
    }
}
