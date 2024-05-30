// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use crate::{
    de::Result as DeResult, Deserialize, DeserializerTrait, IvarAccess, RbFields, RbString,
    Serialize, SerializeIvars, VisitorInstance,
};

/// A type representing a ruby object with extra instance variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance<T> {
    /// The inner value.
    pub value: T,
    /// The extra instance variables attached to this object.
    pub fields: RbFields,
}

struct InstanceVisitor<T>(PhantomData<T>);

impl<'de, T> VisitorInstance<'de> for InstanceVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Instance<T>;

    fn visit<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = T::deserialize(deserializer)?;
        let fields = RbFields::new();
        Ok(Instance { value, fields })
    }

    fn visit_instance<A>(self, access: A) -> DeResult<Self::Value>
    where
        A: crate::InstanceAccess<'de>,
    {
        let (value, mut ivar) = access.value_deserialize()?;

        let mut fields = RbFields::with_capacity(ivar.len());
        while let Some((field, value)) = ivar.next_entry()? {
            fields.insert(field.to_symbol(), value);
        }
        Ok(Instance { value, fields })
    }
}

impl<'de, T> Deserialize<'de> for Instance<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> DeResult<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize_instance(InstanceVisitor(PhantomData))
    }
}

impl<T> Serialize for Instance<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> crate::ser::Result<S::Ok>
    where
        S: crate::SerializerTrait,
    {
        if self.fields.is_empty() {
            self.value.serialize(serializer)
        } else {
            let mut fields = serializer.serialize_instance(&self.value, self.fields.len())?;
            for (k, v) in &self.fields {
                fields.serialize_entry(k, v)?;
            }
            fields.end()
        }
    }
}

impl<T> Instance<T> {
    /// Take the inner value of this instance.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Splits this string into its constituants.
    pub fn into_parts(self) -> (T, RbFields) {
        (self.value, self.fields)
    }
}

impl Instance<RbString> {
    /// Return the encoding of this string, if it has one.
    pub fn encoding(&self) -> Option<&crate::Value> {
        self.fields.get("E").or_else(|| self.fields.get("encoding"))
    }
}

macro_rules! utf8_enc {
    () => {{
        let mut f = RbFields::new();
        f.insert("E".into(), true.into());

        f
    }};
}

impl From<String> for Instance<RbString> {
    fn from(value: String) -> Self {
        Self {
            value: value.into(),
            fields: utf8_enc!(),
        }
    }
}

impl From<&str> for Instance<RbString> {
    fn from(value: &str) -> Self {
        Self {
            value: value.into(),
            fields: utf8_enc!(),
        }
    }
}

impl<T> std::hash::Hash for Instance<T>
where
    T: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.fields.len().hash(state);
        for (var, field) in &self.fields {
            var.hash(state);
            field.hash(state);
        }
    }
}
