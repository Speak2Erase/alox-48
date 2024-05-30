// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::Value;
use indexmap::IndexMap;

mod instance;
mod object;
mod rb_string;
mod rb_struct;
mod sym;
mod symbol;
mod userdata;

pub use instance::Instance;
pub use object::Object;
pub use rb_string::RbString;
pub use rb_struct::RbStruct;
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
