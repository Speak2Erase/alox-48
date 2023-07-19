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

pub use ser::Serializer;

mod types;
pub use types::{Object, RbArray, RbFields, RbHash, RbString, Symbol, Userdata};

/// An enum representing any ruby value.
/// Similar to `serde_json::Value`, although much more nuanced.
///
/// Ruby marshal supports many more data types than serde can- and Value covers (almost) all of them too.
///
/// Value is designed to use [`crate::VisitorExt`] extensively to avoid loss of information in the deserialization process.
///
/// Userdata/Object, for example, store the class name, which is not something that would normally be possible in serde.
/// Symbols are preserved and deserialized as symbols.
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

/// Interpret a `alox_48::Value` as an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct User<'data> {
///     fingerprint: &'data str,
///     location:  &'data str,
/// }
///
///
/// let mut object = alox_48::Object { class: "User".into(), ..Default::default() };
/// object.fields.insert("fingerprint".into(), alox_48::RbString::from("0xF9BA143B95FF6D82").into());
/// object.fields.insert("location".into(), alox_48::RbString::from("Menlo Park, CA").into());
/// let value = alox_48::Value::Object(object);
///
/// let u: User = alox_48::value::from_value(&value).unwrap();
/// assert_eq!(u, User { fingerprint: "0xF9BA143B95FF6D82", location: "Menlo Park, CA" });
///
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the structure of `T`.
#[allow(clippy::module_name_repetitions)]
pub fn from_value<'de, T>(value: &'de Value) -> Result<T, crate::DeError>
where
    T: serde::de::Deserialize<'de>,
{
    T::deserialize(value)
}

/// Convert a `T` into `alox_48::Value`.
///
/// # Example
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize, Debug, PartialEq)]
/// struct User<'data> {
///     fingerprint: &'data str,
///     location:  &'data str,
/// }
///
///
/// let mut object = alox_48::Object { class: "User".into(), ..Default::default() };
/// object.fields.insert("fingerprint".into(), alox_48::RbString::from("0xF9BA143B95FF6D82").into());
/// object.fields.insert("location".into(), alox_48::RbString::from("Menlo Park, CA").into());
/// let original = alox_48::Value::Object(object);
///
/// let value = alox_48::value::to_value(User { fingerprint: "0xF9BA143B95FF6D82", location: "Menlo Park, CA" }).unwrap();
/// assert_eq!(original, value);
///
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to fail, or uses an unsupported data type.
#[allow(clippy::module_name_repetitions)]
pub fn to_value<T>(value: T) -> Result<Value, crate::SerError>
where
    T: serde::Serialize,
{
    T::serialize(&value, ser::Serializer)
}
