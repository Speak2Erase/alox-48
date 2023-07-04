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
use super::Value;

#[allow(clippy::panic_in_result_fn)]
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::SerializeExt,
    {
        match self {
            Value::Nil => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Integer(i) => serializer.serialize_i64(*i),
            Value::String(s) => serializer.serialize_ruby_string(s),
            Value::Symbol(s) => serializer.serialize_symbol(s),
            Value::Array(a) => a.serialize(serializer),
            Value::Hash(h) => h.serialize(serializer),
            Value::Userdata(u) => u.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
        }
    }
}
