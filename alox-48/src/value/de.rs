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
use crate::{
    de::{InstanceAccess, Result},
    ArrayAccess, Deserialize, DeserializerTrait, HashAccess, IvarAccess, Object, RbFields, RbHash,
    RbString, Sym, Userdata, Value, Visitor,
};

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any ruby value")
    }

    fn visit_nil(self) -> Result<Self::Value> {
        Ok(Value::Nil)
    }

    fn visit_bool(self, v: bool) -> Result<Self::Value> {
        Ok(Value::Bool(v))
    }

    fn visit_i32(self, v: i32) -> Result<Self::Value> {
        Ok(Value::Integer(v))
    }

    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        Ok(Value::Float(v))
    }

    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        Ok(Value::String(RbString {
            data: string.to_vec(),
            fields: RbFields::new(),
        }))
    }

    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Symbol(symbol.to_symbol()))
    }

    fn visit_array<A>(self, mut access: A) -> Result<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        let mut array = Vec::with_capacity(access.len());
        while let Some(v) = access.next_element()? {
            array.push(v);
        }
        Ok(Value::Array(array))
    }

    fn visit_hash<A>(self, mut map: A) -> Result<Self::Value>
    where
        A: HashAccess<'de>,
    {
        let mut hash = RbHash::with_capacity(map.len());
        while let Some((k, v)) = map.next_entry()? {
            hash.insert(k, v);
        }
        Ok(Value::Hash(hash))
    }

    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> Result<Self::Value> {
        Ok(Value::Userdata(Userdata {
            class: class.to_symbol(),
            data: data.to_vec(),
        }))
    }

    fn visit_object<A>(self, class: &'de Sym, mut instance_variables: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let mut fields = RbFields::with_capacity(instance_variables.len());
        while let Some((k, v)) = instance_variables.next_entry()? {
            fields.insert(k.to_symbol(), v);
        }
        Ok(Value::Object(Object {
            class: class.to_symbol(),
            fields,
        }))
    }

    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        let (mut value, mut instance_fields) = instance.value(ValueVisitor)?;
        if let Value::String(RbString { fields, .. }) = &mut value {
            fields.reserve(instance_fields.len());
            while let Some((k, v)) = instance_fields.next_entry()? {
                fields.insert(k.to_symbol(), v);
            }
        }
        Ok(value)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(ValueVisitor)
    }
}
