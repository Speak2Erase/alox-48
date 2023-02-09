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

use serde::de;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::forward_to_deserialize_any;

use super::VisitorExt;
use crate::tag::Tag;
use crate::Error;
use crate::Result;

/// The alox-48 deserializer.
#[derive(Debug, Clone)]
pub struct Deserializer<'de> {
    input: &'de [u8],
    objtable: Vec<&'de [u8]>,
    sym_table: Vec<&'de str>,
    stack: Vec<&'de [u8]>,
    remove_ivar_prefix: bool,
    blacklisted_objects: BTreeSet<&'de [u8]>,
}

impl<'de> Deserializer<'de> {
    /// Create a new deserializer with the given input.
    ///
    /// # Errors
    /// Will error if the input has a len < 1.
    ///
    /// Will error if the input has a version number != to 4.8.
    /// The first two bytes of marshal data encode the version number. [major, minor]
    pub fn new(input: &'de [u8]) -> crate::Result<Self> {
        if input.len() < 2 {
            return Err(Error::Eof);
        }

        let (ver, input) = input.split_at(2);
        if ver != [4, 8] {
            return Err(Error::VersionError(std::array::from_fn(|i| ver[i])));
        }

        Ok(Self {
            input,
            objtable: vec![],
            sym_table: vec![],
            stack: vec![],
            remove_ivar_prefix: false,
            blacklisted_objects: BTreeSet::new(),
        })
    }

    fn peek_byte(&self) -> Result<u8> {
        self.input.first().copied().ok_or(Error::Eof)
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn peek_tag(&self) -> Result<Tag> {
        self.peek_byte()?.try_into()
    }

    fn next_tag(&mut self) -> Result<Tag> {
        self.next_byte()?.try_into()
    }

    fn next_bytes_dyn(&mut self, length: usize) -> Result<&'de [u8]> {
        if length > self.input.len() {
            return Result::Err(Error::Eof);
        }

        let (ret, remaining) = self.input.split_at(length);
        self.input = remaining;
        Ok(ret)
    }

    fn read_packed_int(&mut self) -> Result<i32> {
        // The bounds of a Ruby Marshal packed integer are [-(2**30), 2**30 - 1], anything beyond that
        // gets serialized as a bignum.
        //
        // The bounds of an i32 are [-(2**31), 2**31 - 1], so we should be safe.
        let c = self.next_byte()? as i8;

        Ok(match c {
            0 => 0,
            5..=127 => (c - 5) as _,
            -128..=-5 => (c + 5) as _,
            c => {
                let mut x = 0;

                for i in 0..c {
                    x |= (self.next_byte()? as i32) << (8 * i);
                }

                x
            }
        })
    }

    #[allow(clippy::panic_in_result_fn)]
    fn read_float(&mut self) -> Result<f64> {
        let raw_length = self.read_packed_int()?;
        let length = raw_length
            .try_into()
            .map_err(|_| Error::UnexpectedNegativeLength(raw_length))?;
        let out = self.next_bytes_dyn(length)?;

        if let Some(terminator_idx) = out.iter().position(|v| *v == 0) {
            let (str, [0, mantissa @ ..]) = out.split_at(terminator_idx) else {
                unreachable!();
            };
            let float = str::parse::<f64>(&String::from_utf8_lossy(str))
                .map_err(|err| Error::Message(err.to_string()))?;
            let transmuted = u64::from_ne_bytes(float.to_ne_bytes());
            if mantissa.len() > 4 {
                return Err(Error::ParseFloatMantissaTooLong);
            }
            let (mantissa, mask) = mantissa.iter().fold((0u64, 0u64), |(acc, mask), v| {
                ((acc << 8) | u64::from(*v), (mask << 8) | 0xFF)
            });

            let transmuted = (transmuted & !mask) | mantissa;
            Ok(f64::from_ne_bytes(transmuted.to_ne_bytes()))
        } else {
            Ok(str::parse::<f64>(&String::from_utf8_lossy(out))
                .map_err(|err| Error::Message(err.to_string()))?)
        }
    }

    fn read_symbol(&mut self) -> Result<&'de str> {
        let length = self.read_packed_int()? as usize;
        let out = self.next_bytes_dyn(length)?;

        let mut str = match std::str::from_utf8(out) {
            Ok(a) => a,
            Err(err) => return Err(Error::SymbolInvalidUTF8(err)),
        };

        if self.remove_ivar_prefix {
            str = &str[1..];
        }

