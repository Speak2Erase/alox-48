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

use serde::de::Error as SerdeError;
use serde::de::{MapAccess, Unexpected, Visitor};

use super::VisitorExt;
use crate::value::RbFields;

impl<'de, T> VisitorExt<'de> for T
where
    T: Visitor<'de>,
{
    default fn visit_userdata<E>(self, _class: &'de str, _data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Err(SerdeError::invalid_type(
            Unexpected::Other("userdata"),
            &self,
        ))
    }

    default fn visit_object<A>(self, _class: &'de str, fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.visit_map(fields)
    }

    default fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        self.visit_borrowed_str(sym)
    }

    default fn visit_ruby_string<E>(
        self,
        str: &'de [u8],
        _fields: RbFields,
    ) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        let str = String::from_utf8_lossy(str);

        match str {
            std::borrow::Cow::Borrowed(str) => self.visit_borrowed_str(str),
            std::borrow::Cow::Owned(str) => self.visit_string(str),
        }
    }
}

/// Default implementation for VisitorExt.
impl<'de> VisitorExt<'de> for serde::de::IgnoredAny {
    fn visit_userdata<E>(self, _class: &'de str, _data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_object<A>(self, _class: &'de str, _fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_symbol<E>(self, _sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_ruby_string<E>(self, _str: &'de [u8], _fields: RbFields) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }
}
