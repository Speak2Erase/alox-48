#![warn(rust_2018_idioms, clippy::all)]
#![feature(min_specialization)]

//! alox-48
//! (short for aluminum oxide 48)
//!
//! alox-48 is a crate using serde designed to deserialize ruby marshal data.
//! It uses the currently nightly feature `min_specialization` to extend serde's data model,
//! preventing the loss of information in (de)serialization.

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

pub(crate) mod tag;

/// Deserialization via marshal.
///
/// [`de::extension::VisitorExt`] is responsible for extending serde.
pub mod de;
/// Error type.
pub mod error;
/// Serialization **(WIP!)**
pub mod ser;
/// Untyped ruby values and rust equivalents of some ruby types (Hash, Array, etc).
///
/// Useful for deserializing untyped data.
pub mod value;

pub use de::{Deserializer, VisitorExt};
pub use error::{Error, Result};
pub use ser::{to_bytes, SerializeExt, Serializer};
pub use value::{Object, RbArray, RbHash, RbString, Userdata, Value};

/// Deserialize data from some bytes.
/// It's a convenience function over [`Deserializer::new`] and [`serde::Deserialize`].
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data)?;
    T::deserialize(&mut deserializer)
}
