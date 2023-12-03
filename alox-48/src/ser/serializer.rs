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
#![allow(clippy::cast_possible_wrap)]

use indexmap::IndexSet;
use serde::ser;

use super::{Error, Kind, Result};
use crate::Symbol;

/// The `alox_48` serializer.
///
/// `alox_48` does not support some data types.
/// These include:
/// - Enums
/// - Newtype Structs
/// - Unit Structs
#[derive(Debug, Clone)]
pub struct Serializer {
    /// The underlying output of the serializer.
    pub output: Vec<u8>,
    symlink: IndexSet<String>,
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            output: vec![4, 8],
            symlink: IndexSet::new(),
        }
    }
}

impl Serializer {
    /// Creates a new deserializer.
    ///
    /// Same as [`Default::default`].
    #[must_use]
    pub fn new() -> Self {
        Serializer::default()
    }

    // Does not emit a type byte.
    // FIXME: find a way around these warnings
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn write_int(&mut self, v: i64) {
        match v {
            0 => self.append(0),
            1..=122 => self.append(v as u8 + 5),
            -122..=0 => self.append((256 + v - 5) as u8),
            mut v => {
                let mut res = vec![];

                for _ in 0..4 {
                    let b = v & 255;
                    res.push(b as _);

                    v >>= 8;

                    if v == 0 || v == -1 {
                        break;
                    }
                }

                let l_byte = if v < 0 {
                    (256 - res.len()) as u8
                } else {
                    res.len() as _
                };

                self.append(l_byte);
                self.output.append(&mut res);
            }
        }
    }

    fn append(&mut self, b: u8) {
        self.output.push(b);
    }

    fn write_symbol(&mut self, symbol: &str) {
        if let Some(idx) = self.symlink.get_index_of(symbol) {
            self.append(b';');
            self.write_int(idx as _);
        } else {
            self.symlink.insert(symbol.to_string());

            self.append(b':');
            self.write_int(symbol.len() as _);

            self.write_bytes(symbol);
        }
    }

    fn write_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        for &b in bytes.as_ref() {
            self.append(b);
        }
    }
}

macro_rules! serialize_int {
    ($($int:ty),*) => {
        paste::paste! {
            $(
                fn [<serialize_ $int>](self, v: $int) -> Result<Self::Ok> {
                    self.append(b'i');

                    self.write_int(v as _);

                    Ok(())
                }
            )*
        }
    };
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.append(if v { b'T' } else { b'F' });

        Ok(())
    }

    serialize_int! {
        i8, i16, i32, i64,
        u8, u16, u32, u64
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.append(b'f');

        let str = v.to_string();
        self.write_int(str.len() as _);

        self.write_bytes(str);

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        // Rust strings are always utf8, so we encode that
        self.append(b'I');

        // Write string
        self.append(b'"');
        self.write_int(v.len() as _);

        self.write_bytes(v);

        // Write the field len of 1
        self.write_int(1);

        // Append encoding (always utf8)
        self.write_symbol("E");
        self.append(b'T');

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        eprintln!("warning: serializing bytes is unclear, it will be serialized as a raw string");

        self.append(b'"');
        self.write_int(v.len() as _);

        self.write_bytes(v);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.append(b'0');

        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        T::serialize(value, self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.append(b'0');

        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        eprintln!("warning: unit structs do not map well to ruby. serializing as nil");

        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error {
            kind: Kind::Unsupported("newtype struct"),
        })
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let Some(len) = len else {
            return Err(Error {
                kind: Kind::Unsupported("sequences with no size hints"),
            });
        }; // FIXME: Find a solution to this

        self.append(b'[');
        self.write_int(len as _);

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.append(b'[');
        self.write_int(len as _);

        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error {
            kind: Kind::Unsupported("tuple struct"),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let Some(len) = len else {
            return Err(Error {
                kind: Kind::Unsupported("maps with no size hints"),
            });
        }; // FIXME: Find a solution to this

        self.append(b'{');
        self.write_int(len as _);

        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.append(b'o');
        self.write_symbol(name);

        self.write_int(len as _);

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        T::serialize(key, &mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        T::serialize(value, &mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        T::serialize(value, &mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        Serializer::write_symbol(self, &format!("@{key}"));
        T::serialize(value, &mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        T::serialize(value, &mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> super::SerializeExt for &'a mut Serializer {
    fn serialize_symbol(self, symbol: &Symbol) -> Result<Self::Ok> {
        self.write_symbol(symbol.as_str());

        Ok(())
    }

    fn serialize_ruby_string(self, string: &crate::RbString) -> Result<Self::Ok> {
        use serde::Serialize;

        if !string.fields.is_empty() {
            self.append(b'I');
        }

        // Write string
        self.append(b'"');
        self.write_int(string.len() as _);

        self.write_bytes(string.as_slice());

        if !string.fields.is_empty() {
            // Write the field len of 1
            self.write_int(1);

            for (k, v) in &string.fields {
                k.serialize(&mut *self)?;
                v.serialize(&mut *self)?;
            }
        }

        Ok(())
    }

    fn serialize_userdata(self, class: &Symbol, data: &[u8]) -> Result<Self::Ok> {
        self.append(b'u');
        self.write_symbol(class.as_str());

        self.write_int(data.len() as _);
        self.write_bytes(data);

        Ok(())
    }

    fn serialize_object(self, class: &Symbol, len: usize) -> Result<Self::SerializeObject> {
        self.append(b'o');
        self.write_symbol(class.as_str());

        self.write_int(len as _);
        Ok(self)
    }
}

impl<'a> super::SerializeObject for &'a mut Serializer {
    fn serialize_field<T: ?Sized>(&mut self, key: &Symbol, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.write_symbol(&format!("@{}", key.as_str()));

        T::serialize(value, &mut **self)
    }
}
