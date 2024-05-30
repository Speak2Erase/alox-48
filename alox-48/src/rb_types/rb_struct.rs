// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{RbFields, Symbol};

use crate::{
    de::Result as DeResult, ser::Result as SerResult, Deserialize, DeserializerTrait, IvarAccess,
    Serialize, SerializeIvars, SerializerTrait, Sym, Visitor,
};

/// A type equivalent to ruby's `Struct`.
#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct RbStruct {
    /// This struct's class.
    pub class: Symbol,
    /// The fields on this struct.
    pub fields: RbFields,
}

impl RbStruct {
    /// Splits this struct into its constituants.
    #[allow(clippy::must_use_candidate)]
    pub fn into_parts(self) -> (Symbol, RbFields) {
        (self.class, self.fields)
    }
}

impl std::hash::Hash for RbStruct {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.fields.len().hash(state);

        for (var, field) in &self.fields {
            var.hash(state);
            field.hash(state);
        }
    }
}

struct StructVisitor;

impl<'de> Visitor<'de> for StructVisitor {
    type Value = RbStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a ruby struct")
    }

    fn visit_struct<A>(self, class: &'de Sym, mut instance_variables: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let class = class.to_symbol();
        let mut fields = RbFields::with_capacity(instance_variables.len());

        while let Some((k, v)) = instance_variables.next_entry()? {
            fields.insert(k.to_symbol(), v);
        }

        Ok(RbStruct { class, fields })
    }
}

impl<'de> Deserialize<'de> for RbStruct {
    fn deserialize<D>(deserializer: D) -> DeResult<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(StructVisitor)
    }
}

impl Serialize for RbStruct {
    fn serialize<S>(&self, serializer: S) -> SerResult<S::Ok>
    where
        S: SerializerTrait,
    {
        let mut ivars = serializer.serialize_struct(&self.class, self.fields.len())?;
        for (k, v) in &self.fields {
            ivars.serialize_entry(k, v)?;
        }
        ivars.end()
    }
}
