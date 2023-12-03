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
