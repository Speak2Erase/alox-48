// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    de::{DeserializeSeed, DeserializerTrait},
    ArrayAccess, DeError, DeResult, HashAccess, InstanceAccess, IvarAccess, Sym, Symbol, Visitor,
    VisitorInstance, VisitorOption,
};

/// A deserializer that tracks where errors occur.
#[derive(Debug)]
pub struct Deserializer<'trace, T> {
    deserializer: T,
    trace: &'trace mut Trace,
}

/// Like a stack trace, but for deserialization.
///
/// This is used to track the path to an error in a deserialization.
#[derive(Debug, Default)]
pub struct Trace {
    /// The context of the error.
    ///
    /// This will be in reverse order!
    /// The context furthest down the stack is the first element.
    pub context: Vec<Context>,
}

#[derive(Debug)]
struct Wrapped<'trace, X> {
    inner: X,
    trace: &'trace mut Trace,
}

#[derive(Debug)]
// TODO deserializer position (no clue how to do this)
// FIXME this doesn't account for discarding errors!
pub enum Context {
    Nil,
    Bool(bool),
    Int(i32),
    Float(f64),

    Hash(usize),
    HashKey(usize),
    HashValue(usize),

    Array(usize),
    ArrayIndex(usize),

    String(String),
    Symbol(Symbol),
    Regex(String, u8),

    Object(Symbol, usize),
    Struct(Symbol, usize),

    FetchingField(usize),
    Field(Option<Symbol>, usize),

    Class(Symbol),
    Module(Symbol),

    Instance,

    Extended(Symbol),
    UserClass(Symbol),
    UserData(Symbol),
    UserMarshal(Symbol),
    ProcessingData(Symbol),
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for context in self.context.iter().rev() {
            writeln!(f, "{context}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Context::{
            Array, ArrayIndex, Bool, Class, Extended, FetchingField, Field, Float, Hash, HashKey,
            HashValue, Instance, Int, Module, Nil, Object, ProcessingData, Regex, String, Struct,
            Symbol, UserClass, UserData, UserMarshal,
        };
        match self {
            Nil => write!(f, "while processing a nil"),
            Bool(v) => write!(f, "while processing a boolean: {v}"),
            Int(v) => write!(f, "while processing an integer: {v}"),
            Float(v) => write!(f, "while processing a float: {v}"),
            Hash(len) => write!(f, "while processing a hash with {len} entries",),
            HashKey(index) => write!(f, "while processing the {index} key of a hash",),
            HashValue(index) => write!(f, "while processing the {index} value of a hash"),
            Array(len) => write!(f, "while processing an array with {len} elements",),
            ArrayIndex(index) => write!(f, "while processing the {index} element of an array"),
            String(s) => write!(f, "while processing a string: {s}"),
            Symbol(s) => write!(f, "while processing a symbol: {s}"),
            Regex(s, flags) => write!(f, "while processing a regex: /{s}/ {flags}"),
            Object(class, len) => write!(
                f,
                "while processing an instance of {class} with {len} ivars"
            ),
            Struct(name, len) => write!(f, "while processing a struct of {name} with {len} ivars"),
            FetchingField(index) => write!(f, "while fetching the {index} field"),
            Field(Some(field), index) => {
                write!(f, "while processing {field} (field index {index})")
            }
            Field(None, index) => write!(f, "while processing an invalid field at index {index}"),
            Class(class) => write!(f, "while processing a class: {class}"),
            Module(module) => write!(f, "while processing a module: {module}"),
            Instance => write!(f, "while processing an instance"),
            Extended(module) => write!(f, "while processing an object extended by {module}"),
            UserClass(class) => write!(f, "while processing a user class: {class}"),
            UserData(class) => write!(f, "while processing user data: {class}"),
            UserMarshal(class) => write!(f, "while processing user marshal: {class}"),
            ProcessingData(class) => write!(f, "while processing data: {class}"),
        }
    }
}

macro_rules! add_context {
    ($erroring_expr:expr $(, $context:expr )*) => {
        match $erroring_expr {
            Ok(value) => Ok(value),
            Err(err) => {
                $( $context; )*
                Err(err)
            }
        }
    };
}

