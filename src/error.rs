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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected negative length {0}")]
    UnexpectedNegativeLength(i32),
    #[error("Wrong tag {0}")]
    WrongTag(u8),
    #[error("Symbol is invalid utf8 {0}")]
    SymbolInvalidUTF8(Utf8Error),
    #[error("Unresolved symlink {0}")]
    UnresolvedSymlink(usize),
    #[error("Float mantissa too long")]
    ParseFloatMantissaTooLong,
    #[error("Expected a symbol got {0:?}")]
    ExpectedSymbol(Tag),
    #[error("End of file")]
    Eof,
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
