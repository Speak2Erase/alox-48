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
use crate::{Sym, Symbol};

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
    symlink: IndexSet<Symbol>,
}

#[derive(Debug)]
pub struct SerializeIvars<'a> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
}

#[derive(Debug)]
pub struct SerializeHash<'a> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
}

#[derive(Debug)]
pub struct SerializeArray<'a> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
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
            0 => self.write(0),
            1..=122 => self.write(v as u8 + 5),
            -122..=0 => self.write((256 + v - 5) as u8),
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

                self.write(l_byte);
                self.write_bytes(res);
            }
        }
    }

    fn write(&mut self, b: u8) {
        self.output.push(b);
    }

    fn write_symbol(&mut self, symbol: &Sym) {
        if let Some(idx) = self.symlink.get_index_of(symbol) {
            self.write(b';');
            self.write_int(idx as _);
        } else {
            self.symlink.insert(symbol.to_symbol());

            self.write(b':');
            self.write_int(symbol.len() as _);

            self.write_bytes(symbol);
        }
    }

    fn write_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        for &b in bytes.as_ref() {
            self.write(b);
        }
    }

    fn write_bytes_len(&mut self, bytes: impl AsRef<[u8]>) {
        let bytes = bytes.as_ref();

        self.write_int(bytes.len() as _);
        self.write_bytes(bytes)
    }
}

impl<'a> super::SerializerTrait for &'a mut Serializer {
    type Ok = ();

    type SerializeIvars = SerializeIvars<'a>;
    type SerializeHash = SerializeHash<'a>;
    type SerializeArray = SerializeArray<'a>;

    fn serialize_nil(self) -> Result<Self::Ok> {
        self.write(b'0');

        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.write(if v { b'T' } else { b'F' });

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.write(b'i');
        self.write_int(v as i64);

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.write(b'f');

        let str = v.to_string();
        self.write_bytes_len(str);

        Ok(())
    }

    fn serialize_hash(self, len: usize) -> Result<Self::SerializeHash> {
        self.write(b'{');
        self.write_int(len as _);

        Ok(SerializeHash {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_array(self, len: usize) -> Result<Self::SerializeArray> {
        self.write(b'[');
        self.write_int(len as _);

        Ok(SerializeArray {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_string(self, data: &[u8]) -> Result<Self::Ok> {
        self.write(b'"');
        self.write_bytes_len(data);

        Ok(())
    }

    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok> {
        self.write_symbol(sym);

        Ok(())
    }

    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> Result<Self::Ok> {
        self.write(b'/');
        self.write_bytes_len(regex);
        self.write(flags);

        Ok(())
    }

    fn serialize_object(self, class: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        self.write(b'o');
        self.write_symbol(class);
        self.write_int(len as _);

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_struct(self, name: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        self.write(b'S');
        self.write_symbol(name);
        self.write_int(len as _);

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_class(self, class: &Sym) -> Result<Self::Ok> {
        self.write(b'c');
        // Apparently, this isn't a symbol. How strange!
        self.write_bytes_len(class);

        Ok(())
    }

    fn serialize_module(self, module: &Sym) -> Result<Self::Ok> {
        self.write(b'm');
        self.write_bytes_len(module);

        Ok(())
    }

    fn serialize_instance<V>(self, value: &V, len: usize) -> Result<Self::SerializeIvars>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(b'I');
        value.serialize(&mut *self)?;
        self.write_int(len as _);

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_extended<V>(self, module: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        // the ruby docs lie! it is the module which comes before the value.
        self.write(b'e');
        self.write_symbol(module);
        value.serialize(self)
    }

    fn serialize_user_class<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(b'C');
        self.write_symbol(class);
        value.serialize(self)
    }

    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok> {
        self.write(b'u');
        self.write_symbol(class);
        self.write_bytes(data);

        Ok(())
    }

    fn serialize_user_marshal<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(b'U');
        self.write_symbol(class);
        value.serialize(self)
    }

    fn serialize_data<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(b'd');
        self.write_symbol(class);
        value.serialize(self)
    }
}

impl<'a> super::SerializeIvars for SerializeIvars<'a> {
    type Ok = ();

    fn serialize_field(&mut self, k: &Sym) -> Result<()> {
        self.index += 1;
        if self.index > self.len {
            return Err(Error {
                kind: Kind::OvershotProvidedLen(self.len),
            });
        }

        self.serializer.write_symbol(k);

        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: crate::Serialize + ?Sized,
    {
        v.serialize(&mut *self.serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.index < self.len {
            Err(Error {
                kind: Kind::UndershotProvidedLen(self.len),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> super::SerializeHash for SerializeHash<'a> {
    type Ok = ();

    fn serialize_key<K>(&mut self, k: &K) -> Result<()>
    where
        K: crate::Serialize + ?Sized,
    {
        self.index += 1;
        if self.index > self.len {
            return Err(Error {
                kind: Kind::OvershotProvidedLen(self.len),
            });
        }

        k.serialize(&mut *self.serializer)?;

        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: crate::Serialize + ?Sized,
    {
        v.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        if self.index < self.len {
            Err(Error {
                kind: Kind::UndershotProvidedLen(self.len),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> super::SerializeArray for SerializeArray<'a> {
    type Ok = ();

    fn serialize_element<T>(&mut self, v: &T) -> Result<()>
    where
        T: crate::Serialize + ?Sized,
    {
        self.index += 1;
        v.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        if self.index < self.len {
            Err(Error {
                kind: Kind::UndershotProvidedLen(self.len),
            })
        } else {
            Ok(())
        }
    }
}
