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

mod de;
mod from;
mod impls;
mod ser;

mod types;
pub use types::{Object, RbArray, RbFields, RbHash, RbString, Symbol, Userdata};

/// An enum representing any ruby value.
/// Similar to `serde_json::Value`'s and the like.
///
/// Value is designed to use [`crate::VisitorExt`] extensively to avoid loss of information in the deserialization process.
/// Userdata/Object, for example, store the class name, which is not something that would normally be possible in serde.
#[derive(Default, Clone, enum_as_inner::EnumAsInner)]
pub enum Value {
    /// A value equivalent to nil in ruby (or [`()`] in rust.)
    #[default]
    Nil,
    /// A boolean value.
    Bool(bool),
    /// A float value.
    Float(f64),
    /// An integer value.
    Integer(i64),
    /// A ruby string.
    /// Because strings in ruby are not guarenteed to be utf8, [`RbString`] stores a [`Vec<u8>`] instead.
    ///
    /// See [`RbString`] for more information.
    String(RbString),
    /// A symbol from ruby.
    /// It's a newtype around a String, meant to preserve types during (de)serialization.
    /// See [`Symbol`] for more information.
    Symbol(Symbol),
    /// An array of [`Value`].
    Array(RbArray),
    /// Equivalent to a Hash in Ruby.
    Hash(RbHash),
    /// An object serialized by `_dump`.
    Userdata(Userdata),
    /// A generic ruby object.
    Object(Object),
}
