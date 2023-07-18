// Copyright (C) 2022 Lily Lyons
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
#![allow(missing_docs)]

use std::{str::Utf8Error, vec};

use crate::tag::Tag;

/// Type alias around a result.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("Deserialization error")]
pub struct Error {
    #[source]
    pub kind: Kind,
    #[label("this bit here")]
    pub span: miette::SourceSpan,
    #[related]
    pub context: Vec<Context>,
}

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Kind {
    /// A length was negative when it should not have been.
    #[error("Unexpected negative length {0}")]
    UnexpectedNegativeLength(i32),
    /// Unrecognized tag was encountered.
    #[error("Wrong tag {0}")]
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
    /// Unsupported data was encountered.
    ///
    /// alox-48 currently defines these data types as unsupported:
    /// - HashDefault => A hash with a default value
    /// - UserClass => An object inheriting from a default ruby class.
    /// - RawRegexp => A regex in ruby.
    /// - ClassRef => A class in ruby. No methods though.
    /// - ModuleRef => A module in ruby.
    /// - Extended => An object that was extended by a module at runtime.
    /// - UserMarshal => An object that when deserialized deserializes to another type.
    /// - Struct => A ruby "Struct".
    ///
    /// This is a UserClass:
    /// ```rb
    /// class CustomArray < Array
    /// end
    /// Marshal.dump(CustomArray.new)
    /// ```
    #[error("Unsupported data encountered: {0}. This is probably because it does not map well to Rust's type system")]
    Unsupported(&'static str),
    /// End of input.
    #[error("End of input.")]
    Eof,
    /// Version mismatch.
    #[error("Version error, expected [4, 8], got {0:?}")]
    VersionError([u8; 2]),
    /// A custom error thrown by serde.
    #[error("Serde error: {0}")]
    Message(String),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Context {
    #[error("While deserializing a struct {0} with {1} fields")]
    Struct(String, usize),
    #[error("While deserializing an object {0} with {1} fields")]
    Object(String, usize),
    #[error("While deserializing a userdata {0} of len {1}")]
    Userdata(String, usize),
    #[error("While deserializing an array of len {0}")]
    Array(usize),
    #[error("While deserializing a hash of len {0}")]
    Hash(usize),
    #[error("While deserializing a key from a key value pair")]
    Key,
    #[error("While deserializing a value from a key value pair")]
    Value,
    #[error("While deserializing an object instance")]
    Instance,

    // Terminals (these happen at the end of a backtrace)
    #[error("While parsing the marshal version")]
    ParsingVersion,
    #[error("While deserializing an object class name")]
    ClassName,
    #[error("While deserializing an object len")]
    ObjectLen,
    #[error("While deserializing a symbol len")]
    SymbolLen,
    #[error("While deserializing a userdata len")]
    UserdataLen,
    #[error("While deserializing an array length")]
    ArrayLen,
    #[error("While deserializing a hash length")]
    HashLen,
    #[error("While parsing the next tag")]
    FindingTag,
    #[error("While deserializing a symbol")]
    Symbol,
    #[error("While deserializing an already present symbol at {0}")]
    Symlink(usize),
    #[error("While deserializing an already present object at {0}")]
    Objectlink(usize),
    #[error("While deserializing string text")]
    StringText,
    #[error("While deserializing string fields")]
    StringFields,
    #[error("While deserializing an integer")]
    Integer,
    #[error("While deserializing a float")]
    Float,
}

#[allow(unused_macros)]
macro_rules! bubble_error {
    ($bubble:expr, $($context:expr),+ $(,)?) => {
        match $bubble {
            Ok(o) => o,
            Err(mut e) => {
                $(e.context.push($context);)+
                return Err(e);
            }
        }
    };
}
pub(crate) use bubble_error;

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error {
            kind: Kind::Message(msg.to_string()),
            context: vec![],
            span: (0..0).into(),
        }
    }
}