        if self.stack.is_empty() {
            self.sym_table.push(str);
        }
        Ok(str)
    }

    fn read_symlink(&mut self) -> Result<&'de str> {
        let index = self.read_packed_int()? as usize;

        self.sym_table
            .get(index)
            .copied()
            .ok_or(Error::UnresolvedSymlink(index))
    }

    // FIXME: FIND BETTER NAME
    fn read_symbol_either(&mut self) -> Result<&'de str> {
        match self.next_tag()? {
            Tag::Symbol => self.read_symbol(),
            Tag::Symlink => self.read_symlink(),
            t => Err(Error::ExpectedSymbol(t)),
        }
    }

    fn is_blacklisted(&mut self, slice: &'de [u8]) -> bool {
        self.blacklisted_objects.contains(slice)
    }

    fn register_obj(&mut self) {
        // Only push into the object table if we are reading new input
        if !self.stack.is_empty() || self.is_blacklisted(self.input) {
            return;
        }
        self.objtable.push(self.input);
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // This is just barely over the limit.
    // It's fine, I swear.
    #[allow(clippy::too_many_lines)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorExt<'de>,
    {
        if !matches!(
            self.peek_tag()?,
            Tag::Nil
                | Tag::True
                | Tag::False
                | Tag::Integer
                | Tag::Symbol
                | Tag::Symlink
                | Tag::ObjectLink
        ) {
            self.register_obj();
        }

        match self.next_tag()? {
            Tag::Nil => visitor.visit_unit(),
            Tag::True => visitor.visit_bool(true),
            Tag::False => visitor.visit_bool(false),
            Tag::Integer => visitor.visit_i32(self.read_packed_int()?),
            Tag::Float => visitor.visit_f64(self.read_float()?),
            Tag::String => {
                let len = self.read_packed_int()? as _;
                let bytes = self.next_bytes_dyn(len)?;

                visitor.visit_ruby_string(
                    bytes,
                    HashSeq {
                        deserializer: self,
                        len: 0,
                        index: 0,
                        remove_ivar_prefix: false,
                    },
                )
            }
            Tag::Array => {
                let len = self.read_packed_int()? as _;

                visitor.visit_seq(ArraySeq {
                    deserializer: self,
                    len,
                    index: 0,
                })
            }
            Tag::Hash => {
                let len = self.read_packed_int()? as _;

                visitor.visit_map(HashSeq {
                    deserializer: self,
                    len,
                    index: 0,
                    remove_ivar_prefix: false,
                })
            }
            Tag::Symbol => visitor.visit_symbol(self.read_symbol()?),
            Tag::Symlink => visitor.visit_symbol(self.read_symlink()?),
            Tag::Instance => match self.next_tag()? {
                Tag::String => {
                    let len = self.read_packed_int()? as _;
                    let bytes = self.next_bytes_dyn(len)?;

                    let len = self.read_packed_int()? as _;

                    visitor.visit_ruby_string(
                        bytes,
                        HashSeq {
                            len,
                            index: 0,
                            deserializer: self,
                            remove_ivar_prefix: false,
                        },
                    )
                }
                // This should work.
                // I think.
                // Denial is always the best solution. :)
                _ => self.deserialize_any(visitor),
            },
            Tag::Object => {
                let class = self.read_symbol_either()?;
                let len = self.read_packed_int()? as _;

                visitor.visit_object(
                    class,
                    HashSeq {
                        deserializer: self,
                        len,
                        index: 0,
                        remove_ivar_prefix: true,
                    },
                )
            }
            Tag::ObjectLink => {
                let index = self.read_packed_int()? as _;

                let jump_target = self
                    .objtable
                    .get(index)
                    .copied()
                    .ok_or(Error::UnresolvedObjectlink(index))?;

                self.stack.push(self.input);
                self.input = jump_target;

                let result = self.deserialize_any(visitor);

                self.input = self.stack.pop().expect("stack empty");

                result
            }
            Tag::UserDef => {
                let class = self.read_symbol_either()?;
                let len = self.read_packed_int()? as _;

                let data = self.next_bytes_dyn(len)?;

                visitor.visit_userdata(class, data)
            }

            // FIXME: Account for this
            Tag::HashDefault => Err(Error::Unsupported("Hash with default value")),
            Tag::UserClass => Err(Error::Unsupported(
                "User class (class inheriting from a default ruby class)",
            )), // FIXME: make this forward to newtype
            Tag::RawRegexp => Err(Error::Unsupported("Regex")),
            Tag::ClassRef => Err(Error::Unsupported("Class Reference")),
            Tag::ModuleRef => Err(Error::Unsupported("Module Reference")),
            Tag::Extended => Err(Error::Unsupported("Extended object")),
            Tag::UserMarshal => Err(Error::Unsupported(
                "User marshal (object serialized as another)",
            )), // FIXME: Find better name
            Tag::Struct => Err(Error::Unsupported("Ruby struct")), // FIXME: change this in the future
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.peek_tag()? {
            Tag::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ArraySeq<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for ArraySeq<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }

        self.index += 1;

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - self.index)
    }
}

struct HashSeq<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: usize,
    remove_ivar_prefix: bool,
}

impl<'de, 'a> MapAccess<'de> for HashSeq<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }

        self.deserializer.remove_ivar_prefix = self.remove_ivar_prefix;

        self.index += 1;

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.deserializer.remove_ivar_prefix = false;

        seed.deserialize(&mut *self.deserializer)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - self.index)
    }
}
