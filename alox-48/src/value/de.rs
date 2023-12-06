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
    de::Error, de::Result, ArrayAccess, Deserialize, DeserializerTrait, HashAccess, InstanceAccess,
    IvarAccess, Object, RbFields, RbHash, RbString, Sym, Userdata, Value, Visitor, VisitorOption,
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

enum ValueInstanceAccess<'de> {
    String {
        string: &'de [u8],
        fields: &'de RbFields,
    },
}

struct ValueIVarAccess<'de> {
    fields: &'de RbFields,
    index: usize,
}

struct ValueArrayAccess<'de> {
    array: &'de [Value],
    index: usize,
}

struct ValueHashAccess<'de> {
    hash: &'de RbHash,
    index: usize,
}

impl<'de> DeserializerTrait<'de> for &'de Value {
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_nil(),
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Float(f) => visitor.visit_f64(*f),
            Value::Integer(i) => visitor.visit_i32(*i),
            Value::String(s) => {
                if s.fields.is_empty() {
                    visitor.visit_string(s.data.as_slice())
                } else {
                    visitor.visit_instance(ValueInstanceAccess::String {
                        string: &s.data,
                        fields: &s.fields,
                    })
                }
            }
            Value::Symbol(s) => visitor.visit_symbol(s),
            Value::Array(array) => visitor.visit_array(ValueArrayAccess { array, index: 0 }),
            Value::Hash(hash) => visitor.visit_hash(ValueHashAccess { hash, index: 0 }),
            Value::Userdata(u) => visitor.visit_user_data(&u.class, &u.data),
            Value::Object(o) => visitor.visit_object(
                &o.class,
                ValueIVarAccess {
                    fields: &o.fields,
                    index: 0,
                },
            ),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorOption<'de>,
    {
        if matches!(self, Value::Nil) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
}

impl<'de> InstanceAccess<'de> for ValueInstanceAccess<'de> {
    type IvarAccess = ValueIVarAccess<'de>;

    fn value<V>(self, visitor: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String { string, fields } => {
                let value = visitor.visit_string(string)?;
                Ok((value, ValueIVarAccess { fields, index: 0 }))
            }
        }
    }
}

impl<'de> IvarAccess<'de> for ValueIVarAccess<'de> {
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>> {
        let Some((field, _)) = self.fields.get_index(self.index) else {
            return Ok(None);
        };
        Ok(Some(field))
    }

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        let (_, value) = self
            .fields
            .get_index(self.index)
            .ok_or_else(|| Error::custom("out of values".to_string()))?;
        self.index += 1;

        T::deserialize(value)
    }

    fn len(&self) -> usize {
        self.fields.len()
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl<'de> ArrayAccess<'de> for ValueArrayAccess<'de> {
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>,
    {
        let Some(value) = self.array.get(self.index) else {
            return Ok(None);
        };
        self.index += 1;
        T::deserialize(value).map(Some)
    }

    fn len(&self) -> usize {
        self.array.len()
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl<'de> HashAccess<'de> for ValueHashAccess<'de> {
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>,
    {
        let Some((key, _)) = self.hash.get_index(self.index) else {
            return Ok(None);
        };
        K::deserialize(key).map(Some)
    }

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        let (_, value) = self
            .hash
            .get_index(self.index)
            .ok_or_else(|| Error::custom("out of values".to_string()))?;
        self.index += 1;

        T::deserialize(value)
    }

    fn len(&self) -> usize {
        self.hash.len()
    }

    fn index(&self) -> usize {
        self.index
    }
}
