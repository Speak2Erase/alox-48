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

use crate::DeError;
use serde::de::Visitor;

mod impls;

/// This trait is responsible for handling types from ruby's marshal format that do not map well to serde's data model.
///
/// This trait only works with alox48.
///
/// ### Default impl
///
/// Most functions here forward to the closest serde function by default.
/// Some functions, such as [`VisitorExt::visit_userdata`] by default do **not** forward to anything at all and instead error.
/// - [`VisitorExt::visit_object`] -> [`serde::de::Visitor::visit_map`]
/// - [`VisitorExt::visit_symbol`] -> [`serde::de::Visitor::visit_borrowed_str`]
/// - [`VisitorExt::visit_ruby_string`] -> [`serde::de::Visitor::visit_borrowed_str`]/[`serde::de::Visitor::visit_string`]
pub trait VisitorExt<'de>: Visitor<'de> {
    /// For deserializing objects serialized via `_dump` in ruby.
    /// The class name is passed in as well as the relevant data.
    ///
    /// # Errors
    /// Errors by default.
    fn visit_userdata(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, DeError>;

    /// For deserializing ruby objects in general.
    /// It's different to how deserializing structs normally works in serde, as you get a class name.
    ///
    /// Forwards to [`Visitor::visit_map`] by default.
    #[allow(clippy::missing_errors_doc)]
    fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>;

    /// For deserializing ruby symbols.
    /// Only exists to distinguish between strings and symbols.
    ///
    /// Forwards to [`Visitor::visit_borrowed_str`] by default.
    #[allow(clippy::missing_errors_doc)]
    fn visit_symbol(self, sym: &'de str) -> Result<Self::Value, DeError>;

    /// For deserializing ruby strings which may or may not be utf8.
    /// You will also get any extra fields attached to the string, like the encoding (as that is a thing in ruby)
    ///
    /// By default, it uses [`String::from_utf8_lossy`] and matches the resulting [`std::borrow::Cow`] like this:
    /// ```ignore
    /// match str {
    ///     std::borrow::Cow::Borrowed(str) => self.visit_borrowed_str(str),
    ///     std::borrow::Cow::Owned(str) => self.visit_string(str),
    /// }
    /// ```
    #[allow(clippy::missing_errors_doc)]
    fn visit_ruby_string<A>(self, data: &'de [u8], fields: A) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>;
}
