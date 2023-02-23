#![warn(rust_2018_idioms, clippy::pedantic)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::all
)]
#![feature(min_specialization)]

//! alox-48
//! (short for aluminum oxide 48)
//!
//! alox-48 is a crate using serde designed to deserialize ruby marshal data.
//! It uses the currently nightly feature `min_specialization` to extend serde's data model,
//! preventing the loss of information in (de)serialization.
//!
//! alox-48 supports both serialization and deserialization,
//! but some types present in rust's type system and ruby's marshal format are unsupported.
//!
//! Most notably, alox-48 does NOT support object links. Object links are marshal's way of saving space,
//! if an object was serialized already a "link" indicating when it was serialized is serialized instead.
//!
//! ```rb
//! class MyClass
//!  def initialize
//!    @var = 1
//!    @string = "hiya!"
//!  end
//! end
//!
//! a = MyClass.new
//! Marshal.dump([a, a, a])
//! # The array here has 3 indices all "pointing" to the same object.
//! # Instead of serializing MyClass 3 times, Marshal will serialize it once and replace the other 2 occurences with object links.
//! # When deserializing, Marshal will preserve object links and all 3 elements in the array will point to the same object.
//! # In alox-48, this is not the case. Each index will be a "unique" ""object"".
//! ```
//!
//! This does not map well to rust, as it inherently requires a garbage collector.
//! alox-48 will still deserialize object links, however it will simply deserialize them as a copy instead.

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
/// [`crate::VisitorExt`] is responsible for extending serde.
pub mod de;
/// This crate's error type.
pub mod error;
/// Marshal serialization.
///
/// [`crate::SerializeExt`] is responsible for extending serde.
pub mod ser;
/// Untyped ruby values and rust equivalents of some ruby types (Hash, Array, etc).
///
/// Useful for deserializing untyped data.
pub mod value;

pub use de::{Deserializer, VisitorExt};
pub use error::{Error, Result};
pub use ser::{SerializeExt, Serializer};
pub use value::{Object, RbArray, RbHash, RbString, Userdata, Value};

/// Deserialize data from some bytes.
/// It's a convenience function over [`Deserializer::new`] and [`serde::Deserialize`].
#[allow(clippy::missing_errors_doc)]
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data)?;
    T::deserialize(&mut deserializer)
}

/// Serialize the type into bytes.
///
/// # Errors
/// Errors if the type contains data `alox_48` does not support.
/// These include:
/// - Enums
/// - Newtype Structs
/// - Unit Structs
pub fn to_bytes<T>(data: T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;
    Ok(serializer.output)
}
