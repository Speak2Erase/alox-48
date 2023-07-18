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
use crate::value::{Object, RbArray, RbHash, RbString, Symbol, Userdata};

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
                let sequence = RbArray::deserialize(de)?;
                Ok(Value::Array(sequence))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(map);
                let mapping = RbHash::deserialize(de)?;
                Ok(Value::Hash(mapping))
            }
        }

        impl<'de> VisitorExt<'de> for ValueVisitor {
            fn visit_object<E>(self, object: Object) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Object(object))
            }

            fn visit_userdata<E>(self, userdata: Userdata) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Userdata(userdata))
            }

            fn visit_symbol<E>(self, sym: Symbol) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Symbol(sym))
            }

            fn visit_ruby_string<E>(self, string: RbString) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(string))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

pub struct ValueDeserializer<E> {
    value: Value,
    fuck: std::marker::PhantomData<E>,
}

impl<'de, E> serde::de::IntoDeserializer<'de, E> for Value
where
    E: serde::de::Error,
{
    type Deserializer = ValueDeserializer<E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self,
            fuck: std::marker::PhantomData,
        }
    }
}

impl<'de, E> serde::Deserializer<'de> for ValueDeserializer<E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: VisitorExt<'de>,
    {
        match self.value {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Float(v) => visitor.visit_f64(v),
            Value::Integer(v) => visitor.visit_i64(v),
            Value::String(v) => visitor.visit_ruby_string(v),
            Value::Symbol(v) => visitor.visit_symbol(v),
            Value::Array(v) => {
                let seq = serde::de::value::SeqDeserializer::new(v.into_iter());
                visitor.visit_seq(seq)
            }
            Value::Hash(v) => {
                let map = serde::de::value::MapDeserializer::new(v.into_iter());
                visitor.visit_map(map)
            }
            Value::Userdata(v) => visitor.visit_userdata(v),
            Value::Object(v) => visitor.visit_object(v),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf unit unit_struct newtype_struct seq tuple
        option tuple_struct map struct enum identifier ignored_any
    }
}
