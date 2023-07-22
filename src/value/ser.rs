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
use crate::ser::{Error, Kind, Result, SerializeExt};

#[allow(clippy::panic_in_result_fn)]
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: SerializeExt,
    {
        match self {
            Value::Nil => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Integer(i) => serializer.serialize_i64(*i),
            Value::String(s) => serializer.serialize_ruby_string(s),
            Value::Symbol(s) => serializer.serialize_symbol(s),
            Value::Array(a) => a.serialize(serializer),
            Value::Hash(h) => h.serialize(serializer),
            Value::Userdata(u) => u.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
        }
    }
}

macro_rules! serialize_int {
    ($($int:ty),*) => {
        paste::paste! {
            $(
                fn [<serialize_ $int>](self, v: $int) -> Result<Self::Ok> {
                    Ok(Value::Integer(v as i64))
                }
            )*
        }
    };
}

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs [`alox_48::value::to_value`].
/// Unlike the main alox-48 serializer which goes from some value of `T` to binary data,
/// this one goes from `T` to `alox_48::value::Value`.
#[derive(Clone, Copy, Debug)]
pub struct Serializer;

impl serde::ser::Serializer for Serializer {
    type Ok = Value;

    type Error = Error;

    type SerializeSeq = SerializeVec;

    type SerializeTuple = SerializeVec;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = SerializeMap;

    type SerializeStruct = SerializeObject;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Value::Bool(v))
    }

    serialize_int! {
        i8, i16, i32, i64,
        u8, u16, u32, u64
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Value::Float(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let str = String::from(v);
        Ok(Value::String(str.into()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        eprintln!("warning: serializing bytes is unclear, it will be serialized as a raw string");

        Ok(Value::String(v.into()))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Value::Nil)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Value::Nil)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        T::serialize(value, self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        eprintln!("warning: unit structs do not map well to ruby. serializing as nil");

        Ok(Value::Nil)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error {
            kind: Kind::Unsupported("newtype struct"),
        })
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        let values = match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        };
        Ok(SerializeVec { values })
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeVec {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error {
            kind: Kind::Unsupported("tuple struct"),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        let values = match len {
            Some(len) => RbHash::with_capacity(len),
            None => RbHash::new(),
        };
        Ok(SerializeMap { values, key: None })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeObject {
            class: name.into(),
            fields: RbFields::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error {
            kind: Kind::Unsupported("enums"),
        })
    }
}

impl SerializeExt for Serializer {
    fn serialize_object(
        self,
        class: &Symbol,
        len: usize,
    ) -> std::result::Result<Self::SerializeObject, Self::Error> {
        Ok(SerializeObject {
            class: class.clone(),
            fields: RbFields::with_capacity(len),
        })
    }

    fn serialize_ruby_string(
        self,
        string: &RbString,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::String(string.clone()))
    }

    fn serialize_userdata(
        self,
        class: &Symbol,
        data: &[u8],
    ) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::Userdata(Userdata {
            class: class.clone(),
            data: data.to_vec(),
        }))
    }

    fn serialize_symbol(self, symbol: &Symbol) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::Symbol(symbol.clone()))
    }
}

#[allow(missing_debug_implementations)]
pub struct SerializeVec {
    values: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let value = T::serialize(value, Serializer)?;
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.values))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[allow(missing_debug_implementations)]
pub struct SerializeMap {
    values: RbHash,
    key: Option<Value>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.key = Some(T::serialize(key, Serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let key = self.key.take().expect("key missing?");
        let value = T::serialize(value, Serializer)?;
        self.values.insert(key, value);

        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::Hash(self.values))
    }
}

#[allow(missing_debug_implementations)]
pub struct SerializeObject {
    class: Symbol,
    fields: RbFields,
}

impl serde::ser::SerializeStruct for SerializeObject {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let key = key.into();
        let value = T::serialize(value, Serializer)?;
        self.fields.insert(key, value);

        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Value::Object(Object {
            class: self.class,
            fields: self.fields,
        }))
    }
}

impl crate::ser::SerializeObject for SerializeObject {
    fn serialize_field<T: ?Sized>(
        &mut self,
        field: &Symbol,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let field = field.clone();
        let value = T::serialize(value, Serializer)?;
        self.fields.insert(field, value);

        Ok(())
    }
}
