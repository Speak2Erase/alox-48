// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{Object, RbFields, RbHash, RbString, Symbol, Userdata, Value};
use crate::{
    ser::{Error, Kind, Result, Serialize},
    Instance, RbArray, RbStruct, SerializerTrait, Sym,
};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        match self {
            Value::Nil => serializer.serialize_nil(),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Integer(i) => serializer.serialize_i32(*i),
            Value::String(s) => s.serialize(serializer),
            Value::Symbol(s) => s.serialize(serializer),
            Value::Array(a) => a.serialize(serializer),
            Value::Hash(h) => h.serialize(serializer),
            Value::Userdata(d) => d.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
            Value::Instance(i) => i.serialize(serializer),
            Value::RbStruct(s) => s.serialize(serializer),
            Value::Class(c) => serializer.serialize_class(c),
            Value::Module(m) => serializer.serialize_module(m),
            Value::Extended { module, value } => serializer.serialize_extended(module, value),
            Value::UserClass { class, value } => serializer.serialize_user_class(class, value),
            Value::UserMarshal { class, value } => serializer.serialize_user_marshal(class, value),
            Value::Data { class, value } => serializer.serialize_data(class, value),
            Value::Regex { data, flags } => {
                serializer.serialize_regular_expression(data.as_slice(), *flags)
            }
        }
    }
}

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs `to_value`.
/// Unlike the main alox-48 serializer which goes from some value of `T` to binary data,
/// this one goes from `T` to `Value`.
#[derive(Clone, Copy, Debug)]
pub struct Serializer;

#[derive(Debug)]
pub struct SerializeIvars {
    fields: RbFields,
    next_field: Option<Symbol>,
    value: SerializeIvarsValue,
}

#[derive(Debug)]
enum SerializeIvarsValue {
    Instance(Value),
    Object(Symbol),
    Struct(Symbol),
}

#[derive(Debug)]
pub struct SerializeHash {
    hash: RbHash,
    next_key: Option<Value>,
}

#[derive(Debug)]
pub struct SerializeArray(RbArray);

impl SerializerTrait for Serializer {
    type Ok = Value;

    type SerializeIvars = SerializeIvars;
    type SerializeHash = SerializeHash;
    type SerializeArray = SerializeArray;

    fn serialize_nil(self) -> Result<Self::Ok> {
        Ok(Value::Nil)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Value::Bool(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(Value::Integer(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Value::Float(v))
    }

    fn serialize_hash(self, len: usize) -> Result<Self::SerializeHash> {
        Ok(SerializeHash {
            hash: RbHash::with_capacity(len),
            next_key: None,
        })
    }

    fn serialize_array(self, len: usize) -> Result<Self::SerializeArray> {
        Ok(SerializeArray(Vec::with_capacity(len)))
    }

    fn serialize_string(self, data: &[u8]) -> Result<Self::Ok> {
        Ok(Value::String(RbString {
            data: data.to_vec(),
        }))
    }

    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok> {
        Ok(Value::Symbol(sym.to_symbol()))
    }

    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> Result<Self::Ok> {
        Ok(Value::Regex {
            data: RbString::from(regex),
            flags,
        })
    }

    fn serialize_object(self, class: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        Ok(SerializeIvars {
            fields: RbFields::with_capacity(len),
            next_field: None,
            value: SerializeIvarsValue::Object(class.to_symbol()),
        })
    }

    fn serialize_struct(self, name: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        Ok(SerializeIvars {
            fields: RbFields::with_capacity(len),
            next_field: None,
            value: SerializeIvarsValue::Struct(name.to_symbol()),
        })
    }

    fn serialize_class(self, class: &Sym) -> Result<Self::Ok> {
        Ok(Value::Class(class.to_symbol()))
    }

    fn serialize_module(self, module: &Sym) -> Result<Self::Ok> {
        Ok(Value::Module(module.to_symbol()))
    }

    fn serialize_instance<V>(self, value: &V, len: usize) -> Result<Self::SerializeIvars>
    where
        V: Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        Ok(SerializeIvars {
            fields: RbFields::with_capacity(len),
            next_field: None,
            value: SerializeIvarsValue::Instance(value),
        })
    }

    fn serialize_extended<V>(self, module: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        Ok(Value::Extended {
            module: module.to_symbol(),
            value: Box::new(value),
        })
    }

    fn serialize_user_class<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        Ok(Value::UserClass {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok> {
        Ok(Value::Userdata(Userdata {
            class: class.to_symbol(),
            data: data.to_vec(),
        }))
    }

    fn serialize_user_marshal<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        Ok(Value::UserMarshal {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn serialize_data<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        Ok(Value::Data {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }
}

impl crate::SerializeIvars for SerializeIvars {
    type Ok = Value;

    fn serialize_field(&mut self, k: &Sym) -> Result<()> {
        if self.next_field.is_some() {
            return Err(Error {
                kind: Kind::KeyAfterKey,
            });
        }
        self.next_field = Some(k.to_symbol());
        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        let field = self.next_field.take().ok_or(Error {
            kind: Kind::ValueAfterValue,
        })?;

        let value = v.serialize(Serializer)?;
        self.fields.insert(field, value);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match self.value {
            SerializeIvarsValue::Object(class) => Ok(Value::Object(Object {
                class,
                fields: self.fields,
            })),
            SerializeIvarsValue::Instance(value) => Ok(Value::Instance(Instance {
                value: Box::new(value),
                fields: self.fields,
            })),
            SerializeIvarsValue::Struct(class) => Ok(Value::RbStruct(RbStruct {
                class,
                fields: self.fields,
            })),
        }
    }
}

impl crate::SerializeHash for SerializeHash {
    type Ok = Value;

    fn serialize_key<K>(&mut self, k: &K) -> Result<()>
    where
        K: Serialize + ?Sized,
    {
        if self.next_key.is_some() {
            return Err(Error {
                kind: Kind::KeyAfterKey,
            });
        }
        let value = k.serialize(Serializer)?;
        self.next_key = Some(value);
        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        let key = self.next_key.take().ok_or(Error {
            kind: Kind::ValueAfterValue,
        })?;

        let value = v.serialize(Serializer)?;
        self.hash.insert(key, value);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Hash(self.hash))
    }
}

impl crate::SerializeArray for SerializeArray {
    type Ok = Value;

    fn serialize_element<T>(&mut self, v: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        let value = v.serialize(Serializer)?;
        self.0.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Array(self.0))
    }
}
