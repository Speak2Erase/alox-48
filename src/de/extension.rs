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

pub trait VisitorExt<'de>: Visitor<'de> {
    fn visit_userdata<E>(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError;

    fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>;

    fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError;
}

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
}

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
}
