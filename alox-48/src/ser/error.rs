// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
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
    #[error("Overshot the provided len {0} > {1}")]
    OvershotProvidedLen(usize, usize),
    #[error("Undershot the provided len {0} < {1}")]
    UndershotProvidedLen(usize, usize),
    #[error("Custom error: {0}")]
    Message(String),
    #[error("Tried to serialize a key without a value")]
    KeyAfterKey,
    #[error("Tried to serialize a value before its key")]
    ValueAfterValue,
}

impl Error {
    pub fn custom(message: impl std::fmt::Display) -> Self {
        Self {
            kind: Kind::Message(message.to_string()),
        }
    }
}
