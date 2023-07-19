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

use serde::de::{Error as _, Unexpected, Visitor};

use super::VisitorExt;
use crate::DeError;

impl<'de, T> VisitorExt<'de> for T
where
    T: Visitor<'de>,
{
    default fn visit_userdata(
        self,
        _class: &'de str,
        _data: &'de [u8],
    ) -> Result<Self::Value, DeError> {
        Err(DeError::invalid_type(Unexpected::Other("userdata"), &self))
    }

    default fn visit_object<A>(self, _class: &'de str, fields: A) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>,
    {
        self.visit_map(fields)
    }

    default fn visit_symbol(self, sym: &'de str) -> Result<Self::Value, DeError> {
        self.visit_borrowed_str::<DeError>(sym)
    }

    #[allow(unused_imports, unused_variables)]
    default fn visit_ruby_string<A>(
        self,
        data: &'de [u8],
        mut fields: A,
    ) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>,
    {
        use crate::Value;

        #[cfg(feature = "warn-encoding")]
        if !data.is_empty() {
            while let Some(key) = fields.next_key()? {
                if matches!(key, "E" | "encoding") {
                    match fields.next_value()? {
                        Value::Bool(b) if !b => {
                            eprintln!("warning: converting ascii ruby string to utf8");
                        }
                        Value::Bool(b) if b => (),
                        Value::String(s) => {
                            let encoding = s.to_string_lossy();
                            eprintln!(
                                "warning: converting non-utf8 ruby string to utf8: {encoding}"
                            );
                        }
                        v => eprintln!("warning: unexpected encoding type on ruby string: {v:?}"),
                    }
                    break;
                }
                fields.next_value::<serde::de::IgnoredAny>()?;
            }
            eprintln!(
                "warning: converting ruby string with no encoding (likely binary data) to utf8"
            );
        }

        let str = String::from_utf8_lossy(data);

        match str {
            std::borrow::Cow::Borrowed(str) => self.visit_borrowed_str(str),
            std::borrow::Cow::Owned(str) => self.visit_string(str),
        }
    }
}

/// Default implementation for [`VisitorExt`].
impl<'de> VisitorExt<'de> for serde::de::IgnoredAny {
    fn visit_userdata(self, _class: &'de str, _data: &'de [u8]) -> Result<Self::Value, DeError> {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_symbol(self, sym: &'de str) -> Result<Self::Value, DeError> {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_ruby_string<A>(self, data: &'de [u8], fields: A) -> Result<Self::Value, DeError>
    where
        A: serde::de::MapAccess<'de, Error = DeError>,
    {
        Ok(serde::de::IgnoredAny)
    }
}
