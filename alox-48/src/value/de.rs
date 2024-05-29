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
    de::{Error, Result},
    ArrayAccess, Deserialize, DeserializerTrait, HashAccess, Instance, InstanceAccess, IvarAccess,
    Object, RbFields, RbHash, RbString, Sym, Userdata, Value, Visitor, VisitorInstance,
    VisitorOption,
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

    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        Ok(Value::String(RbString {
            data: string.to_vec(),
        }))
    }

    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Symbol(symbol.to_symbol()))
    }

    fn visit_regular_expression(self, data: &'de [u8], flags: u8) -> Result<Self::Value> {
        Ok(Value::Regex {
            data: RbString::from(data),
            flags,
        })
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

    fn visit_struct<A>(self, name: &'de Sym, mut members: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let mut fields = RbFields::with_capacity(members.len());
        while let Some((k, v)) = members.next_entry()? {
            fields.insert(k.to_symbol(), v);
        }
        Ok(Value::RbStruct(crate::RbStruct {
            class: name.to_symbol(),
            fields,
        }))
    }

    fn visit_class(self, class: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Class(class.to_symbol()))
    }

    fn visit_module(self, module: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Module(module.to_symbol()))
    }

    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        let (value, mut instance_fields) = instance.value(ValueVisitor)?;
        let mut fields = RbFields::with_capacity(instance_fields.len());
        while let Some((field, value)) = instance_fields.next_entry()? {
            fields.insert(field.to_symbol(), value);
        }
        let instance = Instance {
            value: Box::new(value),
            fields,
        };

        Ok(Value::Instance(instance))
    }

    fn visit_extended<D>(self, module: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::Extended {
            module: module.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::UserClass {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> Result<Self::Value> {
        Ok(Value::Userdata(Userdata {
            class: class.to_symbol(),
            data: data.to_vec(),
        }))
    }

    fn visit_user_marshal<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::UserMarshal {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_data<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::Data {
            class: class.to_symbol(),
            value: Box::new(value),
        })
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

struct ValueInstanceAccess<'de> {
    value: &'de Value,
    fields: &'de RbFields,
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
            Value::String(s) => visitor.visit_string(&s.data),
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
            Value::Instance(i) => visitor.visit_instance(ValueInstanceAccess {
                value: &i.value,
                fields: &i.fields,
            }),
            Value::Regex { data, flags } => visitor.visit_regular_expression(&data.data, *flags),
            Value::RbStruct(s) => visitor.visit_struct(
                &s.class,
                ValueIVarAccess {
                    fields: &s.fields,
                    index: 0,
                },
            ),
            Value::Class(c) => visitor.visit_class(c),
            Value::Module(m) => visitor.visit_module(m),
            Value::Extended { module, value } => visitor.visit_extended(module, value.as_ref()),
            Value::UserClass { class, value } => visitor.visit_user_class(class, value.as_ref()),
            Value::UserMarshal { class, value } => {
                visitor.visit_user_marshal(class, value.as_ref())
            }
            Value::Data { class, value } => visitor.visit_data(class, value.as_ref()),
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

    fn deserialize_instance<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorInstance<'de>,
    {
        if let Value::Instance(i) = self {
            visitor.visit_instance(ValueInstanceAccess {
                value: &i.value,
                fields: &i.fields,
            })
        } else {
            visitor.visit(self)
        }
    }
}

impl<'de> InstanceAccess<'de> for ValueInstanceAccess<'de> {
    type IvarAccess = ValueIVarAccess<'de>;

    fn value<V>(self, visitor: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: Visitor<'de>,
    {
        let value = self.value.deserialize(visitor)?;
        let access = ValueIVarAccess {
            fields: self.fields,
            index: 0,
        };
        Ok((value, access))
    }

    fn value_deserialize<T>(self) -> Result<(T, Self::IvarAccess)>
    where
        T: Deserialize<'de>,
    {
        let value = T::deserialize(self.value)?;
        let access = ValueIVarAccess {
            fields: self.fields,
            index: 0,
        };
        Ok((value, access))
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
