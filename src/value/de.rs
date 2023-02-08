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

use serde::{de::Visitor, Deserialize, Deserializer};

use crate::de::VisitorExt;
use crate::value::{Object, RbArray, RbFields, RbHash, RbString, Userdata};

use super::{Symbol, Value};

impl<'de> Deserialize<'de> for Value {
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
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
                D: Deserializer<'de>,
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
            fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(fields);
                let fields = RbFields::deserialize(de)?;
                Ok(Value::Object(Object {
                    class: class.into(),
                    fields,
                }))
            }

            fn visit_userdata<E>(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Userdata(Userdata {
                    class: class.into(),
                    data: data.to_vec(),
                }))
            }

            fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Symbol(sym.into()))
            }

            fn visit_ruby_string<A>(
                self,
                str: &'de [u8],
                fields: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(fields);
                let fields = RbFields::deserialize(de)?;

                Ok(Value::String(RbString {
                    data: str.to_vec(),
                    fields,
                }))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SymbolVisitor;

        impl<'de> Visitor<'de> for SymbolVisitor {
            type Value = Symbol;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("symbol")
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

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }
        }

        impl<'de> VisitorExt<'de> for SymbolVisitor {
            fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(sym.into())
            }
        }

        deserializer.deserialize_any(SymbolVisitor)
    }
}

impl<'de> Deserialize<'de> for RbString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
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

        impl<'de> VisitorExt<'de> for StringVisitor {
            fn visit_ruby_string<A>(
                self,
                str: &'de [u8],
                fields: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(fields);
                let fields = RbFields::deserialize(de)?;

                Ok(RbString {
                    data: str.to_vec(),
                    fields,
                })
            }
        }

        deserializer.deserialize_any(StringVisitor)
    }
}

impl<'de> Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ObjectVisitor;

        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = Object;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("object")
            }
        }

        impl<'de> VisitorExt<'de> for ObjectVisitor {
            fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(fields);
                let fields = RbFields::deserialize(de)?;
                Ok(Object {
                    class: class.into(),
                    fields,
                })
            }
        }

        deserializer.deserialize_any(ObjectVisitor)
    }
}

impl<'de> Deserialize<'de> for Userdata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UserdataVisitor;

        impl<'de> Visitor<'de> for UserdataVisitor {
            type Value = Userdata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("userdata")
            }
        }

        impl<'de> VisitorExt<'de> for UserdataVisitor {
            fn visit_userdata<E>(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Userdata {
                    class: class.into(),
                    data: data.to_vec(),
                })
            }
        }

        deserializer.deserialize_any(UserdataVisitor)
    }
}
