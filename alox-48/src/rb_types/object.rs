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
use super::{RbFields, Symbol};

use crate::{
    de::Result as DeResult, ser::Result as SerResult, Deserialize, DeserializerTrait, IvarAccess,
    Serialize, SerializeIvars, SerializerTrait, Sym, Visitor,
};

/// A type equivalent to ruby's `Object`.
#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct Object {
    /// This object's class.
    pub class: Symbol,
    /// The fields on this object.
    pub fields: RbFields,
}

impl Object {
    /// Splits this object into its constituants.
    #[allow(clippy::must_use_candidate)]
    pub fn into_parts(self) -> (Symbol, RbFields) {
        (self.class, self.fields)
    }
}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.fields.len().hash(state);

        for (var, field) in &self.fields {
            var.hash(state);
            field.hash(state);
        }
    }
}

struct ObjectVisitor;

impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = Object;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an object")
    }

    fn visit_object<A>(self, class: &'de Sym, mut instance_variables: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let class = class.to_symbol();
        let mut fields = RbFields::with_capacity(instance_variables.len());

        while let Some((k, v)) = instance_variables.next_entry()? {
            fields.insert(k.to_symbol(), v);
        }

        Ok(Object { class, fields })
    }
}

impl<'de> Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> DeResult<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(ObjectVisitor)
    }
}

impl Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
    where
        S: SerializerTrait,
    {
        let mut ivars = serializer.serialize_object(&self.class, self.fields.len())?;
        for (k, v) in &self.fields {
            ivars.serialize_entry(k, v)?;
        }
        ivars.end()
    }
}
