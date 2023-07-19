// Copyright (C) 2023 Lily Lyons
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
use super::{Object, RbArray, RbHash, RbString, Symbol, Userdata, Value};

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Bool(b) => b.fmt(f),
            Value::Float(n) => n.fmt(f),
            Value::Integer(i) => i.fmt(f),
            Value::String(s) => f.write_fmt(format_args!("{:?}", s.to_string_lossy())),
            Value::Symbol(s) => s.fmt(f),
            Value::Array(a) => a.fmt(f),
            Value::Object(o) => {
                let mut d = f.debug_struct(o.class.as_str());

                for (k, v) in &o.fields {
                    d.field(k.as_str(), v);
                }

                d.finish()
            }
            Value::Hash(h) => h.fmt(f),
            Value::Userdata(u) => f
                .debug_struct(u.class.as_str())
                .field("data", &u.data)
                .finish(),
        }
    }
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

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self {
            Value::Bool(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Value::Integer(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::Float(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self {
            Value::String(v) => other.as_bytes() == v.as_slice(),
            Value::Symbol(v) => other.as_str() == v.as_str(),
            _ => false,
        }
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        match self {
            Value::String(v) => other.as_bytes() == v.as_slice(),
            Value::Symbol(v) => other == v.as_str(),
            _ => false,
        }
    }
}

impl PartialEq<RbString> for Value {
    fn eq(&self, other: &RbString) -> bool {
        match self {
            Value::String(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<Symbol> for Value {
    fn eq(&self, other: &Symbol) -> bool {
        match self {
            Value::Symbol(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<Object> for Value {
    fn eq(&self, other: &Object) -> bool {
        match self {
            Value::Object(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<Userdata> for Value {
    fn eq(&self, other: &Userdata) -> bool {
        match self {
            Value::Userdata(v) => other == v,
            _ => false,
        }
    }
}

// TODO: PartialEq for T where Value: PartialEq<T> in a Vec<T>
impl PartialEq<RbArray> for Value {
    fn eq(&self, other: &RbArray) -> bool {
        match self {
            Value::Array(v) => other == v,
            _ => false,
        }
    }
}

impl PartialEq<RbHash> for Value {
    fn eq(&self, other: &RbHash) -> bool {
        match self {
            Value::Hash(v) => other == v,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => {}
            Value::Bool(b) => b.hash(state),
            Value::Float(f) => f.to_bits().hash(state), // not the best but eh whos using a float as a hash key
            Value::Integer(i) => i.hash(state),
            Value::String(s) => {
                s.data.hash(state);
            }
            Value::Symbol(s) => s.0.hash(state),
            Value::Array(v) => v.hash(state),
            Value::Hash(h) => {
                h.len().hash(state);
                for (key, value) in h.iter() {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Value::Object(o) => o.hash(state),
            Value::Userdata(u) => u.hash(state),
        }
    }
}
