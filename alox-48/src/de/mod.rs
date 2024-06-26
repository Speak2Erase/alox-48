// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod deserializer;
mod error;
mod ignored;
mod impls;
mod traits;

pub use ignored::Ignored;

pub use error::Result;
pub use error::{Error, Kind, Unexpected};

pub use deserializer::Deserializer;
pub use traits::{
    ArrayAccess, Deserialize, DeserializeSeed, Deserializer as DeserializerTrait, HashAccess,
    InstanceAccess, IvarAccess, Visitor, VisitorInstance, VisitorOption,
};
