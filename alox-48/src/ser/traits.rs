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

pub trait Serializer: Sized {
    type Ok;

    type SerializeIvars: SerializeIvars<Ok = Self::Ok>;
    type SerializeHash: SerializeHash<Ok = Self::Ok>;
    type SerializeArray: SerializeArray<Ok = Self::Ok>;

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

    // provided
    fn serialize_rust_string(self, string: &str) -> Result<Self::Ok> {
        struct StringSerialize<'a>(&'a str);
        impl<'a> Serialize for StringSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
            where
                S: Serializer,
            {
                serializer.serialize_string(self.0.as_bytes())
            }
        }
        let mut fields = self.serialize_instance(&StringSerialize(string), 1)?;
        fields.serialize_entry(Sym::new("E"), &true)?;
        fields.end()
    }

    fn collect_array<I>(self, iter: I) -> Result<Self::Ok>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Serialize,
    {
        let iter = iter.into_iter();
        let mut serialize_array = self.serialize_array(iter.len())?;
        for item in iter {
            serialize_array.serialize_element(&item)?;
        }
        serialize_array.end()
    }

    fn collect_hash<K, V, I>(self, iter: I) -> Result<Self::Ok>
    where
        I: IntoIterator<Item = (K, V)>,
        I::IntoIter: ExactSizeIterator,
        K: Serialize,
        V: Serialize,
    {
        let iter = iter.into_iter();
        let mut serialize_hash = self.serialize_hash(iter.len())?;
        for (key, value) in iter {
            serialize_hash.serialize_entry(&key, &value)?;
        }
        serialize_hash.end()
    }
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
