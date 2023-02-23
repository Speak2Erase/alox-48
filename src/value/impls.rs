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
    borrow::{Borrow, BorrowMut, Cow},
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    string::FromUtf8Error,
};

use super::{RbString, Symbol};

impl RbString {
    #[must_use]
    /// Return the encoding of this string, if it has one.
    pub fn encoding(&self) -> Option<&crate::Value> {
        self.fields.get("E").or_else(|| self.fields.get("encoding"))
    }

    #[must_use]
    /// Uses [`String::from_utf8_lossy`] to convert this string to rust string in a lossy manner.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }

    /// Tries to convert this string to a rust string.
    ///
    /// # Errors
    /// Errors when this string is not valid utf8.
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

impl Display for RbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_lossy())
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
    [u8]: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data.as_slice().eq(other)
    }
}

impl Borrow<[u8]> for RbString {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl BorrowMut<[u8]> for RbString {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl<T> PartialEq<T> for Symbol
where
    String: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
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

impl BorrowMut<str> for Symbol {
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for Symbol {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}
