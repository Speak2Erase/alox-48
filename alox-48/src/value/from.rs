// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::Instance;

use super::{Object, RbHash, RbString, Symbol, Userdata, Value};

impl Value {
    /// Convert a symbol into a value.
    #[must_use]
    pub fn from_symbol(symbol: String) -> Self {
        Self::Symbol(symbol.into())
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Nil,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        // FIXME should these use instance?
        Value::String(RbString {
            data: value.into_bytes(),
        })
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(RbString {
            data: value.to_string().into_bytes(),
        })
    }
}

impl From<RbString> for Value {
    fn from(value: RbString) -> Self {
        Self::String(value)
    }
}

impl From<Symbol> for Value {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl From<RbHash> for Value {
    fn from(value: RbHash) -> Self {
        Self::Hash(value)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect())
    }
}

impl TryInto<String> for Value {
    type Error = Self;

    fn try_into(self) -> Result<String, Self::Error> {
        self.into_string()
            .map(|str| str.to_string_lossy().into_owned())
    }
}

impl TryInto<RbString> for Value {
    type Error = Self;

    fn try_into(self) -> Result<RbString, Self::Error> {
        self.into_string()
    }
}

impl TryInto<i32> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i32, Self::Error> {
        self.into_integer()
    }
}

impl TryInto<f64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<f64, Self::Error> {
        self.into_float()
    }
}

impl TryInto<Object> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Object, Self::Error> {
        self.into_object()
    }
}

impl TryInto<Userdata> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Userdata, Self::Error> {
        self.into_userdata()
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Nil => false,
            Value::Bool(b) => b,
            _ => true,
        }
    }
}

impl<T> From<Instance<T>> for Value
where
    T: Into<Value>,
{
    fn from(instance: Instance<T>) -> Self {
        let value = Box::new(instance.value.into());
        Value::Instance(Instance {
            value,
            fields: instance.fields,
        })
    }
}
