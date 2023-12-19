// Copyright (C) 2022 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
use super::{Object, RbFields, RbHash, RbString, Symbol, Userdata, Value};
use crate::{
    ser::{Error, Kind, Result, Result as SerResult, Serialize},
    RbArray, SerializerTrait, Sym,
};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
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
        }
    }
}

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs [`alox_48::value::to_value`].
/// Unlike the main alox-48 serializer which goes from some value of `T` to binary data,
/// this one goes from `T` to `alox_48::value::Value`.
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
    type SerializeArray = SerializeArray;
    type SerializeHash = SerializeHash;

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
            fields: RbFields::new(),
        }))
    }

    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok> {
        Ok(Value::Symbol(sym.to_symbol()))
    }

    fn serialize_regular_expression(self, _regex: &[u8], _flags: u8) -> Result<Self::Ok> {
        todo!()
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

    fn serialize_class(self, _class: &Sym) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_module(self, _module: &Sym) -> Result<Self::Ok> {
        todo!()
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

    fn serialize_extended<V>(self, _module: &Sym, _value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        todo!()
    }

    fn serialize_user_class<V>(self, _class: &Sym, _value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        todo!()
    }

    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok> {
        Ok(Value::Userdata(Userdata {
            class: class.to_symbol(),
            data: data.to_vec(),
        }))
    }

    fn serialize_user_marshal<V>(self, _class: &Sym, _value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        todo!()
    }

    fn serialize_data<V>(self, _class: &Sym, _value: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        todo!()
    }
}

impl crate::SerializeIvars for SerializeIvars {
    type Ok = Value;

    fn serialize_field(&mut self, k: &Sym) -> Result<()> {
        self.next_field = Some(k.to_symbol());
        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        let field = self.next_field.take().ok_or_else(|| Error {
            kind: Kind::Message("serialized value before field".to_string()),
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
            SerializeIvarsValue::Instance(_) => todo!(),
            SerializeIvarsValue::Struct(_) => todo!(),
        }
    }
}

impl crate::SerializeHash for SerializeHash {
    type Ok = Value;

    fn serialize_key<K>(&mut self, k: &K) -> Result<()>
    where
        K: Serialize + ?Sized,
    {
        let value = k.serialize(Serializer)?;
        self.next_key = Some(value);
        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: Serialize + ?Sized,
    {
        let key = self.next_key.take().ok_or_else(|| Error {
            kind: Kind::Message("serialized value before key".to_string()),
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
