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

mod deserializer;
mod error;
mod ignored;
mod impls;
mod traits;

pub(crate) use error::Result;
pub use error::{Error, Kind};

pub use deserializer::Deserializer;
pub use traits::{
    ArrayAccess, Deserialize, Deserializer as DeserializerTrait, HashAccess, InstanceAccess,
    IvarAccess, Visitor,
};
