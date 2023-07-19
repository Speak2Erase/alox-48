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
use serde::Deserialize;

use super::VisitorExt;
use super::{bubble_error, Context, Error, Kind, Result};
use crate::tag::Tag;

/// The alox-48 deserializer.
#[derive(Debug, Clone)]
pub struct Deserializer<'de> {
    input: &'de [u8],
    objtable: Vec<&'de [u8]>,
    sym_table: Vec<&'de str>,
    stack: Vec<&'de [u8]>,
    blacklisted_objects: BTreeSet<&'de [u8]>,
}

macro_rules! deserialize_len {
    ($this:expr, $($context:expr),+ $(,)?) => {{
        let raw_length = bubble_error!($this.read_packed_int(), $( $context, )+);
        raw_length.try_into().map_err(|_| Error {
            kind: Kind::UnexpectedNegativeLength(raw_length),
            context: vec![$( $context, )+],
            span: span!(self)
        })?
    }};
}

macro_rules! span {
    ($this:expr) => {
        (0..0).into() // TODO
    };
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
        if input.len() < 2 {
            return Err(Error {
                kind: Kind::Eof,
                context: vec![Context::ParsingVersion],
                span: (0..input.len()).into(),
            });
        }

        let (ver, input) = input.split_at(2);
        if ver != [4, 8] {
            return Err(Error {
                kind: Kind::VersionError(ver.try_into().unwrap()),
                context: vec![Context::ParsingVersion],
                span: (0..2).into(),
            });
        }

        Ok(Self {
            input,
            objtable: vec![],
            sym_table: vec![],
            stack: vec![],
            blacklisted_objects: BTreeSet::new(),
        })
    }

    fn peek_byte(&self) -> Result<u8> {
        self.input.first().copied().ok_or(Error {
            kind: Kind::Eof,
            context: vec![],
            span: span!(self),
        })
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn peek_tag(&self) -> Result<Tag> {
        let byte = bubble_error!(self.peek_byte(), Context::FindingTag);
        Tag::from_u8(byte).ok_or(Error {
            kind: Kind::WrongTag(byte),
            context: vec![Context::FindingTag],
            span: span!(self),
        })
    }

    fn next_tag(&mut self) -> Result<Tag> {
        let byte = bubble_error!(self.next_byte(), Context::FindingTag);
        Tag::from_u8(byte).ok_or(Error {
            kind: Kind::WrongTag(byte),
            context: vec![Context::FindingTag],
            span: span!(self),
        })
    }

    fn next_bytes_dyn(&mut self, length: usize) -> Result<&'de [u8]> {
        if length > self.input.len() {
            return Err(Error {
                kind: Kind::Eof,
                context: vec![],
                span: span!(self),
            });
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
        let c = bubble_error!(self.next_byte(), Context::Integer) as i8;

        Ok(match c {
            0 => 0,
            5..=127 => (c - 5) as _,
            -128..=-5 => (c + 5) as _,
            1..=4 => {
                let mut x = 0;

                for i in 0..c {
                    let n = bubble_error!(self.next_byte(), Context::Integer) as i32;
                    let n = n << (8 * i);
                    x |= n;
                }

                x
            }
            -4..=-1 => {
                let mut x = -1;

                for i in 0..-c {
                    let a = !(0xFF << (8 * i)); // wtf is this magic
                    let b = bubble_error!(self.next_byte(), Context::Integer) as i32;
                    let b = b << (8 * i);

                    x = (x & a) | b;
                }

                x
            }
        })
    }

    #[allow(clippy::panic_in_result_fn)]
    fn read_float(&mut self) -> Result<f64> {
        let length = deserialize_len!(self, Context::Float);
        let out = bubble_error!(self.next_bytes_dyn(length), Context::Float);

        if let Some(terminator_idx) = out.iter().position(|v| *v == 0) {
            let (str, [0, mantissa @ ..]) = out.split_at(terminator_idx) else {
                unreachable!();
            };
            let float = str::parse::<f64>(&String::from_utf8_lossy(str)).map_err(|err| Error {
                kind: Kind::Message(err.to_string()),
                context: vec![Context::Float],
                span: span!(self),
            })?;
            let transmuted = u64::from_ne_bytes(float.to_ne_bytes());
            if mantissa.len() > 4 {
                return Err(Error {
                    kind: Kind::ParseFloatMantissaTooLong,
                    context: vec![Context::Float],
                    span: span!(self),
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
                    context: vec![Context::Float],
                    span: span!(self),
                })?,
            )
        }
    }

    fn read_symbol(&mut self) -> Result<&'de str> {
        let length = deserialize_len!(self, Context::Symbol);
        let out = self.next_bytes_dyn(length)?;

        let str = match std::str::from_utf8(out) {
            Ok(a) => a,
            Err(err) => {
                return Err(Error {
                    kind: Kind::SymbolInvalidUTF8(err),
                    context: vec![Context::Symbol],
                    span: span!(self),
                });
            }
        };

        if self.stack.is_empty() {
            self.sym_table.push(str);
        }
        Ok(str)
    }

    fn read_symlink(&mut self) -> Result<&'de str> {
        let index = self.read_packed_int()? as usize;

        self.sym_table.get(index).copied().ok_or(Error {
            kind: Kind::UnresolvedSymlink(index),
            context: vec![Context::Symbol],
            span: span!(self),
        })
    }

    // FIXME: FIND BETTER NAME
    fn read_symbol_either(&mut self) -> Result<&'de str> {
        match self.next_tag()? {
            Tag::Symbol => self.read_symbol(),
            Tag::Symlink => self.read_symlink(),
            t => Err(Error {
                kind: Kind::ExpectedSymbol(t),
                context: vec![Context::Symbol],
                span: span!(self),
            }),
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
            bubble_error!(self.peek_tag(), Context::FindingTag),
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

        match bubble_error!(self.next_tag(), Context::FindingTag) {
            Tag::Nil => visitor.visit_unit(),
            Tag::True => visitor.visit_bool(true),
            Tag::False => visitor.visit_bool(false),
            Tag::Integer => visitor.visit_i32(self.read_packed_int()?),
            Tag::Float => visitor.visit_f64(self.read_float()?),
            Tag::String => {
                let len = deserialize_len!(self, Context::StringText);
                let data = bubble_error!(self.next_bytes_dyn(len), Context::StringText);

                let mut index = 0;
                let result = visitor.visit_ruby_string(
                    data,
                    HashSeq {
                        deserializer: self,
                        len: 0,
                        index: &mut index,
                    },
                )?;
                // We don't need to deserialize anything else as this is a bare string.
                Ok(result)
            }
            Tag::Array => {
                let len = deserialize_len!(self, Context::ArrayLen);
                let mut index = 0;

                let result = visitor.visit_seq(ArraySeq {
                    deserializer: self,
                    len,
                    index: &mut index,
                })?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::Hash => {
                let len = deserialize_len!(self, Context::HashLen);
                let mut index = 0;

                let result = visitor.visit_map(HashSeq {
                    deserializer: self,
                    len,
                    index: &mut index,
                })?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Key
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                    // Value
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::Symbol => visitor.visit_symbol(self.read_symbol()?),
            Tag::Symlink => visitor.visit_symbol(self.read_symlink()?),
            // Honestly, I have no idea why this is a thing...
            // Instance is extremely unclear and I've never seen Marshal data with it prefixing anything but a string.
            Tag::Instance => {
                match bubble_error!(self.next_tag(), Context::Instance) {
                    Tag::String => {
                        let len = deserialize_len!(self, Context::Instance, Context::StringText);
                        let data = bubble_error!(
                            self.next_bytes_dyn(len),
                            Context::Instance,
                            Context::StringText
                        );

                        let len = deserialize_len!(self, Context::Instance, Context::StringFields);
                        let mut index = 0;

                        let result = visitor.visit_ruby_string(
                            data,
                            HashSeq {
                                deserializer: self,
                                len,
                                index: &mut index,
                            },
                        )?;

                        // Deserialize remaining elements that weren't deserialized
                        while index < len {
                            index += 1;
                            // Key
                            serde::de::IgnoredAny::deserialize(&mut *self)?;
                            // Value
                            serde::de::IgnoredAny::deserialize(&mut *self)?;
                        }

                        Ok(result)
                    }
                    // This should work. Definitely.
                    // :ferrisclueless:
                    _ => Err(Error {
                        kind: Kind::Unsupported("Instance with raw IVARs"),
                        context: vec![Context::Instance, Context::FindingTag],
                        span: span!(self),
                    }),
                }
            }
            Tag::Object => {
                let class = bubble_error!(self.read_symbol_either(), Context::ClassName);

                let len = deserialize_len!(self, Context::ObjectLen);
                let mut index = 0;

                let result = visitor.visit_object(
                    class,
                    ObjSeq {
                        deserializer: self,
                        len,
                        index: &mut index,
                    },
                )?;

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Key
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                    // Value
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                }

                Ok(result)
            }
            Tag::ObjectLink => {
                let index = self.read_packed_int()? as _;

                let jump_target = self.objtable.get(index).copied().ok_or(Error {
                    kind: Kind::UnresolvedObjectlink(index),
                    context: vec![Context::Objectlink(index)],
                    span: span!(self),
                })?;

                self.stack.push(self.input);
                self.input = jump_target;

                let result = self.deserialize_any(visitor);

                self.input = self.stack.pop().expect("stack empty");

                result
            }
            Tag::UserDef => {
                let class = bubble_error!(self.read_symbol_either(), Context::ClassName);
                let len = deserialize_len!(self, Context::UserdataLen);

                let data = self.next_bytes_dyn(len)?;

                visitor.visit_userdata(class, data)
            }
            // FIXME: lazy
            Tag::HashDefault => {
                let len = self.read_packed_int()? as _;
                let mut index = 0;

                let result = visitor.visit_map(HashSeq {
                    deserializer: self,
                    len,
                    index: &mut index,
                });

                // Deserialize remaining elements that weren't deserialized
                while index < len {
                    index += 1;
                    // Key
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                    // Value
                    serde::de::IgnoredAny::deserialize(&mut *self)?;
                }

                // Ignore the default value.
                // This should work.
                // Probably.
                // :)
                serde::de::IgnoredAny::deserialize(&mut *self)?;

                result
            }
            Tag::UserClass => Err(Error {
                kind: Kind::Unsupported("User class (class inheriting from a default ruby class)"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }), // FIXME: make this forward to newtype
            Tag::RawRegexp => Err(Error {
                kind: Kind::Unsupported("Regex"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }),
            Tag::ClassRef => Err(Error {
                kind: Kind::Unsupported("Class Reference"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }),
            Tag::ModuleRef => Err(Error {
                kind: Kind::Unsupported("Module Reference"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }),
            Tag::Extended => Err(Error {
                kind: Kind::Unsupported("Extended object"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }),
            Tag::UserMarshal => Err(Error {
                kind: Kind::Unsupported("User marshal (object serialized as another)"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }), // FIXME: Find better name
            Tag::Struct => Err(Error {
                kind: Kind::Unsupported("Ruby struct"),
                context: vec![Context::FindingTag],
                span: span!(self),
            }), // FIXME: change this in the future
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.peek_tag()? {
            Tag::Nil => {
                self.next_byte()?;

                visitor.visit_none()
            }
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
    index: &'a mut usize,
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
        if *self.index >= self.len {
            return Ok(None);
        }

        *self.index += 1;

        Ok(bubble_error!(
            seed.deserialize(&mut *self.deserializer).map(Some),
            Context::Array(self.len)
        ))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - *self.index)
    }
}

struct HashSeq<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: &'a mut usize,
}

impl<'de, 'a> MapAccess<'de> for HashSeq<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if *self.index >= self.len {
            return Ok(None);
        }

        *self.index += 1;

        Ok(bubble_error!(
            seed.deserialize(&mut *self.deserializer).map(Some),
            Context::Key
        ))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        Ok(bubble_error!(
            seed.deserialize(&mut *self.deserializer),
            Context::Key
        ))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - *self.index)
    }
}

struct ObjSeq<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    index: &'a mut usize,
}

struct ObjKeyDeserializer<'a, 'de>(&'a mut Deserializer<'de>);

impl<'a, 'de> serde::Deserializer<'de> for ObjKeyDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: VisitorExt<'de>,
    {
        // as we're deserializing an instance variable, we're going to get a string prefixed via a @.
        let symbol: &str = Deserialize::deserialize(&mut *self.0)?;
        // remove the @ prefix, and visit symbol.
        visitor.visit_symbol(&symbol[1..])
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf unit unit_struct newtype_struct seq tuple
        option tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a> MapAccess<'de> for ObjSeq<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if *self.index >= self.len {
            return Ok(None);
        }

        *self.index += 1;

        Ok(bubble_error!(
            seed.deserialize(ObjKeyDeserializer(&mut *self.deserializer))
                .map(Some),
            Context::Key
        ))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        Ok(bubble_error!(
            seed.deserialize(&mut *self.deserializer),
            Context::Key
        ))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - *self.index)
    }
}
