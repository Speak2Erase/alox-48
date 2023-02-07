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

use std::{
    borrow::{Borrow, Cow},
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    string::FromUtf8Error,
};

use super::{RbString, Symbol};

impl RbString {
    pub fn encoding(&self) -> Option<&crate::Value> {
        self.fields.get("E")
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }

    pub fn to_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data)
    }
}

impl Debug for RbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RbString")
            .field("data", &self.to_string_lossy())
            .field("fields", &self.fields)
            .finish()
    }
}

impl Deref for RbString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for RbString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> PartialEq<T> for RbString
where
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &T) -> bool {
        self.data.as_slice() == other.as_ref()
    }
}

impl Borrow<[u8]> for RbString {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl<T> PartialEq<T> for Symbol
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl Deref for Symbol {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(":{}", self.0))
    }
}

impl DerefMut for Symbol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        self
    }
}