impl<'de, 'trace, T> Deserializer<'trace, T>
where
    T: DeserializerTrait<'de>,
{
    /// Create a new deserializer.
    pub fn new(deserializer: T, track: &'trace mut Trace) -> Self {
        Self {
            deserializer,
            trace: track,
        }
    }
}

impl Trace {
    /// Create a new trace.
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, context: Context) {
        self.context.push(context);
    }
}

impl<'de, 'trace, T> DeserializerTrait<'de> for Deserializer<'trace, T>
where
    T: DeserializerTrait<'de>,
{
    fn deserialize<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserializer.deserialize(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: VisitorOption<'de>,
    {
        self.deserializer.deserialize_option(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }

    fn deserialize_instance<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: VisitorInstance<'de>,
    {
        self.deserializer.deserialize_instance(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }
}

impl<'de, 'trace, X> Visitor<'de> for Wrapped<'trace, X>
where
    X: Visitor<'de>,
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.expecting(formatter)
    }

    fn visit_nil(self) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_nil(), self.trace.push(Context::Nil))
    }

    fn visit_bool(self, v: bool) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_bool(v), self.trace.push(Context::Bool(v)))
    }

    fn visit_i32(self, v: i32) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_i32(v), self.trace.push(Context::Int(v)))
    }

    fn visit_f64(self, v: f64) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_f64(v), self.trace.push(Context::Float(v)))
    }

    fn visit_hash<A>(self, map: A) -> DeResult<Self::Value>
    where
        A: HashAccess<'de>,
    {
        let wrapped = Wrapped {
            inner: map,
            trace: self.trace,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_hash(wrapped),
            self.trace.push(Context::Hash(len))
        )
    }

    fn visit_array<A>(self, array: A) -> DeResult<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        let wrapped = Wrapped {
            inner: array,
            trace: self.trace,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_array(wrapped),
            self.trace.push(Context::Array(len))
        )
    }

    fn visit_string(self, string: &'de [u8]) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_string(string),
            self.trace.push(Context::String(
                String::from_utf8_lossy(string).into_owned()
            ))
        )
    }

    fn visit_symbol(self, symbol: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_symbol(symbol),
            self.trace.push(Context::Symbol(symbol.to_symbol()))
        )
    }

    fn visit_regular_expression(self, regex: &'de [u8], flags: u8) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_regular_expression(regex, flags),
            self.trace.push(Context::Regex(
                String::from_utf8_lossy(regex).into_owned(),
                flags
            ))
        )
    }

    fn visit_object<A>(self, class: &'de Sym, instance_variables: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let wrapped = WrappedIvarAccess {
            inner: instance_variables,
            trace: self.trace,
            current_field: None,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_object(class, wrapped),
            self.trace.push(Context::Object(class.to_symbol(), len))
        )
    }

    fn visit_struct<A>(self, name: &'de Sym, members: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let wrapped = WrappedIvarAccess {
            inner: members,
            trace: self.trace,
            current_field: None,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_struct(name, wrapped),
            self.trace.push(Context::Struct(name.to_symbol(), len))
        )
    }

    fn visit_class(self, class: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_class(class),
            self.trace.push(Context::Class(class.to_symbol()))
        )
    }

    fn visit_module(self, module: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_module(module),
            self.trace.push(Context::Module(module.to_symbol()))
        )
    }

    fn visit_instance<A>(self, instance: A) -> DeResult<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        let wrapped = Wrapped {
            inner: instance,
            trace: self.trace,
        };
        add_context!(
            self.inner.visit_instance(wrapped),
            self.trace.push(Context::Instance)
        )
    }

    fn visit_extended<D>(self, module: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_extended(module, wrapped),
            self.trace.push(Context::Extended(module.to_symbol()))
        )
    }

    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_user_class(class, wrapped),
            self.trace.push(Context::UserClass(class.to_symbol()))
        )
    }

    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_user_data(class, data),
            self.trace.push(Context::UserData(class.to_symbol()))
        )
    }

    fn visit_user_marshal<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_user_marshal(class, wrapped),
            self.trace.push(Context::UserMarshal(class.to_symbol()))
        )
    }

    fn visit_data<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_data(class, wrapped),
            self.trace.push(Context::ProcessingData(class.to_symbol()))
        )
    }
}

