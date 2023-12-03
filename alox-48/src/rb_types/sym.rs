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

use crate::Symbol;

#[repr(transparent)]
pub struct Sym(pub(crate) str);

impl Sym {
    pub fn new(str: &str) -> &Self {
        // SAFETY: Sym is just a wrapper of str and is repr(transparent) so they have identical layouts. This should be safe.
        //
        // double checked with miri.
        // as far as I am aware (especially since this is what the stdlib does) this is only way to convert to a dst like we want.
        unsafe { &*(str as *const str as *const Sym) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_ivar(&self) -> bool {
        self.0.starts_with('@')
    }
}

impl AsRef<str> for Sym {
    fn as_ref(&self) -> &str {
        &self.0
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

impl std::fmt::Display for &Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\"{}\"", &self.0))
    }
}

impl std::fmt::Debug for &Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sym").field(&&self.0).finish()
    }
}

impl From<&str> for &Sym {
    fn from(value: &str) -> Self {
        Sym::new(value)
    }
}

impl From<&Sym> for &str {
    fn from(value: &Sym) -> Self {
        &value.0
    }
}

impl PartialEq<str> for &Sym {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for &Sym {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Symbol> for &Sym {
    fn eq(&self, other: &Symbol) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<Sym> for &Sym {
    fn eq(&self, other: &Sym) -> bool {
        self.0.eq(&other.0)
    }
}
