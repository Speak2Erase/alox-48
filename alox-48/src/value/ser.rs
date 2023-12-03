// Copyright (C) 2022 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
use super::{Object, RbFields, RbHash, RbString, Symbol, Userdata, Value};
use crate::ser::{Error, Kind, Result};

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs [`alox_48::value::to_value`].
/// Unlike the main alox-48 serializer which goes from some value of `T` to binary data,
/// this one goes from `T` to `alox_48::value::Value`.
#[derive(Clone, Copy, Debug)]
pub struct Serializer;
