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
