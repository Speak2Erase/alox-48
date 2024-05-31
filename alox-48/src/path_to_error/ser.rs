use std::cell::Cell;

// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{add_context, Context, Trace};
use crate::{
    SerResult, Serialize, SerializeArray, SerializeHash, SerializeIvars, SerializerTrait, Sym,
    Symbol,
};

/// A serializer that tracks the path to an error.
///
/// It's actually relatively uncommon for errors to occur in serialization, but it's still useful to have this.
/// Usually an error in serialization is due to a bug in a `Serialize` impl, or a `Serialize` impl returning a custom error.
#[derive(Debug)]
pub struct Serializer<'trace, S> {
    serializer: S,
    trace: &'trace mut Trace,
}

#[derive(Debug)]
pub struct Wrapped<'trace, X> {
    inner: X,

    trace: &'trace mut Trace,

    len: usize,
    index: usize,
}

#[derive(Debug)]
pub struct WrappedIvars<'trace, X> {
    inner: X,

    trace: &'trace mut Trace,
    // because of the way serializers work, we can't actually add the calling context like with deserializers
    // so we have to store it here
    calling_context: Context,

    symbol: Option<Symbol>,
    len: usize,
    index: usize,
}

struct WrappedSerialize<'trace, X> {
    inner: X,
    trace: &'trace Cell<Trace>,
}

impl<'trace, S> Serializer<'trace, S>
where
    S: SerializerTrait,
{
    /// Create a new serializer.
    pub fn new(serializer: S, trace: &'trace mut Trace) -> Self {
        Self { serializer, trace }
    }
}

impl<'trace, S> SerializerTrait for Serializer<'trace, S>
where
    S: SerializerTrait,
{
    type Ok = S::Ok;
    type SerializeArray = Wrapped<'trace, S::SerializeArray>;
    type SerializeHash = Wrapped<'trace, S::SerializeHash>;
    type SerializeIvars = WrappedIvars<'trace, S::SerializeIvars>;

    fn serialize_nil(self) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_nil(),
            self.trace.push(Context::Nil)
        )
    }

    fn serialize_bool(self, v: bool) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_bool(v),
            self.trace.push(Context::Bool(v))
        )
    }

    fn serialize_i32(self, v: i32) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_i32(v),
            self.trace.push(Context::Int(v))
        )
    }

    fn serialize_f64(self, v: f64) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_f64(v),
            self.trace.push(Context::Float(v))
        )
    }

    fn serialize_hash(self, len: usize) -> SerResult<Self::SerializeHash> {
        add_context!(
            self.serializer.serialize_hash(len),
            self.trace.push(Context::Hash(len))
        )
        .map(|inner| Wrapped {
            inner,
            trace: self.trace,
            len,
            index: 0,
        })
    }

    fn serialize_array(self, len: usize) -> SerResult<Self::SerializeArray> {
        add_context!(
            self.serializer.serialize_array(len),
            self.trace.push(Context::Array(len))
        )
        .map(|inner| Wrapped {
            inner,
            trace: self.trace,
            len,
            index: 0,
        })
    }

    fn serialize_string(self, data: &[u8]) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_string(data),
            self.trace
                .push(Context::String(String::from_utf8_lossy(data).to_string()))
        )
    }

    fn serialize_symbol(self, sym: &crate::Sym) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_symbol(sym),
            self.trace.push(Context::Symbol(sym.to_symbol()))
        )
    }

    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_regular_expression(regex, flags),
            self.trace.push(Context::Regex(
                String::from_utf8_lossy(regex).to_string(),
                flags
            ))
        )
    }

    fn serialize_object(self, class: &crate::Sym, len: usize) -> SerResult<Self::SerializeIvars> {
        add_context!(
            self.serializer.serialize_object(class, len),
            self.trace.push(Context::Object(class.to_symbol(), len))
        )
        .map(|inner| WrappedIvars {
            inner,
            trace: self.trace,
            calling_context: Context::Object(class.to_symbol(), len),
            symbol: None,
            len,
            index: 0,
        })
    }

    fn serialize_struct(self, name: &crate::Sym, len: usize) -> SerResult<Self::SerializeIvars> {
        add_context!(
            self.serializer.serialize_struct(name, len),
            self.trace.push(Context::Struct(name.to_symbol(), len))
        )
        .map(|inner| WrappedIvars {
            inner,
            trace: self.trace,
            calling_context: Context::Struct(name.to_symbol(), len),
            symbol: None,
            len,
            index: 0,
        })
    }

    fn serialize_class(self, class: &crate::Sym) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_class(class),
            self.trace.push(Context::Class(class.to_symbol()))
        )
    }

    fn serialize_module(self, module: &crate::Sym) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_module(module),
            self.trace.push(Context::Module(module.to_symbol()))
        )
    }

    fn serialize_instance<V>(self, value: &V, len: usize) -> SerResult<Self::SerializeIvars>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: value,
            trace: &trace,
        };

        add_context!(self.serializer.serialize_instance(&wrapped, len), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::Instance);
        })
        .map(|inner| WrappedIvars {
            inner,
            trace: self.trace,
            calling_context: Context::Instance,
            symbol: None,
            len,
            index: 0,
        })
    }

    fn serialize_extended<V>(self, module: &crate::Sym, value: &V) -> SerResult<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: value,
            trace: &trace,
        };

        add_context!(self.serializer.serialize_extended(module, &wrapped), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::Extended(module.to_symbol()));
        })
    }

    fn serialize_user_class<V>(self, class: &crate::Sym, value: &V) -> SerResult<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: value,
            trace: &trace,
        };

        add_context!(self.serializer.serialize_user_class(class, &wrapped), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::UserClass(class.to_symbol()));
        })
    }

    fn serialize_user_data(self, class: &crate::Sym, data: &[u8]) -> SerResult<Self::Ok> {
        add_context!(
            self.serializer.serialize_user_data(class, data),
            self.trace.push(Context::UserData(class.to_symbol()))
        )
    }

    fn serialize_user_marshal<V>(self, class: &crate::Sym, value: &V) -> SerResult<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: value,
            trace: &trace,
        };

        add_context!(self.serializer.serialize_user_marshal(class, &wrapped), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::UserMarshal(class.to_symbol()));
        })
    }

    fn serialize_data<V>(self, class: &crate::Sym, value: &V) -> SerResult<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: value,
            trace: &trace,
        };

        add_context!(self.serializer.serialize_data(class, &wrapped), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::Data(class.to_symbol()));
        })
    }
}

