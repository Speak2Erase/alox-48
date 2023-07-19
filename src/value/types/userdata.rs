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
use super::Symbol;
use crate::DeError;

/// This type represents types serialized with `_dump` from ruby.
/// Its main intended use is in [`Value`], but you can also use it with [`serde::Deserialize`]:
///
/// ```
/// #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
/// #[serde(from = "alox_48::Userdata")]
/// struct MyUserData {
///     field: [char; 4],
/// }
///
/// impl From<alox_48::Userdata> for MyUserData {
///     fn from(value: alox_48::Userdata) -> Self {
///         assert_eq!(value.class, "MyUserData");
///         let field = std::array::from_fn(|i| {
///             value.data[i] as char
///         });
///
///         Self {
///             field
///         }
///     }
/// }
///
/// let bytes = &[
///     0x04, 0x08, 0x75, 0x3a, 0x0f, 0x4d, 0x79, 0x55, 0x73, 0x65, 0x72, 0x44, 0x61, 0x74, 0x61, 0x09, 0x61, 0x62, 0x63, 0x64
/// ];
///
/// let data: MyUserData = alox_48::from_bytes(bytes).expect("invalid marshal data");
/// assert_eq!(
///     data,
///     MyUserData {
///         field: ['a', 'b', 'c', 'd']
///     }
/// )
///     
///
/// ```
#[derive(Hash, PartialEq, Eq, Default, Debug, Clone)]
pub struct Userdata {
    /// Userdata class.
    pub class: Symbol,
    /// Userdata data.
    pub data: Vec<u8>,
}

impl serde::Serialize for Userdata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::SerializeExt,
    {
        serializer.serialize_userdata(&self.class, &self.data)
    }
}

impl<'de> serde::Deserialize<'de> for Userdata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UserdataVisitor;

        impl<'de> serde::de::Visitor<'de> for UserdataVisitor {
            type Value = Userdata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("userdata")
            }
        }

        impl<'de> crate::VisitorExt<'de> for UserdataVisitor {
            fn visit_userdata(
                self,
                class: &'de str,
                data: &'de [u8],
            ) -> Result<Self::Value, DeError> {
                Ok(Userdata {
                    class: class.into(),
                    data: data.to_vec(),
                })
            }
        }

        deserializer.deserialize_any(UserdataVisitor)
    }
}