impl<'de, 'trace, X> VisitorOption<'de> for Wrapped<'trace, X>
where
    X: VisitorOption<'de>,
{
    type Value = X::Value;

    fn visit_none(self) -> DeResult<Self::Value> {
        self.inner.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .visit_some(Deserializer::new(deserializer, self.trace))
    }
}

impl<'de, 'trace, X> VisitorInstance<'de> for Wrapped<'trace, X>
where
    X: VisitorInstance<'de>,
{
    type Value = X::Value;

    fn visit<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .visit(Deserializer::new(deserializer, self.trace))
    }

    fn visit_instance<A>(self, access: A) -> DeResult<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        self.inner.visit_instance(Wrapped {
            inner: access,
            trace: self.trace,
        })
    }
}

impl<'de, 'trace, X> InstanceAccess<'de> for Wrapped<'trace, X>
where
    X: InstanceAccess<'de>,
{
    type IvarAccess = WrappedIvarAccess<'trace, X::IvarAccess>;

    fn value<V>(self, visitor: V) -> DeResult<(V::Value, Self::IvarAccess)>
    where
        V: Visitor<'de>,
    {
        let wrapped_visitor = Wrapped {
            inner: visitor,
            trace: &mut *self.trace,
        };
        let (value, access) = self.inner.value(wrapped_visitor)?;
        let wrapped_access = WrappedIvarAccess {
            inner: access,
            trace: self.trace,
            current_field: None,
        };
        Ok((value, wrapped_access))
    }

    fn value_deserialize_seed<V>(self, seed: V) -> DeResult<(V::Value, Self::IvarAccess)>
    where
        V: DeserializeSeed<'de>,
    {
        let wrapped_seed = Wrapped {
            inner: seed,
            trace: &mut *self.trace,
        };
        let (value, access) = self.inner.value_deserialize_seed(wrapped_seed)?;
        let wrapped_access = WrappedIvarAccess {
            inner: access,
            trace: self.trace,
            current_field: None,
        };
        Ok((value, wrapped_access))
    }
}

struct WrappedIvarAccess<'trace, X> {
    inner: X,
    trace: &'trace mut Trace,
    current_field: Option<Symbol>,
}

impl<'de, 'trace, X> IvarAccess<'de> for WrappedIvarAccess<'trace, X>
where
    X: IvarAccess<'de>,
{
    fn next_ivar(&mut self) -> DeResult<Option<&'de Sym>> {
        let symbol = add_context!(
            self.inner.next_ivar(),
            self.trace.push(Context::FetchingField(self.index()))
        )?;
        self.current_field = symbol.map(Sym::to_symbol);
        Ok(symbol)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeResult<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let wrapped_seed = Wrapped {
            inner: seed,
            trace: self.trace,
        };
        add_context!(
            self.inner.next_value_seed(wrapped_seed),
            self.trace
                .push(Context::Field(self.current_field.clone(), self.index()))
        )
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

impl<'de, 'trace, X> HashAccess<'de> for Wrapped<'trace, X>
where
    X: HashAccess<'de>,
{
    fn next_key_seed<K>(&mut self, seed: K) -> DeResult<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        add_context!(
            self.inner.next_key_seed(Wrapped {
                inner: seed,
                trace: self.trace,
            }),
            self.trace.push(Context::HashKey(self.index()))
        )
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeResult<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        add_context!(
            self.inner.next_value_seed(Wrapped {
                inner: seed,
                trace: self.trace,
            }),
            self.trace.push(Context::HashValue(self.index()))
        )
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

impl<'de, 'trace, X> ArrayAccess<'de> for Wrapped<'trace, X>
where
    X: ArrayAccess<'de>,
{
    fn next_element_seed<T>(&mut self, seed: T) -> DeResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        add_context!(
            self.inner.next_element_seed(Wrapped {
                inner: seed,
                trace: self.trace,
            }),
            self.trace.push(Context::ArrayIndex(self.index()))
        )
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

impl<'de, 'trace, X> DeserializeSeed<'de> for Wrapped<'trace, X>
where
    X: DeserializeSeed<'de>,
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .deserialize(Deserializer::new(deserializer, self.trace))
    }
}
