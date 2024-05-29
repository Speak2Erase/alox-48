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
use super::Value;
use indexmap::IndexMap;

mod instance;
mod object;
mod rb_string;
mod sym;
mod symbol;
mod userdata;

pub use instance::Instance;
pub use object::Object;
pub use rb_string::RbString;
pub use sym::Sym;
pub use symbol::Symbol;
pub use userdata::Userdata;

/// Shorthand type alias for a ruby array.
pub type RbArray = Vec<Value>;
/// Shorthand type alias for a ruby hash.
pub type RbHash = IndexMap<Value, Value>;

/// A type alias used to represent fields of objects.
/// All objects store a [`Symbol`] to represent the key for instance variable, and we do that here too.
pub type RbFields = IndexMap<Symbol, Value>;
