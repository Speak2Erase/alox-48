// Copyright (C) 2024 Lily Lyons
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
// along with alox-48.  If not, see <https://www.gnu.org/licenses/>.

use std::marker::PhantomData;

use crate::{
    de::Result as DeResult, Deserialize, DeserializerTrait, IvarAccess, RbFields, RbString,
    Serialize, SerializeIvars, VisitorInstance,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance<T> {
    pub value: T,
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
