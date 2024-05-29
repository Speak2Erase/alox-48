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
    #[error("Overshot the provided len: {0}")]
    OvershotProvidedLen(usize),
    #[error("Undershot the provided len: {0}")]
    UndershotProvidedLen(usize),
    #[error("Custom error: {0}")]
    Message(String),
}

impl Error {
    pub fn custom(message: impl Into<String>) -> Self {
        Self {
            kind: Kind::Message(message.into()),
        }
    }
}
