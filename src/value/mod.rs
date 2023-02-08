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

use enum_as_inner::EnumAsInner;
use indexmap::IndexMap;
use std::hash::Hash;

/// An enum representing any ruby value.
/// Similar to [`serde_json::Value`] and the like.
///
/// Value is designed to use [`crate::VisitorExt`] extensively to avoid loss of information in the deserialization process.
/// Userdata/Object, for example, store the class name, which is not something that would normally be possible in serde.
#[derive(Default, Debug, Clone, EnumAsInner)]
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

/// This type represents types serialized with `_dump` from ruby.
/// Its main intended use is in [`Value`], but you can also use it with [`serde::Deserialize`]:
///
/// ```
/// #[derive(serde::Deserialize)]
/// #[serde(from = "alox_48::Userdata")]
/// struct MyUserData {
///     field: [char; 4],
/// }
///
/// impl From<alox_48::Userdata> for MyUserData {
///     fn from(value: alox_48::Userdata) -> Self {
///         assert_eq!("MyUserData", value.class.to_string());
///         let field = std::array::from_fn(|i| {
///             value.data[i] as char
///         });
///
///         Self {
///             field
///         }
///     }
/// }
/// ```
#[derive(Hash, PartialEq, Eq, Default, Debug, Clone)]
pub struct Userdata {
    pub class: Symbol,
    pub data: Vec<u8>,
}

/// A type equivalent to ruby's `Object`.
/// What more needs to be said?
#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct Object {
    pub class: Symbol,
    pub fields: RbFields,
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.fields.len().hash(state);
        for (var, field) in self.fields.iter() {
            var.hash(state);
            field.hash(state);
        }
    }
}

/// A type equivalent to ruby's `String`.
/// ruby strings do not have to be utf8 encoded, so this type uses [`Vec<u8>`] instead.
///
/// ruby strings also can have attached extra fields (usually just the encoding), and this struct is no exception.
/// An [`RbString`] constructed from a rust [`String`] will always have the field `:E` set to true, which is how
/// ruby denotes that a string is utf8.
#[derive(PartialEq, Eq, Default, Clone)]
pub struct RbString {
    pub data: Vec<u8>,
    pub fields: RbFields,
}

/// A symbol from ruby.
/// It's a newtype around a String, meant to preserve types during (de)serialization.
///
/// When serializing, a [`String`] will be serialized as a String, but a [`Symbol`] will be serialized as a Symbol.
#[derive(Hash, PartialEq, Eq, Default, Debug, Clone)]
pub struct Symbol(pub String);

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Nil => other.is_nil(),
            Value::Bool(b) => {
                if let Value::Bool(b2) = other {
                    b == b2
                } else {
                    false
                }
            }
            Value::Float(f) => {
                if let Value::Float(f2) = other {
                    (f.is_nan() && f2.is_nan()) || f == f2
                } else {
                    false
                }
            }
            Value::Integer(i) => {
                if let Value::Integer(i2) = other {
                    i == i2
                } else {
                    false
                }
            }
            Value::String(s) => {
                if let Value::String(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Value::Symbol(s) => {
                if let Value::Symbol(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Value::Array(v) => {
                if let Value::Array(v2) = other {
                    v == v2
                } else {
                    false
                }
            }
            Value::Hash(h) => {
                if let Value::Hash(h2) = other {
                    h == h2
                } else {
                    false
                }
            }
            Value::Object(o) => {
                if let Value::Object(o2) = other {
                    o == o2
                } else {
                    false
                }
            }
            Value::Userdata(u) => {
                if let Value::Userdata(u2) = other {
                    u == u2
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    // FIXME: add float impl
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => {}
            Value::Bool(b) => b.hash(state),
            Value::Float(f) => f.to_bits().hash(state), // not the best but eh whos using a float as a hash key
            Value::Integer(i) => i.hash(state),
            Value::String(s) => {
                s.data.hash(state);
            }
            Value::Symbol(s) => s.0.hash(state),
            Value::Array(v) => v.hash(state),
            Value::Hash(h) => {
                h.len().hash(state);
                for (key, value) in h.iter() {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Value::Object(o) => o.hash(state),
            Value::Userdata(u) => u.hash(state),
        }
    }
}

/// Shorthand type alias for a ruby array.
pub type RbArray = Vec<Value>;
/// Shorthand type alias for a ruby hash.
pub type RbHash = IndexMap<Value, Value>;

/// A type alias used to represent fields of objects.
/// All objects store a [`Symbol`] to represent the key for instance variable, and we do that here too.
pub type RbFields = IndexMap<Symbol, Value>;
