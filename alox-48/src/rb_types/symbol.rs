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

use std::borrow::Borrow;

use super::sym::Sym;

/// A symbol from ruby.
/// It's a newtype around a String, meant to preserve types during (de)serialization.
///
/// When serializing, a [`String`] will be serialized as a String, but a [`Symbol`] will be serialized as a Symbol.
#[derive(Hash, Eq, Default, Clone)]
pub struct Symbol(pub(crate) String);

#[allow(clippy::must_use_candidate)]
impl Symbol {
    pub fn new(string: String) -> Self {
        Self(string)
    }

    /// Get this symbol as a borrowed str.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_sym(&self) -> &Sym {
        Sym::new(&self.0)
    }

    /// Get the length of this symbol.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the string data is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_ivar(&self) -> bool {
        self.0.starts_with('@')
    }

    pub fn as_rust_field_name(&self) -> Option<&Sym> {
        self.0.strip_prefix('@').map(Sym::new)
    }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Symbol> for String {
    fn from(value: Symbol) -> Self {
        value.0
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Symbol").field(&self.0).finish()
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(":{}", self.0))
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for Symbol {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Symbol> for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<Sym> for Symbol {
    fn eq(&self, other: &Sym) -> bool {
        self.0.eq(&other.0)
    }
}

impl Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<Sym> for Symbol {
    fn borrow(&self) -> &Sym {
        self.as_sym()
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Symbol {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<Sym> for Symbol {
    fn as_ref(&self) -> &Sym {
        self.as_sym()
    }
}

impl From<&Sym> for Symbol {
    fn from(value: &Sym) -> Self {
        value.to_owned()
    }
}
