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

/// A structure that can be serialized into ruby marshal data.
pub trait Serialize {
    /// Serialize this value into the given serializer.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: Serializer;
}

/// A structure that can serialize data into ruby marshal format.
///
/// You MUST provide exact sizes for arrays, hashes, etc because marshal prefixes them with their size.
pub trait Serializer: Sized {
    /// The output of the serializer.
    type Ok;

    /// The ivar serializer of this serializer.
    type SerializeIvars: SerializeIvars<Ok = Self::Ok>;
    /// The hash serializer of this serializer.
    type SerializeHash: SerializeHash<Ok = Self::Ok>;
    /// The array serializer of this serializer.
    type SerializeArray: SerializeArray<Ok = Self::Ok>;

    /// Serialize a nil value.
    fn serialize_nil(self) -> Result<Self::Ok>;

    /// Serialize a boolean value.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok>;

    /// Serialize an integer value.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok>;

    /// Serialize a float value.
    fn serialize_f64(self, v: f64) -> Result<Self::Ok>;

    /// Serialize a hash.
    fn serialize_hash(self, len: usize) -> Result<Self::SerializeHash>;

    /// Serialize an array.
    fn serialize_array(self, len: usize) -> Result<Self::SerializeArray>;

    /// Serialize a string.
    fn serialize_string(self, data: &[u8]) -> Result<Self::Ok>;

    /// Serialize a symbol.
    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok>;

    /// Serialize a regular expression.
    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> Result<Self::Ok>;

    /// Serialize a object.
    fn serialize_object(self, class: &Sym, len: usize) -> Result<Self::SerializeIvars>;

    /// Serialize a struct.
    fn serialize_struct(self, name: &Sym, len: usize) -> Result<Self::SerializeIvars>;

    /// Serialize a class.
    fn serialize_class(self, class: &Sym) -> Result<Self::Ok>;

    /// Serialize a module.
    fn serialize_module(self, module: &Sym) -> Result<Self::Ok>;

    /// Serialize an instance.
    fn serialize_instance<V>(self, value: &V, len: usize) -> Result<Self::SerializeIvars>
    where
        V: Serialize + ?Sized;

    /// Serialize an extended value.
    fn serialize_extended<V>(self, module: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    /// Serialize a user class.
    fn serialize_user_class<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    /// Serialize user data.
    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok>;

    /// Serialize user marshal.
    fn serialize_user_marshal<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    /// Serialize data.
    fn serialize_data<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized;

    /// A convenience method for serializing a string.
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

    /// A convenience method for serializing an array.
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

    /// A convenience method for serializing a hashmap.
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

/// A structure that can serialize instance variables of an object.
pub trait SerializeIvars {
    /// The output of the serializer.
    type Ok;

    /// Serialize a field.
    ///
    /// Generally you should have a symbol prefixed with an `@` character.
    /// It's not invalid to not do this, but ruby will discard your data outside of a specific circumstance.
    ///
    /// When serializing a string, the ivar `E` will indicate the encoding of the string.
    /// `false`: ASCII-8BIT
    /// `true`: UTF-8
    /// everything else: custom encoding
    ///
    /// Not providing an encoding will mean that ruby will assume the encoding is binary.
    fn serialize_field(&mut self, k: &Sym) -> Result<()>;

    /// Serialize a value.
    ///
    /// Must be called after `serialize_field`.
    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized;

    /// Serialize a field and value.
    fn serialize_entry<V>(&mut self, k: &Sym, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        self.serialize_field(k)?;
        self.serialize_value(v)
    }

    /// End the serialization.
    fn end(self) -> Result<Self::Ok>;
}

/// A structure that can serialize a hash.
pub trait SerializeHash {
    /// The output of the serializer.
    type Ok;

    /// Serialize a key.
    fn serialize_key<K>(&mut self, k: &K) -> Result<()>
    where
        K: Serialize + ?Sized;

    /// Serialize a value.
    ///
    /// Must be called after `serialize_key`.
    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized;

    /// Serialize a key and value.
    fn serialize_entry<K, V>(&mut self, k: &K, v: &V) -> Result<()>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.serialize_key(k)?;
        self.serialize_value(v)
    }

    /// End the serialization.
    fn end(self) -> Result<Self::Ok>;
}

/// A structure that can serialize an array.
pub trait SerializeArray {
    /// The output of the serializer.
    type Ok;

    /// Serialize an element.
    fn serialize_element<T>(&mut self, v: &T) -> Result<()>
    where
        T: Serialize + ?Sized;

    /// End the serialization.
    fn end(self) -> Result<Self::Ok>;
}