impl<'trace, X> SerializeArray for Wrapped<'trace, X>
where
    X: SerializeArray,
{
    type Ok = X::Ok;

    fn serialize_element<T>(&mut self, v: &T) -> SerResult<()>
    where
        T: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: v,
            trace: &trace,
        };

        self.index += 1;
        add_context!(self.inner.serialize_element(&wrapped), {
            self.trace.push(Context::Array(self.len));
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::ArrayIndex(self.index - 1));
        })
    }

    fn end(self) -> SerResult<Self::Ok> {
        add_context!(self.inner.end(), self.trace.push(Context::Array(self.len)))
    }
}

impl<'trace, X> SerializeHash for Wrapped<'trace, X>
where
    X: SerializeHash,
{
    type Ok = X::Ok;

    fn serialize_key<T>(&mut self, v: &T) -> SerResult<()>
    where
        T: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: v,
            trace: &trace,
        };

        add_context!(self.inner.serialize_key(&wrapped), {
            self.trace.push(Context::Hash(self.len));
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::HashKey(self.index));
        })
    }

    fn serialize_value<T>(&mut self, v: &T) -> SerResult<()>
    where
        T: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: v,
            trace: &trace,
        };

        self.index += 1;
        add_context!(self.inner.serialize_value(&wrapped), {
            self.trace.push(Context::Hash(self.len));
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            self.trace.push(Context::HashValue(self.index - 1));
        })
    }

    fn end(self) -> SerResult<Self::Ok> {
        add_context!(self.inner.end(), self.trace.push(Context::Hash(self.len)))
    }
}

impl<'trace, X> SerializeIvars for WrappedIvars<'trace, X>
where
    X: SerializeIvars,
{
    type Ok = X::Ok;

    fn serialize_field(&mut self, k: &Sym) -> SerResult<()> {
        self.symbol = Some(k.to_symbol());
        add_context!(self.inner.serialize_field(k), {
            self.trace.push(self.calling_context.clone());
            self.trace
                .push(Context::WritingField(k.to_symbol(), self.index));
        })
    }

    fn serialize_value<V>(&mut self, v: &V) -> SerResult<()>
    where
        V: Serialize + ?Sized,
    {
        let trace = Cell::default();
        let wrapped = WrappedSerialize {
            inner: v,
            trace: &trace,
        };

        self.index += 1;
        add_context!(self.inner.serialize_value(&wrapped), {
            let trace = trace.into_inner();
            self.trace.context.extend(trace.context);
            {
                self.trace.push(self.calling_context.clone());
                self.trace
                    .push(Context::Field(self.symbol.clone(), self.index - 1));
            };
        })
    }

    fn end(self) -> SerResult<Self::Ok> {
        add_context!(self.inner.end(), {
            self.trace.push(self.calling_context.clone());
            self.trace.push(Context::WritingFields(self.len));
        })
    }
}

impl<'trace, X> Serialize for WrappedSerialize<'trace, X>
where
    X: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
    where
        S: SerializerTrait,
    {
        let mut trace = Trace::new();
        let result = self
            .inner
            .serialize(Serializer::new(serializer, &mut trace));
        // This might seem weird. How are we using a Cell with a non-copy type?
        // Well, Cell::get *requires* copy, but not Cell::set.
        // So we can use Cell::set to move the trace into the cell, and then get it back out with into_inner.
        self.trace.set(trace);

        result
    }
}
