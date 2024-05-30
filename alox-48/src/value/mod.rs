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

use crate::{
    rb_types::{Object, RbArray, RbFields, RbHash, RbString, Symbol, Userdata},
    Instance, RbStruct,
};

/// An enum representing any ruby value.
///
/// Similar to `serde_json::Value`, although much more nuanced.
#[derive(Default, Clone, enum_as_inner::EnumAsInner, Debug)]
pub enum Value {
    /// A value equivalent to nil in ruby (or [`()`] in rust.)
    #[default]
    Nil,
    /// A boolean value.
    Bool(bool),
    /// A float value.
    Float(f64),
    /// An integer value.
    Integer(i32),
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
    /// An "instance".
    ///
    /// The naming is a bit of a misnomer, as it's not an instance of a class, but rather a value with attached ivars.
    /// It's distinct from object though, as it's a value (like a string) which does not usually have ivars.
    Instance(Instance<Box<Value>>),
    /// Equivalent to a `Regexp` in Ruby.
    Regex {
        /// The regex data.
        data: RbString,
        /// Any flags associated with the regex. (global match, case insensitive, etc.)
        flags: u8,
    },
    /// Equivalent to a `Struct` in Ruby.
    RbStruct(RbStruct),
    /// Equivalent to a `Class` in Ruby.
    Class(Symbol),
    /// Equivalent to a `Module` in Ruby.
    Module(Symbol),
    /// A value that has been extended with a module.
    Extended {
        /// The module that was extended.
        module: Symbol,
        /// The value that was extended.
        value: Box<Value>,
    },
    /// A subclass of a ruby class like `Hash` or `Array`.
    UserClass {
        /// The subclass.
        class: Symbol,
        /// The value of the subclass.
        value: Box<Value>,
    },
    /// An object that has been serialized as another type.
    UserMarshal {
        /// The class of the original object.
        class: Symbol,
        /// The value of the object.
        value: Box<Value>,
    },
    /// Unclear what this is? It's releated to C extensions according to the ruby docs.
    Data {
        /// The class of the data.
        class: Symbol,
        /// The value of the data.
        value: Box<Value>,
    },
}

/// Interpret a `alox_48::Value` as an instance of type `T`.
///
/// # Example
///
/// ```
/// use alox_48::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
///
/// let mut object = alox_48::Object { class: "User".into(), ..Default::default() };
/// object.fields.insert("fingerprint".into(), alox_48::RbString::from("0xF9BA143B95FF6D82").into());
/// object.fields.insert("location".into(), alox_48::RbString::from("Menlo Park, CA").into());
/// let value = alox_48::Value::Object(object);
///
/// let u: User = alox_48::from_value(&value).unwrap();
/// assert_eq!(u, User { fingerprint: "0xF9BA143B95FF6D82".to_string(), location: "Menlo Park, CA".to_string() });
///
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the structure of `T`.
#[allow(clippy::module_name_repetitions)]
pub fn from_value<'de, T>(value: &'de Value) -> Result<T, crate::DeError>
where
    T: crate::Deserialize<'de>,
{
    T::deserialize(value)
}

/// Convert a `T` into `alox_48::Value`.
///
/// # Example
///
/// ```
/// use alox_48::Serialize;
///
/// #[derive(Serialize, Debug, PartialEq)]
/// struct User {
///     fingerprint: String,
///     location:  String,
/// }
///
///
/// let mut object = alox_48::Object { class: "User".into(), ..Default::default() };
/// object.fields.insert("@fingerprint".into(), alox_48::Instance::from("0xF9BA143B95FF6D82").into());
/// object.fields.insert("@location".into(), alox_48::Instance::from("Menlo Park, CA").into());
/// let original = alox_48::Value::Object(object);
///
/// let value = alox_48::to_value(User { fingerprint: "0xF9BA143B95FF6D82".to_string(), location: "Menlo Park, CA".to_string() }).unwrap();
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
    T: crate::Serialize,
{
    T::serialize(&value, ser::Serializer)
}
