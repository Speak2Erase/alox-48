// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::{DeError, Deserialize, DeserializerTrait};

mod de;
mod ser;

pub use de::{Deserializer, Trace};

/// Deserialize a value from a given deserializer.
///
/// Automatically tracks the path to the error, and returns it as a `Trace`.
pub fn deserialize<'de, T>(input: impl DeserializerTrait<'de>) -> Result<T, (DeError, Trace)>
where
    T: Deserialize<'de>,
{
    let mut track = Trace::new();
    let deserializer = Deserializer::new(input, &mut track);

    let value = T::deserialize(deserializer);

    match value {
        Ok(value) => Ok(value),
        Err(err) => Err((err, track)),
    }
}
