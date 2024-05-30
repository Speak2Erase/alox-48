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
