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
#![allow(dead_code, unused_variables)]

use std::borrow::Cow;

use serde::de;
use serde::forward_to_deserialize_any;

use super::VisitorExt;
use crate::Error;
use crate::Result;

pub struct Deserializer<'de> {
    input: &'de [u8],
    cursor: usize,
    objtable: Vec<usize>,
    symlink: Vec<&'de str>,
    remove_ivar_prefix: bool,
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de [u8]) -> crate::Result<Self> {
        Ok(Self {
            input,
            cursor: 2,
            objtable: vec![],
            symlink: vec![],
            remove_ivar_prefix: false,
        })
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Message("aaa".to_string()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
