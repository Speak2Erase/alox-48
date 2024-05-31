// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
mod error;
mod impls;
mod serializer;
mod traits;

pub use error::Result;

pub use error::{Error, Kind};
pub use serializer::Serializer;

pub use traits::{
    Serialize, SerializeArray, SerializeHash, SerializeIvars, Serializer as SerializerTrait,
};

/// A helper to ensure byte slices are serialized as strings.
///
/// By default, this crate serializes *all* `[T]` as arrays, even if `T` is `u8`.
/// Without specialization, this isn't really possible to fix.
///
/// This type is a workaround for that issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteString<'a>(pub &'a [u8]);

impl Serialize for ByteString<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_string(self.0)
    }
}
