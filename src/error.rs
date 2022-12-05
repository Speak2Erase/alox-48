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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),

    /// There is a version mismatch in your marshal data.
    /// (It's expected that 4.8 is the version number)
    /// This usually indicates invalid marshal data.
    VersionError([u8; 2]),

    TypeError(String),
    UnsupportedType(u8),
    BignumSignError(u8),
    WrongClass(&'static str, String),
    WrongInstanceVarCount(usize),

    Utf8Error(std::str::Utf8Error),
    ParseFloatError(std::num::ParseFloatError),
    StringExtraIvars,
    NonUtf8String(String),

    Eof,
    /// There are several features not supported by this crate that ruby marshal does support.
    /// These include:
    ///     Object references
    ///     Object module extensions
    ///     Classes
    ///     Data objects
    ///     Symbols
    Unsupported,
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            Message(msg) => f.write_str(msg),
            VersionError(ver) => f.write_fmt(format_args!(
                "version error (expected 4.8<=, got {}.{})",
                ver[0], ver[1]
            )),
            TypeError(msg) => f.write_str(msg),
            UnsupportedType(typ) => f.write_fmt(format_args!(
                "unsupported type indentifier {}",
                *typ as char
            )),
            BignumSignError(sign) => {
                f.write_fmt(format_args!("unrecognized bignum sign {}", *sign as char))
            }
            WrongClass(name, class) => {
                f.write_fmt(format_args!("wrong class, expected {name} got {class}"))
            }
            WrongInstanceVarCount(count) => {
                f.write_fmt(format_args!("wrong instance var count {count}"))
            }
            Utf8Error(err) => f.write_fmt(format_args!("utf8 conversion error {err}")),
            ParseFloatError(err) => f.write_fmt(format_args!("float parsing error {err}")),
            StringExtraIvars => f.write_str("ruby extra string ivars are unsupported"),
            NonUtf8String(string) => f.write_fmt(format_args!(
                "ruby string encoded with non-utf8 encoding ({string})"
            )),
            Eof => f.write_str("unexpected end of file"),
            Unsupported => f.write_str("unsupported"),
        }
    }
}

impl std::error::Error for Error {}
