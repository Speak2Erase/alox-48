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

use std::str::Utf8Error;

use crate::tag::Tag;

/// Type alias around a result.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    /// ENd of input.
    #[error("End of input.")]
    Eof,
    /// Version mismatch.
    #[error("Version error, expected [4, 8], got {0:?}")]
    VersionError([u8; 2]),
    /// A custom error thrown by serde.
    #[error("Serde error: {0}")]
    Message(String),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}
