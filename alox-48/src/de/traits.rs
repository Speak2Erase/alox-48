// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{error::Unexpected, Error, Result};
use crate::Sym;
use std::marker::PhantomData;

/// A structure that can be deserialized from ruby marshal format.
pub trait Deserialize<'de>: Sized {
    /// Deserialize this value from the given deserializer.
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: Deserializer<'de>;
}

/// A stateful form of `Deserialize`- useful when you need to pass data into a Deserialize impl.
pub trait DeserializeSeed<'de>: Sized {
    /// The value that will be produced.
    type Value;

    /// Deserialize this value from the given deserializer.
    ///
    /// Equivalent to `Deserialize::deserialize`, but with data passed in.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>;
}

/// A structure that can deserialize data from ruby marshal format.
pub trait Deserializer<'de>: Sized {
    /// Deserialize a value from the given visitor.
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>;

    /// Deserialize an optional value from the given visitor.
    ///
    /// This is used for deserializing `Option<T>`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorOption<'de>;

    /// Deserialize an instance from the given visitor.
    ///
    /// This is used for deserializing `Instance<T>`.
    fn deserialize_instance<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorInstance<'de>;
}

/// This trait represents a visitor that walks through a deserializer.
pub trait Visitor<'de>: Sized {
    /// The type that this visitor will produce.
    type Value;

    /// Format a message stating what the visitor is expecting to receive.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    /// Input contains a `nil` value.
    // Primitives
    fn visit_nil(self) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Nil, &self))
    }
    /// Input contains a boolean value.
    fn visit_bool(self, v: bool) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Bool(v), &self))
    }
    /// Input contains an integer value.
    fn visit_i32(self, v: i32) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Integer(v), &self))
    }
    /// Input contains a float value.
    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Float(v), &self))
    }

    /// Input contains a hash.
    // Collections
    fn visit_hash<A>(self, _map: A) -> Result<Self::Value>
    where
        A: HashAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Hash, &self))
    }
    /// Input contains an array.
    fn visit_array<A>(self, _array: A) -> Result<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Array, &self))
    }
    /// Input contains a string.
    ///
    /// Ruby strings are not guarenteed to be UTF-8, so this is a `&[u8]` instead of a `&str`.
    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::String(string), &self))
    }
    /// Input contains a symbol.
    ///
    /// Symbols are an interned string in ruby, so they are used frequently in variable names and such.
    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Symbol(symbol), &self))
    }
    /// Input contains a regular expression.
    ///
    /// The flags associated with the regex (global matching, case insensitivity, etc) are also included in a bitfield.
    fn visit_regular_expression(self, regex: &'de [u8], _flags: u8) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Regex(regex), &self))
    }

    // Class instances types
    /// Input contains an object.
    fn visit_object<A>(self, class: &'de Sym, _instance_variables: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Object(class), &self))
    }
    /// Input contains a struct.
    ///
    /// Structs are similar to objects, but with predefined accessors.
    fn visit_struct<A>(self, name: &'de Sym, _members: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        Err(Error::invalid_value(Unexpected::Struct(name), &self))
    }
    // Other
    /// Input contains a class.
    ///
    /// There's not much to do with this, as it's just the name of the class.
    fn visit_class(self, class: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Class(class), &self))
    }
    /// Input contains a module.
    ///
    /// There's not much to do with this, as it's just the name of the module.
    fn visit_module(self, module: &'de Sym) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::Module(module), &self))
    }

    // Extended/modified types
    /// Input contains an instance with extra instance variables.
    ///
    /// This is an object like `String`, `Hash`, etc that has been extended with extra instance variables.
    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        struct SeededVisitor<T>(pub T);
        impl<'de, T> DeserializeSeed<'de> for SeededVisitor<T>
        where
            T: Visitor<'de>,
        {
            type Value = T::Value;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize(self.0)
            }
        }

        let (value, _) = instance.value_seed(SeededVisitor(self))?;
        Ok(value)
    }
    /// Input contains an extended object.
    ///
    /// This is an object that has been extended with a module.
    fn visit_extended<D>(self, _module: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }

    // User types
    /// Input contains an object that is subclassed from a special class (`String`, `Array`, etc).
    fn visit_user_class<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
    /// Input contains user data.
    ///
    /// User data is a blob of data that is not interpreted by the marshal format, and is instead handed off to type-specific deserializers.
    fn visit_user_data(self, class: &'de Sym, _data: &'de [u8]) -> Result<Self::Value> {
        Err(Error::invalid_value(Unexpected::UserData(class), &self))
    }
    /// Input contains an object that has been deserialized as another type.
    fn visit_user_marshal<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
    /// Input contains C extension data.
    ///
    /// It's unclear what this actually is, the ruby docs are not very clear.
    fn visit_data<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize(self)
    }
}

/// This trait represents a visitor that walks through a deserializer.
///
/// It's specialized for deserializing optional values.
// todo investigate other ways of doing this
pub trait VisitorOption<'de> {
    /// The type that this visitor will produce.
    type Value;

    /// Input contains a `nil` value.
    fn visit_none(self) -> Result<Self::Value>;

    /// Input contains a value.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>;
}

/// This trait represents a visitor that walks through a deserializer.
///
/// It's specialized for deserializing instances (ruby objects with extra instance variables).
pub trait VisitorInstance<'de> {
    /// The type that this visitor will produce.
    type Value;

    /// The input does not contain any instance variables.
    fn visit<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: Deserializer<'de>;

    /// The input contains instance variables.
    fn visit_instance<A>(self, access: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>;
}

