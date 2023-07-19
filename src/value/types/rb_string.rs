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
use super::RbFields;
use crate::DeError;

/// A type equivalent to ruby's `String`.
/// ruby strings do not have to be utf8 encoded, so this type uses [`Vec<u8>`] instead.
///
/// ruby strings also can have attached extra fields (usually just the encoding), and this struct is no exception.
/// An [`RbString`] constructed from a rust [`String`] will always have the field `:E` set to true, which is how
/// ruby denotes that a string is utf8.
#[derive(PartialEq, Eq, Default, Clone)]
pub struct RbString {
    /// The data of this string.
    pub data: Vec<u8>,
    /// Extra fields associated with this string.
    pub fields: RbFields,
}

impl RbString {
    #[must_use]
    /// Return the encoding of this string, if it has one.
    pub fn encoding(&self) -> Option<&crate::Value> {
        self.fields.get("E").or_else(|| self.fields.get("encoding"))
    }

    #[must_use]
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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl serde::Serialize for RbString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::SerializeExt,
    {
        serializer.serialize_ruby_string(self)
    }
}

impl<'de> serde::Deserialize<'de> for RbString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> serde::de::Visitor<'de> for StringVisitor {
            type Value = RbString;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a ruby string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }
        }

        impl<'de> crate::VisitorExt<'de> for StringVisitor {
            fn visit_ruby_string(self, string: RbString) -> Result<Self::Value, DeError> {
                Ok(string)
            }
        }

        deserializer.deserialize_any(StringVisitor)
    }
}

impl std::fmt::Debug for RbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RbString")
            .field("data", &self.to_string_lossy())
            .field("fields", &self.fields)
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

macro_rules! utf8_enc {
    () => {{
        let mut f = RbFields::new();
        f.insert("E".into(), true.into());

        f
    }};
}

impl From<&str> for RbString {
    fn from(value: &str) -> Self {
        let fields = utf8_enc!();

        Self {
            data: value.as_bytes().to_vec(),
            fields,
        }
    }
}

impl From<String> for RbString {
    fn from(value: String) -> Self {
        let fields = utf8_enc!();

        Self {
            data: value.into_bytes(),
            fields,
        }
    }
}

impl From<&[u8]> for RbString {
    fn from(value: &[u8]) -> Self {
        Self {
            data: value.to_vec(),
            fields: Default::default(),
        }
    }
}

impl From<Vec<u8>> for RbString {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
            fields: Default::default(),
        }
    }
}
