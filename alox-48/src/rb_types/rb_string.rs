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

use crate::{
    de::Result as DeResult, ser::Result as SerResult, Deserialize, DeserializerTrait, Serialize,
    SerializerTrait, Visitor,
};

/// A type equivalent to ruby's `String`.
/// ruby strings do not have to be utf8 encoded, so this type uses [`Vec<u8>`] instead.
#[derive(PartialEq, Eq, Default, Clone)]
pub struct RbString {
    /// The data of this string.
    pub data: Vec<u8>,
}

#[allow(clippy::must_use_candidate)]
impl RbString {
    /// Uses [`String::from_utf8_lossy`] to convert this string to rust string in a lossy manner.
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }

    /// Tries to convert this string to a rust string.
    ///
    /// # Errors
    /// Errors when this string is not valid utf8.
    pub fn to_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data)
    }

    /// Get the length of the string data.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the string data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the string data as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for RbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RbString")
            .field("data", &self.to_string_lossy())
            .finish()
    }
}

impl std::fmt::Display for RbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_lossy())
    }
}

impl<T> PartialEq<T> for RbString
where
    [u8]: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data.as_slice().eq(other)
    }
}

impl std::borrow::Borrow<[u8]> for RbString {
    fn borrow(&self) -> &[u8] {
        &self.data
    }
}

impl std::borrow::BorrowMut<[u8]> for RbString {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl From<&str> for RbString {
    fn from(value: &str) -> Self {
        Self {
            data: value.as_bytes().to_vec(),
        }
    }
}

impl From<String> for RbString {
    fn from(value: String) -> Self {
        Self {
            data: value.into_bytes(),
        }
    }
}

impl From<&[u8]> for RbString {
    fn from(value: &[u8]) -> Self {
        Self {
            data: value.to_vec(),
        }
    }
}

impl From<Vec<u8>> for RbString {
    fn from(value: Vec<u8>) -> Self {
        Self { data: value }
    }
}

struct StringVisitor;

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = &'de [u8];

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a ruby string")
    }

    fn visit_string(self, string: &'de [u8]) -> DeResult<Self::Value> {
        Ok(string)
    }
}

impl<'de> Visitor<'de> for StringVisitor {
    type Value = RbString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a ruby string")
    }

    fn visit_string(self, string: &'de [u8]) -> DeResult<Self::Value> {
        Ok(RbString {
            data: string.to_vec(),
        })
    }
}

impl<'de> Deserialize<'de> for RbString {
    fn deserialize<D>(deserializer: D) -> DeResult<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(StringVisitor)
    }
}

impl Serialize for RbString {
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_string(&self.data)
    }
}
