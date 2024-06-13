// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![allow(missing_docs)]

use std::str::Utf8Error;

use crate::{tag::Tag, Sym, Visitor};

/// Type alias around a result.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    #[source]
    pub kind: Kind,
}

// TODO: provide error context

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Kind {
    /// A length was negative when it should not have been.
    #[error("Unexpected negative length {0}")]
    UnexpectedNegativeLength(i32),
    /// Unrecognized tag was encountered.
    #[error("Wrong tag 0x{0:X} ({})", unknown_tag_to_char(*_0))]
    WrongTag(u8),
    /// A symbol was invalid utf8.
    /// All symbols in ruby should be valid.
    #[error("Symbol is invalid utf8 {0}")]
    SymbolInvalidUTF8(Utf8Error),
    /// A symbol link was not valid. (probably too large)
    #[error("Unresolved symlink {0}")]
    UnresolvedSymlink(usize),
    /// An object link was not valid. (probably too large)
    #[error("Unresolved Object link {0}")]
    UnresolvedObjectlink(usize),
    /// A float's mantissa was too long.
    #[error("Float mantissa too long")]
    ParseFloatMantissaTooLong,
    /// A symbol was expected (usually for a class name) and something else was found.
    #[error("Expected a symbol got {0:?}")]
    ExpectedSymbol(Tag),
    /// End of input.
    #[error("End of input")]
    Eof,
    /// Version mismatch.
    #[error("Version error, expected [4, 8], got {0:?}")]
    VersionError([u8; 2]),
    /// A custom error thrown by a visitor.
    #[error("{0}")]
    Message(String),

    #[error("Tried to deserialize a key without a value")]
    KeyAfterKey,
    #[error("Tried to deserialize a value before its key")]
    ValueAfterValue,
    #[error("A circular reference was detected while deserializing an object link")]
    CircularReference,
}

fn unknown_tag_to_char(tag: u8) -> char {
    if tag.is_ascii() && !(tag.is_ascii_control() || tag.is_ascii_whitespace()) {
        tag as char
    } else {
        '.'
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Unexpected<'a> {
    Nil,
    Bool(bool),
    Integer(i32),
    Float(f64),
    Hash,
    Array,
    String(&'a [u8]),
    Symbol(&'a Sym),
    Regex(&'a [u8]),
    Object(&'a Sym),
    Struct(&'a Sym),
    Class(&'a Sym),
    Module(&'a Sym),
    Instance,
    Extended(&'a Sym),
    UserClass(&'a Sym),
    UserData(&'a Sym),
    UserMarshal(&'a Sym),
    Data(&'a Sym),
}

pub trait Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl Expected for &str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl<'de, T> Expected for T
where
    T: Visitor<'de>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expecting(f)
    }
}

impl<'a> std::fmt::Display for dyn Expected + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Expected::fmt(self, f)
    }
}

impl<'a> std::fmt::Display for Unexpected<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unexpected::Nil => f.write_str("nil"),
            Unexpected::Bool(v) => write!(f, "bool `{v}`"),
            Unexpected::Integer(v) => write!(f, "integer `{v}`"),
            Unexpected::Float(v) => write!(f, "float `{v}`"),
            Unexpected::Hash => write!(f, "hash"),
            Unexpected::Array => write!(f, "array"),
            Unexpected::String(s) => {
                let s = String::from_utf8_lossy(s);
                write!(f, "string {s:?}")
            }
            Unexpected::Symbol(s) => write!(f, "symbol `{s}`"),
            Unexpected::Regex(r) => {
                let r = String::from_utf8_lossy(r);
                write!(f, "string {r:?}")
            }
            Unexpected::Object(c) => write!(f, "an instance of `{c}`"),
            Unexpected::Struct(n) => write!(f, "a struct of `{n}`"),
            Unexpected::Class(c) => write!(f, "class `{c}`"),
            Unexpected::Module(m) => write!(f, "module `{m}`"),
            Unexpected::Instance => write!(f, "object with ivars"),
            Unexpected::Extended(m) => write!(f, "extended object `{m}`"),
            Unexpected::UserClass(c) => write!(f, "user class object `{c}`"),
            Unexpected::UserData(c) => write!(f, "user data object `{c}`"),
            Unexpected::UserMarshal(c) => write!(f, "user marshal object `{c}`"),
            Unexpected::Data(c) => write!(f, "c data object `{c}`"),
        }
    }
}

impl Error {
    pub fn custom(str: impl std::fmt::Display) -> Self {
        Error {
            kind: Kind::Message(str.to_string()),
        }
    }

    pub fn invalid_type(unexpected: Unexpected<'_>, exp: &dyn Expected) -> Self {
        Self::custom(format!("invalid type: {unexpected}, expected `{exp}`"))
    }

    pub fn invalid_value(unexpected: Unexpected<'_>, exp: &dyn Expected) -> Self {
        Self::custom(format!("invalid value: {unexpected}, expected `{exp}`"))
    }

    pub fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        Self::custom(format!("invalid length: {len}, expected `{exp}`"))
    }

    pub fn unknown_field(field: &Sym, expected: &[&Sym]) -> Self {
        struct OneOf<'a> {
            expected: &'a [&'a Sym],
        }
        impl<'a> std::fmt::Display for OneOf<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.expected {
                    [] => write!(f, "there should be none"),
                    [exp] => write!(f, "expected `{exp}`"),
                    [exp1, exp2] => write!(f, "expected `{exp1}` or `{exp2}`"),
                    exp => {
                        for (i, exp) in exp.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "`{exp}`")?;
                        }
                        Ok(())
                    }
                }
            }
        }

        Self::custom(format!("unknown field {field}, {}", OneOf { expected }))
    }

    pub fn missing_field(field: &Sym) -> Self {
        Self::custom(format!("missing field `{field}`"))
    }

    pub fn duplicate_field(field: &Sym) -> Self {
        Self::custom(format!("duplicate field `{field}`"))
    }
}
