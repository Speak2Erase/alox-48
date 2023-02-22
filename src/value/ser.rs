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

use serde::Serialize;

use crate::SerializeExt;

use super::{RbString, Symbol, Userdata, Value};

#[allow(clippy::panic_in_result_fn)]
impl Serialize for Value {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeExt,
    {
        todo!()
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeExt,
    {
        serializer.serialize_symbol(self)
    }
}

impl Serialize for RbString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeExt,
    {
        serializer.serialize_ruby_string(self)
    }
}

impl Serialize for Userdata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeExt,
    {
        serializer.serialize_userdata(&self.class, &self.data)
    }
}
