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

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    #[source]
    pub kind: Kind,
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

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error {
            kind: Kind::Message(msg.to_string()),
        }
    }
}