/// Provides a visitor with access to an instance.
pub trait InstanceAccess<'de>: Sized {
    /// The instance variable accessor for this instance.
    type IvarAccess: IvarAccess<'de>;

    /// Deserialize the value of the instance, using the given seed.
    ///
    /// This allows you to pass data in when deserializing the value.
    fn value_seed<V>(self, seed: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: DeserializeSeed<'de>;

    /// Deserialize the value of the instance.
    ///
    /// This is a convenience method to deserialize a value without its visitor.
    fn value<T>(self) -> Result<(T, Self::IvarAccess)>
    where
        T: Deserialize<'de>,
    {
        self.value_seed(PhantomData::<T>)
    }
}

/// Provides access to instance variables.
pub trait IvarAccess<'de> {
    /// Get the next instance variable.
    ///
    /// Returns `None` if there are no more instance variables.
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>>;

    /// Get the next value.
    ///
    /// This should be called after `next_ivar`.
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>;

    /// Get the next value.
    ///
    /// This should be called after `next_ivar`.
    fn next_value<T>(&mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        self.next_value_seed(PhantomData::<T>)
    }

    /// Get the next instance variable and value.
    ///
    /// Returns `None` if there are no more instance variables.
    fn next_entry_seed<V>(&mut self, seed: V) -> Result<Option<(&'de Sym, V::Value)>>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(var) = self.next_ivar()? {
            self.next_value_seed(seed).map(|v| Some((var, v)))
        } else {
            Ok(None)
        }
    }

    /// Get the next instance variable and value.
    ///
    /// Returns `None` if there are no more instance variables.
    fn next_entry<T>(&mut self) -> Result<Option<(&'de Sym, T)>>
    where
        T: Deserialize<'de>,
    {
        self.next_entry_seed(PhantomData::<T>)
    }

    /// Get the number of instance variables.
    fn len(&self) -> usize;

    /// Get the index of the current instance variable.
    fn index(&self) -> usize;

    /// Returns `true` if there are no instance variables.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Provides access to hash elements.
pub trait HashAccess<'de> {
    /// Get the next key.
    ///
    /// Returns `None` if there are no more keys.
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>;

    /// Get the next value.
    ///
    /// This should be called after `next_key`.
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>;

    /// Get the next key.
    ///
    /// Returns `None` if there are no more keys.
    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>,
    {
        self.next_key_seed(PhantomData::<K>)
    }

    /// Get the next value.
    ///
    /// This should be called after `next_key`.
    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>,
    {
        self.next_value_seed(PhantomData::<V>)
    }

    /// Get the next key and value.
    fn next_entry_seed<K, V>(
        &mut self,
        key_seed: K,
        value_seed: V,
    ) -> Result<Option<(K::Value, V::Value)>>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        if let Some(k) = self.next_key_seed(key_seed)? {
            self.next_value_seed(value_seed).map(|v| Some((k, v)))
        } else {
            Ok(None)
        }
    }

    /// Get the next key and value.
    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        self.next_entry_seed(PhantomData::<K>, PhantomData::<V>)
    }

    /// Get the number of elements.
    fn len(&self) -> usize;

    /// Get the index of the current element.
    fn index(&self) -> usize;

    /// Returns `true` if there are no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Provides access to array elements.
pub trait ArrayAccess<'de> {
    /// Get the next element.
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>;

    /// Get the next element.
    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>,
    {
        self.next_element_seed(PhantomData::<T>)
    }

    /// Get the number of elements.
    fn len(&self) -> usize;

    /// Get the index of the current element.
    fn index(&self) -> usize;

    /// Returns `true` if there are no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'de, 'a, A> IvarAccess<'de> for &'a mut A
where
    A: IvarAccess<'de>,
{
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>> {
        (**self).next_ivar()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        (**self).next_value_seed(seed)
    }

    fn next_entry<T>(&mut self) -> Result<Option<(&'de Sym, T)>>
    where
        T: Deserialize<'de>,
    {
        (**self).next_entry()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn index(&self) -> usize {
        (**self).index()
    }
}

impl<'de, 'a, A> HashAccess<'de> for &'a mut A
where
    A: HashAccess<'de>,
{
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        (**self).next_key_seed(seed)
    }

    fn next_key<K>(&mut self) -> Result<Option<K>>
    where
        K: Deserialize<'de>,
    {
        (**self).next_key()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        (**self).next_value_seed(seed)
    }

    fn next_value<V>(&mut self) -> Result<V>
    where
        V: Deserialize<'de>,
    {
        (**self).next_value()
    }

    fn next_entry_seed<K, V>(
        &mut self,
        key_seed: K,
        value_seed: V,
    ) -> Result<Option<(K::Value, V::Value)>>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        (**self).next_entry_seed(key_seed, value_seed)
    }

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        (**self).next_entry()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn index(&self) -> usize {
        (**self).index()
    }
}

impl<'de, 'a, A> ArrayAccess<'de> for &'a mut A
where
    A: ArrayAccess<'de>,
{
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        (**self).next_element_seed(seed)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>>
    where
        T: Deserialize<'de>,
    {
        (**self).next_element()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn index(&self) -> usize {
        (**self).index()
    }
}
