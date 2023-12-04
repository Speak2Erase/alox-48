use crate::Sym;

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
use super::Result;

pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: Serializer;
}

pub trait Serializer {
    type Ok;

    type SerializeIvars: SerializeIvars;
    type SerializeHash: SerializeHash;
    type SerializeArray: SerializeArray;

    fn serialize_nil(self) -> Result<Self::Ok>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok>;

    fn serialize_i32(self, v: i32) -> Result<Self::Ok>;

    fn serialize_f64(self, v: f64) -> Result<Self::Ok>;

    fn serialize_hash(self, len: usize) -> Result<Self::SerializeHash>;

    fn serialize_array(self, len: usize) -> Result<Self::SerializeArray>;

    // todo maybe have a convenience method that serializes strings with the encoding set?
    fn serialize_string(self, data: &[u8]) -> Result<Self::Ok>;

    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok>;

    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> Result<Self::Ok>;

    fn serialize_object(self, class: &Sym, len: usize) -> Result<Self::SerializeIvars>;

    fn serialize_struct(self, name: &Sym, len: usize) -> Result<Self::SerializeIvars>;

    fn serialize_class(self, class: &Sym) -> Result<Self::Ok>;

    fn serialize_module(self, module: &Sym) -> Result<Self::Ok>;

    fn serialize_instance<V>(self, value: &V, len: usize) -> Result<Self::SerializeIvars>
    where
        V: Serialize + ?Sized;

    fn serialize_extended<V>(self, module: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    fn serialize_user_class<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok>;

    fn serialize_user_marshal<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    fn serialize_data<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;
}

pub trait SerializeIvars {
    type Ok;

    fn serialize_field(&mut self, k: &Sym) -> Result<()>;

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized;

    fn serialize_entry<V>(&mut self, k: &Sym, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        self.serialize_field(k)?;
        self.serialize_value(v)
    }

    fn end(self) -> Result<Self::Ok>;
}

pub trait SerializeHash {
    type Ok;

    fn serialize_key<K>(&mut self, k: &K) -> Result<()>
    where
        K: Serialize + ?Sized;

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized;

    fn serialize_entry<K, V>(&mut self, k: &K, v: &V) -> Result<()>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.serialize_key(k)?;
        self.serialize_value(v)
    }

    fn end(self) -> Result<Self::Ok>;
}

pub trait SerializeArray {
    type Ok;

    fn serialize_element<T>(&mut self, v: &T) -> Result<()>
    where
        T: Serialize + ?Sized;

    fn end(self) -> Result<Self::Ok>;
}
