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
use super::{RbFields, Symbol};
use crate::DeError;

/// A type equivalent to ruby's `Object`.
#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct Object {
    /// This object's class.
    pub class: Symbol,
    /// The fields on this object.
    pub fields: RbFields,
}

impl serde::Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::SerializeExt,
    {
        use crate::ser::SerializeObject;

        let mut s = serializer.serialize_object(&self.class, self.fields.len())?;

        for (k, v) in &self.fields {
            s.serialize_field(k, v)?;
        }
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ObjectVisitor;

        impl<'de> serde::de::Visitor<'de> for ObjectVisitor {
            type Value = Object;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("object")
            }
        }

        impl<'de> crate::VisitorExt<'de> for ObjectVisitor {
            fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, DeError>
            where
                A: serde::de::MapAccess<'de, Error = DeError>,
            {
                let fields = serde::Deserialize::deserialize(
                    serde::de::value::MapAccessDeserializer::new(fields),
                )?;
                Ok(Object {
                    class: class.into(),
                    fields,
                })
            }
        }

        deserializer.deserialize_any(ObjectVisitor)
    }
}

impl<'de> serde::Deserializer<'de> for &'de Object {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: crate::VisitorExt<'de>,
    {
        let fields = serde::de::value::MapDeserializer::new(self.fields.iter());
        visitor.visit_object(self.class.as_str(), fields)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf unit unit_struct newtype_struct seq tuple
        option tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> serde::de::IntoDeserializer<'de, DeError> for &'de Object {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.fields.len().hash(state);

        for (var, field) in &self.fields {
            var.hash(state);
            field.hash(state);
        }
    }
}
