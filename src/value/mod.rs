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

mod de;

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Default, Debug, Clone, enum_as_inner::EnumAsInner, Deserialize)]
pub enum Value {
    #[default]
    Nil,
    Array(RbArray),
    Bool(bool),
    Float(f64),
    Integer(i128),
    String(String),
    // Hash(RbHash),
}

#[derive(Debug, Clone)]
pub struct RbObject {
    pub class: String,
    pub members: RbHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RbUserData {
    pub class: String,
    pub data: Vec<u8>,
}

pub type RbArray = Vec<Value>;
pub type RbHash = IndexMap<Value, Value>;
