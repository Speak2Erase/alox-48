// Copyright (C) 2022 Lily Lyons
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

use serde::de::Error as SerdeError;
use serde::de::{MapAccess, Unexpected, Visitor};

use crate::value::RbFields;

/// This trait is responsible for handling types from ruby's marshal format that do not map well to serde's data model.
/// Most functions here forward to the closest serde function by default.
/// Some functions, such as [`VisitorExt::visit_userdata`] by default do **not** forward to anything at all and instead error.
///
/// This trait only works with alox48.
pub trait VisitorExt<'de>: Visitor<'de> {
    /// For deserializing objects serialized via `_dump` in ruby.
    /// The class name is passed in as well as the relevant data.
    ///
    /// Errors by default.
    fn visit_userdata<E>(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError;

    /// For deserializing ruby objects in general.
    /// It's different to how deserializing structs normally works in serde, as you get a class name.
    ///
    /// Forwards to [`Visitor::visit_map`] by default.
    fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>;

    /// For deserializing ruby symbols.
    /// Only exists to distinguish between strings and symbols.
    ///
    /// Forwards to [`Visitor::visit_borrowed_str`] by default.
    fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError;

    /// For deserializing ruby strings which may or may not be utf8.
    /// You will also get any extra fields attached to the string, like the encoding (as that is a thing in ruby)
    ///
    /// By default, it uses [`String::from_utf8_lossy`] and matches the resulting [`std::borrow::Cow`] like this:
    /// ```
    /// match str {
    ///     std::borrow::Cow::Borrowed(str) => self.visit_borrowed_str(str),
    ///     std::borrow::Cow::Owned(str) => self.visit_string(str),
    /// }
    /// ```
    fn visit_ruby_string<E>(self, str: &'de [u8], fields: RbFields) -> Result<Self::Value, E>
    where
        E: SerdeError;
}

impl<'de, T> VisitorExt<'de> for T
where
    T: Visitor<'de>,
{
    default fn visit_userdata<E>(self, _class: &'de str, _data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Err(SerdeError::invalid_type(
            Unexpected::Other("userdata"),
            &self,
        ))
    }

    default fn visit_object<A>(self, _class: &'de str, fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.visit_map(fields)
    }

    default fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        self.visit_borrowed_str(sym)
    }

    default fn visit_ruby_string<E>(
        self,
        str: &'de [u8],
        _fields: RbFields,
    ) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        let str = String::from_utf8_lossy(str);

        match str {
            std::borrow::Cow::Borrowed(str) => self.visit_borrowed_str(str),
            std::borrow::Cow::Owned(str) => self.visit_string(str),
        }
    }
}

/// Default implementation for VisitorExt.
impl<'de> VisitorExt<'de> for serde::de::IgnoredAny {
    fn visit_userdata<E>(self, _class: &'de str, _data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_object<A>(self, _class: &'de str, _fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_symbol<E>(self, _sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_ruby_string<E>(self, _str: &'de [u8], _fields: RbFields) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }
}
