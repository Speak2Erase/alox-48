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
mod from;
mod ser;

use enum_as_inner::EnumAsInner;
use std::hash::Hash;

use indexmap::IndexMap;
#[derive(Default, Debug, Clone, EnumAsInner)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(String),
    Symbol(String),
    Array(RbArray),
    Hash(RbHash),
    Userdata(Userdata),
    Object(Object),
}

#[derive(Hash, PartialEq, Eq, Default, Debug, Clone)]
pub struct Userdata {
    pub class: String,
    pub data: Vec<u8>,
}

#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct Object {
    pub class: String,
    pub fields: IndexMap<String, Value>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Nil => other.is_nil(),
            Value::Bool(b) => {
                if let Value::Bool(b2) = other {
                    b == b2
                } else {
                    false
                }
            }
            Value::Float(f) => {
                if let Value::Float(f2) = other {
                    (f.is_nan() && f2.is_nan()) || f == f2
                } else {
                    false
                }
            }
            Value::Integer(i) => {
                if let Value::Integer(i2) = other {
                    i == i2
                } else {
                    false
                }
            }
            Value::String(s) => {
                if let Value::String(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Value::Symbol(s) => {
                if let Value::Symbol(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Value::Array(v) => {
                if let Value::Array(v2) = other {
                    v == v2
                } else {
                    false
                }
            }
            Value::Hash(h) => {
                if let Value::Hash(h2) = other {
                    h == h2
                } else {
                    false
                }
            }
            Value::Object(o) => {
                if let Value::Object(o2) = other {
                    o == o2
                } else {
                    false
                }
            }
            Value::Userdata(u) => {
                if let Value::Userdata(u2) = other {
                    u == u2
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => {}
            Value::Bool(b) => b.hash(state),
            Value::Float(_) => {}
            Value::Integer(i) => i.hash(state),
            Value::String(s) => s.hash(state),
            Value::Symbol(s) => s.hash(state),
            Value::Array(v) => v.hash(state),
            Value::Hash(_) => {}
            Value::Object(_) => {}
            Value::Userdata(u) => u.hash(state),
        }
    }
}

pub type RbArray = Vec<Value>;
pub type RbHash = IndexMap<Value, Value>;
