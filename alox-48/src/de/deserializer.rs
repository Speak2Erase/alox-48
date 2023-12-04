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

// These are necessary evils, sadly.
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]

use std::collections::BTreeSet;

use super::{ignored::Ignored, Error, Kind, Result};
use crate::{tag::Tag, Deserialize, DeserializerTrait, Sym, Visitor};

/// The alox-48 deserializer.
#[derive(Debug, Clone)]
pub struct Deserializer<'de> {
    cursor: Cursor<'de>,

    objtable: Vec<usize>,
    stack: Vec<usize>,
    blacklisted_objects: BTreeSet<usize>,

    sym_table: Vec<&'de Sym>,
}

#[derive(Debug, Clone)]
struct Cursor<'de> {
    input: &'de [u8],
    position: usize,
}

struct InstanceAccess<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    // used to track undeserialized data
    len: &'a mut usize,
    index: &'a mut usize,
}

struct IvarAccess<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: &'a mut usize,
}

struct ArrayAccess<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: &'a mut usize,
}

struct HashAccess<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: &'a mut usize,
}

impl<'de> Cursor<'de> {
    fn new(input: &'de [u8]) -> Self {
        Self { input, position: 0 }
    }

    fn seek(&mut self, position: usize) {
        self.position = position;
    }

    fn peek_byte(&self) -> Result<u8> {
        self.input
            .get(self.position)
            .copied()
            .ok_or(Error { kind: Kind::Eof })
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.position += 1;
        Ok(byte)
    }

    fn peek_tag(&self) -> Result<Tag> {
        let byte = self.peek_byte()?;
        Tag::from_u8(byte).ok_or(Error {
            kind: Kind::WrongTag(byte),
        })
    }

    fn next_tag(&mut self) -> Result<Tag> {
        let byte = self.next_byte()?;
        Tag::from_u8(byte).ok_or(Error {
            kind: Kind::WrongTag(byte),
        })
    }

    fn next_bytes_dyn(&mut self, length: usize) -> Result<&'de [u8]> {
        if length > self.input.len() {
            return Err(Error { kind: Kind::Eof });
        }

        let ret = &self.input[self.position..self.position + length];
        self.position += length;
        Ok(ret)
    }
}

