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

use super::{Error, Kind, Result};
use crate::tag::Tag;

/// The alox-48 deserializer.
#[derive(Debug, Clone)]
pub struct Deserializer<'de> {
    cursor: Cursor<'de>,

    objtable: Vec<usize>,
    stack: Vec<usize>,
    blacklisted_objects: BTreeSet<usize>,

    sym_table: Vec<&'de str>,
}

#[derive(Debug, Clone)]
struct Cursor<'de> {
    input: &'de [u8],
    position: usize,
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

macro_rules! deserialize_len {
    ($this:expr) => {{
        let raw_length = $this.read_packed_int()?;
        raw_length.try_into().map_err(|_| Error {
            kind: Kind::UnexpectedNegativeLength(raw_length),
        })?
    }};
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
        let length = deserialize_len!(self);
        let out = self.cursor.next_bytes_dyn(length)?;

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

    fn read_symbol(&mut self) -> Result<&'de str> {
        let length = deserialize_len!(self);
        let out = self.cursor.next_bytes_dyn(length)?;

        let str = match std::str::from_utf8(out) {
            Ok(a) => a,
            Err(err) => {
                return Err(Error {
                    kind: Kind::SymbolInvalidUTF8(err),
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
        })
    }

    // FIXME: FIND BETTER NAME
    fn read_symbol_either(&mut self) -> Result<&'de str> {
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
}
