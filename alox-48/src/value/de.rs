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

use serde::Deserialize;

use crate::de::VisitorExt;
use crate::value::{Object, RbString, Userdata};
use crate::DeError;

use super::Value;

impl<'de> serde::Deserialize<'de> for Value {
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("any ruby value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.into()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.into()))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let de = serde::de::value::SeqAccessDeserializer::new(seq);
                let sequence = Deserialize::deserialize(de)?;
                Ok(Value::Array(sequence))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(map);
                let mapping = Deserialize::deserialize(de)?;
                Ok(Value::Hash(mapping))
            }
        }

        impl<'de> VisitorExt<'de> for ValueVisitor {
            fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, DeError>
            where
                A: serde::de::MapAccess<'de, Error = DeError>,
            {
                let fields =
                    Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(fields))?;
                Ok(Value::Object(Object {
                    class: class.into(),
                    fields,
                }))
            }

            fn visit_userdata(
                self,
                class: &'de str,
                data: &'de [u8],
            ) -> Result<Self::Value, DeError> {
                Ok(Value::Userdata(Userdata {
                    class: class.into(),
                    data: data.to_vec(),
                }))
            }

            fn visit_symbol(self, sym: &'de str) -> Result<Self::Value, DeError> {
                Ok(Value::Symbol(sym.into()))
            }

            fn visit_ruby_string<A>(
                self,
                data: &'de [u8],
                fields: A,
            ) -> Result<Self::Value, DeError>
            where
                A: serde::de::MapAccess<'de, Error = DeError>,
            {
                let fields =
                    Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(fields))?;
                Ok(Value::String(RbString {
                    data: data.to_vec(),
                    fields,
                }))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

// This is a little hacky and I'm unsure how I feel about it.
// It could be solved by making Value borrow from the deserializer data, although that feels iffy in itself.
impl<'de> serde::de::IntoDeserializer<'de, DeError> for &'de Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for &'de Value {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: VisitorExt<'de>,
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Float(v) => visitor.visit_f64(*v),
            Value::Integer(v) => visitor.visit_i64(*v),
            Value::String(v) => {
                let fields = serde::de::value::MapDeserializer::new(v.fields.iter());
                visitor.visit_ruby_string(&v.data, fields)
            }
            Value::Symbol(v) => visitor.visit_symbol(v.as_str()),
            Value::Array(v) => {
                let seq = serde::de::value::SeqDeserializer::new(v.iter());
                visitor.visit_seq(seq)
            }
            Value::Hash(v) => {
                let map = serde::de::value::MapDeserializer::new(v.iter());
                visitor.visit_map(map)
            }
            Value::Userdata(v) => visitor.visit_userdata(v.class.as_str(), &v.data),
            Value::Object(v) => {
                let fields = serde::de::value::MapDeserializer::new(v.fields.iter());
                visitor.visit_object(v.class.as_str(), fields)
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf unit unit_struct newtype_struct seq tuple
        option tuple_struct map struct enum identifier ignored_any
    }
}