impl<'de> Deserializer<'de> {
    /// Create a new deserializer with the given input.
    ///
    /// # Errors
    /// Will error if the input has a len < 1.
    ///
    /// Will error if the input has a version number != to 4.8.
    /// The first two bytes of marshal data encode the version number. [major, minor]
    // this function should never panic in practice as we perform bounds checking.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(input: &'de [u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        if input.len() < 2 {
            return Err(Error { kind: Kind::Eof });
        }

        let v1 = cursor.next_byte()?;
        let v2 = cursor.next_byte()?;
        if [v1, v2] != [4, 8] {
            return Err(Error {
                kind: Kind::VersionError([v1, v2]),
            });
        }

        Ok(Self {
            cursor,
            objtable: vec![],
            sym_table: vec![],
            stack: vec![],
            blacklisted_objects: BTreeSet::new(),
        })
    }

    fn read_packed_int(&mut self) -> Result<i32> {
        // The bounds of a Ruby Marshal packed integer are [-(2**30), 2**30 - 1], anything beyond that
        // gets serialized as a bignum.
        //
        // The bounds of an i32 are [-(2**31), 2**31 - 1], so we should be safe.
        let c = self.cursor.next_byte()? as i8;

        Ok(match c {
            0 => 0,
            5..=127 => (c - 5) as _,
            -128..=-5 => (c + 5) as _,
            1..=4 => {
                let mut x = 0;

                for i in 0..c {
                    let n = self.cursor.next_byte()? as i32;
                    let n = n << (8 * i);
                    x |= n;
                }

                x
            }
            -4..=-1 => {
                let mut x = -1;

                for i in 0..-c {
                    let a = !(0xFF << (8 * i)); // wtf is this magic
                    let b = self.cursor.next_byte()? as i32;
                    let b = b << (8 * i);

                    x = (x & a) | b;
                }

                x
            }
        })
    }

    #[allow(clippy::panic_in_result_fn)]
    fn read_float(&mut self) -> Result<f64> {
        let out = self.read_bytes_len()?;

        if let Some(terminator_idx) = out.iter().position(|v| *v == 0) {
            let (str, [0, mantissa @ ..]) = out.split_at(terminator_idx) else {
                unreachable!();
            };
            let float = str::parse::<f64>(&String::from_utf8_lossy(str)).map_err(|err| Error {
                kind: Kind::Message(err.to_string()),
            })?;
            let transmuted = u64::from_ne_bytes(float.to_ne_bytes());
            if mantissa.len() > 4 {
                return Err(Error {
                    kind: Kind::ParseFloatMantissaTooLong,
                });
            }
            let (mantissa, mask) = mantissa.iter().fold((0u64, 0u64), |(acc, mask), v| {
                ((acc << 8) | u64::from(*v), (mask << 8) | 0xFF)
            });

            let transmuted = (transmuted & !mask) | mantissa;
            Ok(f64::from_ne_bytes(transmuted.to_ne_bytes()))
        } else {
            Ok(
                str::parse::<f64>(&String::from_utf8_lossy(out)).map_err(|err| Error {
                    kind: Kind::Message(err.to_string()),
                })?,
            )
        }
    }

    fn read_symbol(&mut self) -> Result<&'de Sym> {
        let out = self.read_bytes_len()?;

        let str = match std::str::from_utf8(out) {
            Ok(a) => a,
            Err(err) => {
                return Err(Error {
                    kind: Kind::SymbolInvalidUTF8(err),
                });
            }
        };
        let sym = Sym::new(str);

        if self.stack.is_empty() {
            self.sym_table.push(sym);
        }
        Ok(sym)
    }

    fn read_symlink(&mut self) -> Result<&'de Sym> {
        let index = self.read_packed_int()? as usize;

        self.sym_table.get(index).copied().ok_or(Error {
            kind: Kind::UnresolvedSymlink(index),
        })
    }

    // FIXME: FIND BETTER NAME
    fn read_symbol_either(&mut self) -> Result<&'de Sym> {
        match self.cursor.next_tag()? {
            Tag::Symbol => self.read_symbol(),
            Tag::Symlink => self.read_symlink(),
            t => Err(Error {
                kind: Kind::ExpectedSymbol(t),
            }),
        }
    }

    fn is_blacklisted(&mut self, position: usize) -> bool {
        self.blacklisted_objects.contains(&position)
    }

    fn register_obj(&mut self) {
        // Only push into the object table if we are reading new input
        if !self.stack.is_empty() || self.is_blacklisted(self.cursor.position) {
            return;
        }
        self.objtable.push(self.cursor.position);
    }

    fn read_usize(&mut self) -> Result<usize> {
        let raw_length = self.read_packed_int()?;
        usize::try_from(raw_length).map_err(|_| Error {
            kind: Kind::UnexpectedNegativeLength(raw_length),
        })
    }

    fn read_bytes_len(&mut self) -> Result<&'de [u8]> {
        let len = self.read_usize()?;
        self.cursor.next_bytes_dyn(len)
    }
}

