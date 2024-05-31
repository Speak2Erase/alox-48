// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::{
    DeError, Deserialize, DeserializerTrait, SerError, Serialize, SerializerTrait, Symbol,
};

mod de;
mod ser;

pub use de::Deserializer;
pub use ser::Serializer;

/// Like a stack trace, but for deserialization.
///
/// This is used to track the path to an error in a deserialization.
#[derive(Debug, Default, Clone)]
pub struct Trace {
    /// The context of the error.
    ///
    /// This will be in reverse order!
    /// The context furthest down the stack is the first element.
    pub context: Vec<Context>,
}

/// Part of the context of the error.
#[derive(Debug, Clone)]
// TODO deserializer position (no clue how to do this)
// FIXME this doesn't account for discarding errors!
pub enum Context {
    /// Error occurred while processing a `nil`.
    Nil,
    /// Error occurred while processing a boolean.
    Bool(bool),
    /// Error occurred while processing an integer.
    Int(i32),
    /// Error occurred while processing a float.
    Float(f64),

    /// Error occurred while processing a hash.
    ///
    /// The usize is the number of entries in the hash.
    Hash(usize),
    /// Error occurred while processing a key in a hash.
    ///
    /// The usize is the index of the key.
    HashKey(usize),
    /// Error occurred while processing a value in a hash.
    ///
    /// The usize is the index of the value.
    HashValue(usize),

    /// Error occurred while processing an array.
    ///
    /// The usize is the number of elements in the array.
    Array(usize),
    /// Error occurred while processing an element in an array.
    ///
    /// The usize is the index of the element.
    ArrayIndex(usize),

    /// Error occurred while processing a string.
    ///
    /// The string is converted from UTF-8 in a lossy manner!
    String(String),
    /// Error occurred while processing a symbol.
    Symbol(Symbol),
    /// Error occurred while processing a regex.
    ///
    /// The string is the regex pattern, and the u8 is the flags.
    Regex(String, u8),

    /// Error occurred while processing an object.
    ///
    /// The symbol is the class of the object, and the usize is the number of instance variables.
    Object(Symbol, usize),
    /// Error occurred while processing a struct.
    ///
    /// The symbol is the name of the struct, and the usize is the number of instance variables.
    Struct(Symbol, usize),

    /// Error occurred while fetching a field.
    ///
    /// The usize is the index of the field.
    FetchingField(usize),
    /// Error occurred while writing a field.
    ///
    /// The symbol is the name of the field, and the usize is the index of the field.
    WritingField(Symbol, usize),
    /// Error occurred while writing fields.
    ///
    /// The usize is the number of fields.
    WritingFields(usize),
    /// Error occurred while processing a field.
    ///
    /// The symbol is the name of the field, and the usize is the index of the field.
    /// The field name may not be present if the field is out of bounds.
    Field(Option<Symbol>, usize),

    /// Error occurred while processing a class.
    Class(Symbol),
    /// Error occurred while processing a module.
    Module(Symbol),

    /// Error occurred while processing an instance.
    Instance,

    /// Error occurred while processing an object extended by a module.
    ///
    /// The symbol is the module that extended the object.
    Extended(Symbol),
    /// Error occurred while processing a user class.
    ///
    /// The symbol is the class of the user class.
    UserClass(Symbol),
    /// Error occurred while processing user data.
    ///
    /// The symbol is the class of the user data.
    UserData(Symbol),
    /// Error occurred while processing user marshal.
    ///
    /// The symbol is the class of the user marshal.
    UserMarshal(Symbol),
    /// Error occurred while processing data.
    ///
    /// The symbol is the class of the data.
    Data(Symbol),
}

/// Deserialize a value from a given deserializer.
///
/// Automatically tracks the path to the error, and returns it as a `Trace`.
pub fn deserialize<'de, T>(input: impl DeserializerTrait<'de>) -> Result<T, (DeError, Trace)>
where
    T: Deserialize<'de>,
{
    let mut track = Trace::new();
    let deserializer = Deserializer::new(input, &mut track);

    let value = T::deserialize(deserializer);

    match value {
        Ok(value) => Ok(value),
        Err(err) => Err((err, track)),
    }
}

/// Serialize a value to a given serializer.
///
/// Automatically tracks the path to the error, and returns it as a `Trace`.
pub fn serialize<S>(input: impl Serialize, serializer: S) -> Result<S::Ok, (SerError, Trace)>
where
    S: SerializerTrait,
{
    let mut track = Trace::new();
    let serializer = Serializer::new(serializer, &mut track);

    let value = input.serialize(serializer);

    match value {
        Ok(value) => Ok(value),
        Err(err) => Err((err, track)),
    }
}

impl Trace {
    /// Create a new trace.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, context: Context) {
        self.context.push(context);
    }
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
            Array, ArrayIndex, Bool, Class, Data, Extended, FetchingField, Field, Float, Hash,
            HashKey, HashValue, Instance, Int, Module, Nil, Object, Regex, String, Struct, Symbol,
            UserClass, UserData, UserMarshal, WritingField, WritingFields,
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
            WritingField(field, index) => {
                write!(f, "while writing {field} (field index {index})")
            }
            WritingFields(len) => write!(f, "while writing {len} fields"),
            Field(None, index) => write!(f, "while processing an invalid field at index {index}"),
            Class(class) => write!(f, "while processing a class: {class}"),
            Module(module) => write!(f, "while processing a module: {module}"),
            Instance => write!(f, "while processing an instance"),
            Extended(module) => write!(f, "while processing an object extended by {module}"),
            UserClass(class) => write!(f, "while processing a user class: {class}"),
            UserData(class) => write!(f, "while processing user data: {class}"),
            UserMarshal(class) => write!(f, "while processing user marshal: {class}"),
            Data(class) => write!(f, "while processing data: {class}"),
        }
    }
}

macro_rules! add_context {
    ($erroring_expr:expr, $context:expr) => {
        match $erroring_expr {
            Ok(value) => Ok(value),
            Err(err) => {
                $context;
                Err(err)
            }
        }
    };
}
pub(crate) use add_context;
