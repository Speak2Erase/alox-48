// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{Object, RbArray, RbHash, RbString, Symbol, Userdata, Value};

impl PartialEq for Value {
    #[allow(clippy::too_many_lines)]
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
            Value::Instance(i) => {
                if let Value::Instance(i2) = other {
                    i == i2
                } else {
                    false
                }
            }
            Value::Regex { data, flags } => {
                if let Value::Regex {
                    data: data2,
                    flags: flags2,
                } = other
                {
                    data == data2 && flags == flags2
                } else {
                    false
                }
            }
            Value::RbStruct(s) => {
                if let Value::RbStruct(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Value::Class(c) => {
                if let Value::Class(c2) = other {
                    c == c2
                } else {
                    false
                }
            }
            Value::Module(m) => {
                if let Value::Module(m2) = other {
                    m == m2
                } else {
                    false
                }
            }
            Value::Extended { module, value } => {
                if let Value::Extended {
                    module: module2,
                    value: value2,
                } = other
                {
                    module == module2 && value == value2
                } else {
                    false
                }
            }
            Value::UserClass { class, value } => {
                if let Value::UserClass {
                    class: class2,
                    value: value2,
                } = other
                {
                    class == class2 && value == value2
                } else {
                    false
                }
            }
            Value::UserMarshal { class, value } => {
                if let Value::UserMarshal {
                    class: class2,
                    value: value2,
                } = other
                {
                    class == class2 && value == value2
                } else {
                    false
                }
            }
            Value::Data { class, value } => {
                if let Value::Data {
                    class: class2,
                    value: value2,
                } = other
                {
                    class == class2 && value == value2
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

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
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
            Value::Instance(i) => i.value.as_ref() == other,
            _ => false,
        }
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        match self {
            Value::String(v) => other.as_bytes() == v.as_slice(),
            Value::Symbol(v) => other == v.as_str(),
            Value::Instance(i) => i.value.as_ref() == other,
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
        std::mem::discriminant(self).hash(state);
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
                for (key, value) in h {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Value::Object(o) => o.hash(state),
            Value::Userdata(u) => u.hash(state),
            Value::Instance(i) => i.hash(state),
            Value::Regex { data, flags } => {
                data.data.hash(state);
                flags.hash(state);
            }
            Value::RbStruct(s) => s.hash(state),
            Value::Class(c) => c.hash(state),
            Value::Module(m) => m.hash(state),
            Value::Extended { module, value } => {
                module.hash(state);
                value.hash(state);
            }
            Value::UserClass { class, value }
            | Value::UserMarshal { class, value }
            | Value::Data { class, value } => {
                class.hash(state);
                value.hash(state);
            }
        }
    }
}