impl<'de, 'a> super::DeserializerTrait<'de> for &'a mut Deserializer<'de> {
    // This is just barely over the limit.
    // It's fine, I swear.
    #[allow(clippy::too_many_lines)]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.cursor.peek_tag()?.is_object_link_referenceable() {
            self.register_obj();
        }

        match self.cursor.next_tag()? {
            Tag::Nil => visitor.visit_nil(),
            Tag::True => visitor.visit_bool(true),
            Tag::False => visitor.visit_bool(false),
            Tag::Integer => visitor.visit_i32(self.read_packed_int()?),
            Tag::Float => visitor.visit_f64(self.read_float()?),
            Tag::String => {
                let data = self.read_bytes_len()?;
                visitor.visit_string(data)
            }
            Tag::Array => {
                let len = self.read_usize()?;
                let mut index = 0;

                let result = visitor.visit_array(ArrayAccess {
                    deserializer: self,
                    len,
                    index: &mut index,
                })?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    Ignored::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::Hash => {
                let len = self.read_usize()?;
                let mut index = 0;

                let result = visitor.visit_hash(HashAccess {
                    deserializer: self,
                    len,
                    index: &mut index,
                })?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Key
                    Ignored::deserialize(&mut *self)?;
                    // Value
                    Ignored::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::Symbol => visitor.visit_symbol(self.read_symbol()?),
            Tag::Symlink => visitor.visit_symbol(self.read_symlink()?),
            // Instance genuinely baffles me.
            Tag::Instance => {
                let mut len = 0;
                let mut index = 0;

                let result = visitor.visit_instance(&mut InstanceAccess {
                    deserializer: &mut *self,
                    len: &mut len,
                    index: &mut index,
                })?;

                while index < len {
                    index += 1;
                    // Ivar
                    self.read_symbol_either()?;
                    // Value
                    Ignored::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::Object => {
                let class = self.read_symbol_either()?;

                let len = self.read_usize()?;
                let mut index = 0;

                let result = visitor.visit_object(
                    class,
                    IvarAccess {
                        deserializer: self,
                        len,
                        index: &mut index,
                    },
                )?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Ivar
                    self.read_symbol_either()?;
                    // Value
                    Ignored::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::ObjectLink => {
                let index = self.read_usize()?;

                let jump_target = self.objtable.get(index).copied().ok_or(Error {
                    kind: Kind::UnresolvedObjectlink(index),
                })?;

                self.stack.push(self.cursor.position);
                self.cursor.seek(jump_target);

                let result = self.deserialize(visitor);

                self.cursor
                    .seek(self.stack.pop().expect("stack should not empty"));

                result
            }
            Tag::UserDef => {
                let class = self.read_symbol_either()?;
                let data = self.read_bytes_len()?;

                visitor.visit_user_data(class, data)
            }
            // FIXME: this ignores default hash values. we should fix this?
            Tag::HashDefault => {
                let len = self.read_packed_int()? as _;
                let mut index = 0;

                let result = visitor.visit_hash(HashAccess {
                    deserializer: self,
                    len,
                    index: &mut index,
                });

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Key
                    Ignored::deserialize(&mut *self)?;
                    // Value
                    Ignored::deserialize(&mut *self)?;
                }

                // Ignore the default value.
                // This should work.
                // Probably.
                // :)
                Ignored::deserialize(&mut *self)?;

                result
            }
            Tag::UserClass => {
                let class = self.read_symbol_either()?;
                visitor.visit_user_class(class, &mut *self)
            }
            Tag::RawRegexp => {
                let regex = self.read_bytes_len()?;
                let flags = self.cursor.next_byte()?;
                visitor.visit_regular_expression(regex, flags)
            }
            Tag::ClassRef => {
                let class = self.read_symbol_either()?;
                visitor.visit_class(class)
            }
            Tag::ModuleRef => {
                let module = self.read_symbol_either()?;
                visitor.visit_module(module)
            }
            // the ruby docs are wrong about this actually!
            // they say the object comes first, then the module, but actually it's the other way around.
            Tag::Extended => {
                let module = self.read_symbol_either()?;
                visitor.visit_extended(module, &mut *self)
            }
            Tag::UserMarshal => {
                let class = self.read_symbol_either()?;
                visitor.visit_user_class(class, &mut *self)
            }
            Tag::Struct => {
                let len = self.read_packed_int()? as _;
                let mut index = 0;

                let name = self.read_symbol_either()?;
                let result = visitor.visit_struct(
                    name,
                    IvarAccess {
                        deserializer: self,
                        len,
                        index: &mut index,
                    },
                )?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Member
                    self.read_symbol_either()?;
                    // Value
                    Ignored::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            // I'm not sure why this exists. The ruby marshal doc mentions that it's for types from C extensions,
            // But Data is functionally identical to UserMarshal.
            Tag::Data => {
                let class = self.read_symbol_either()?;
                visitor.visit_data(class, &mut *self)
            }
        }
    }
}

impl<'de, 'a> super::InstanceAccess<'de> for InstanceAccess<'de, 'a> {
    type IvarAccess = IvarAccess<'de, 'a>;

    fn value<V>(mut self, visitor: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: Visitor<'de>,
    {
        let result = self.deserializer.deserialize(visitor)?;

        let len = self.deserializer.read_usize()?;
        *self.len = len;

        Ok((
            result,
            IvarAccess {
                deserializer: &mut *self.deserializer,
                len,
                index: self.index,
            },
        ))
    }
}

impl<'de, 'a> super::IvarAccess<'de> for IvarAccess<'de, 'a> {
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>> {
        if *self.index >= self.len {
            return Ok(None);
        }
        *self.index += 1;

        self.deserializer.read_symbol_either().map(Some)
    }

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(&mut *self.deserializer)
    }
}

impl<'de, 'a> super::ArrayAccess<'de> for ArrayAccess<'de, 'a> {
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>,
    {
        if *self.index >= self.len {
            return Ok(None);
        }
        *self.index += 1;

        T::deserialize(&mut *self.deserializer).map(Some)
    }
}

impl<'de, 'a> super::HashAccess<'de> for HashAccess<'de, 'a> {
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>,
    {
        if *self.index >= self.len {
            return Ok(None);
        }
        *self.index += 1;

        K::deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>,
    {
        V::deserialize(&mut *self.deserializer)
    }
}
