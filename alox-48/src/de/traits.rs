// Copyright (C) 2023 Lily Lyons
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
use super::{error::Unexpected, Error, Result};
use crate::Sym;

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: Deserializer<'de>;
}

pub trait Deserializer<'de>: Sized {
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorOption<'de>;
}

pub trait Visitor<'de>: Sized {
    type Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    // Primitives
    fn visit_nil(self) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Nil, &self))
    }
    fn visit_bool(self, v: bool) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Bool(v), &self))
    }
    fn visit_i32(self, v: i32) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Integer(v), &self))
    }
    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Float(v), &self))
    }

    // Collections
    fn visit_hash<A>(self, map: A) -> Result<Self::Value>
    where
        A: HashAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Hash, &self))
    }
    fn visit_array<A>(self, array: A) -> Result<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Array, &self))
    }
    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::String(string), &self))
    }
    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Symbol(symbol), &self))
    }
    fn visit_regular_expression(self, regex: &'de [u8], flags: u8) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Regex(regex), &self))
    }

    // Class instances types
    fn visit_object<A>(self, class: &'de Sym, instance_variables: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Object(class), &self))
    }
    fn visit_struct<A>(self, name: &'de Sym, members: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Struct(name), &self))
    }
    // Other
    fn visit_class(self, class: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Class(class), &self))
    }
    fn visit_module(self, module: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Module(module), &self))
    }

    // Extended/modified types
    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        // TODO: serde doesn't do antyhing like this. Maybe this is bad?
        // probably something to do with self-describing formats (which marshal is not)
        let (value, _) = instance.value(self)?;
        Ok(value)
    }
    fn visit_extended<D>(self, module: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }

    // User types
    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::UserData(class), &self))
    }
    fn visit_user_marshal<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
    fn visit_data<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
}

// todo investigate other ways of doing this
pub trait VisitorOption<'de> {
    type Value;

    fn visit_none(self) -> Result<Self::Value>;

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>;
}

pub trait InstanceAccess<'de>: Sized {
    type IvarAccess: IvarAccess<'de>;

    fn value<V>(self, visitor: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: Visitor<'de>;
}

pub trait IvarAccess<'de> {
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>>;

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>;

    fn next_entry<T>(&mut self) -> Result<Option<(&'de Sym, T)>>
    where
        T: Deserialize<'de>,
    {
        if let Some(var) = self.next_ivar()? {
            self.next_value().map(|v| Some((var, v)))
        } else {
            Ok(None)
        }
    }

    fn len(&self) -> usize;

    fn index(&self) -> usize;
}

pub trait HashAccess<'de> {
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>;

    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>;

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        if let Some(k) = self.next_key()? {
            self.next_value().map(|v| Some((k, v)))
        } else {
            Ok(None)
        }
    }

    fn len(&self) -> usize;

    fn index(&self) -> usize;
}

pub trait ArrayAccess<'de> {
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>;

    fn len(&self) -> usize;

    fn index(&self) -> usize;
}

impl<'de, 'a, A> IvarAccess<'de> for &'a mut A
where
    A: IvarAccess<'de>,
{
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>> {
        (**self).next_ivar()
    }

    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        (**self).next_value()
    }

    fn next_entry<T>(&mut self) -> Result<Option<(&'de Sym, T)>>
    where
        T: Deserialize<'de>,
    {
        (**self).next_entry()
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn index(&self) -> usize {
        (*self).index()
    }
}

impl<'de, 'a, A> HashAccess<'de> for &'a mut A
where
    A: HashAccess<'de>,
{
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>,
    {
        (**self).next_key()
    }

    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>,
    {
        (**self).next_value()
    }

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        (**self).next_entry()
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn index(&self) -> usize {
        (*self).index()
    }
}

impl<'de, 'a, A> ArrayAccess<'de> for &'a mut A
where
    A: ArrayAccess<'de>,
{
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>,
    {
        (**self).next_element()
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn index(&self) -> usize {
        (*self).index()
    }
}
