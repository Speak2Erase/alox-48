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

/// Type alias around a result.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("Serialization error")]
pub struct Error {
    #[source]
    pub kind: Kind,
    #[related]
    pub context: Vec<Context>,
}

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Kind {
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
    #[error("Serde error: {0}")]
    Message(String),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Context {
    #[error("While seserializing a struct {0} with {1} fields")]
    Struct(String, usize),
    #[error("While seserializing an object {0} with {1} fields")]
    Object(String, usize),
    #[error("While seserializing a userdata {0} of len {1}")]
    Userdata(String, usize),
    #[error("While seserializing an array of len {0}")]
    Array(usize),
    #[error("While serializing a hash of len {0}")]
    Hash(usize),
    #[error("While serializing a key from a key value pair")]
    Key,
    #[error("While serializing a value from a key value pair")]
    Value,

    // Terminals (these happen at the end of a backtrace)
    #[error("While serializing a symbol")]
    Symbol,
    #[error("While serializing an already present symbol at {0}")]
    Symlink(usize),
    #[error("While serializing an already present object at {0}")]
    Objectlink(usize),
    #[error("While serializing string text")]
    StringText,
    #[error("While serializing string fields")]
    StringFields,
    #[error("While serializing an integer")]
    Integer,
    #[error("While serializing a float")]
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


impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error {
            kind: Kind::Message(msg.to_string()),
            context: vec![],
        }
    }
}
