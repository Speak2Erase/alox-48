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

use std::hash::Hash;

use indexmap::IndexMap;
#[derive(Default, Debug, Clone, enum_as_inner::EnumAsInner)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i128),
    String(String),
    Array(RbArray),
    Hash(RbHash),
    Userdata(Vec<u8>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {}
}

pub type RbArray = Vec<Value>;
pub type RbHash = IndexMap<Value, Value>;
