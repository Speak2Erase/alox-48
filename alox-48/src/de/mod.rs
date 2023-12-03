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

mod deserializer;
mod error;

pub(crate) use error::Result;

pub use error::{Error, Kind};

use crate::Symbol;

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: Deserializer<'de>;
}

pub trait Deserializer<'de> {
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>;
}

pub trait Visitor<'de>: Sized {
    type Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    // Primitives
    fn visit_nil(self) -> Result<Self::Value> {}
    fn visit_bool(self, v: bool) -> Result<Self::Value> {}
    fn visit_i32(self, v: i32) -> Result<Self::Value> {}
    fn visit_u32(self, v: u32) -> Result<Self::Value> {}
    fn visit_f64(self, v: f64) -> Result<Self::Value> {}

    // Collections
    fn visit_hash(self) -> Result<Self::Value> {}
    fn visit_array(self) -> Result<Self::Value> {}
    fn visit_string(self) -> Result<Self::Value> {}
    fn visit_symbol(self) -> Result<Self::Value> {}
    fn visit_regular_expression(self) -> Result<Self::Value> {}

    fn visit_object(self) -> Result<Self::Value> {}
    fn visit_struct(self) -> Result<Self::Value> {}
    // Other
    fn visit_instance(self) -> Result<Self::Value> {}
    fn visit_extended(self) -> Result<Self::Value> {}

    fn visit_class(self) -> Result<Self::Value> {}
    fn visit_module(self) -> Result<Self::Value> {}
    // User types
    fn visit_user_class(self) -> Result<Self::Value> {}
    fn visit_user_data(self) -> Result<Self::Value> {}
    fn visit_user_marshal(self) -> Result<Self::Value> {}
}

pub trait FieldAccess<'de> {
    fn next_field(&mut self) -> Result<Option<Symbol>>;

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>;

    fn next_entry<T>(&mut self) -> Result<Option<(Symbol, T)>>
    where
        T: Deserialize<'de>;
}

pub trait HashAccess<'de> {
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>;

    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>;

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>;
}

pub trait ArrayAccess<'de> {
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>;
}
