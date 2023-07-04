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
use serde::de::{Unexpected, Visitor};

use super::VisitorExt;
use crate::{Object, RbString, Symbol, Userdata};

impl<'de, T> VisitorExt<'de> for T
where
    T: Visitor<'de>,
{
    default fn visit_userdata<E>(self, _userdata: Userdata) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Err(SerdeError::invalid_type(
            Unexpected::Other("userdata"),
            &self,
        ))
    }

    default fn visit_object<E>(self, object: Object) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        let fields = serde::de::value::MapDeserializer::new(
            object
                .fields
                .into_iter()
                .map(|(k, v)| (crate::Value::from(k), v)),
        );
        self.visit_map(fields)
    }

    default fn visit_symbol<E>(self, sym: Symbol) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        self.visit_string(sym.to_string())
    }

    #[allow(unused_imports, unused_variables)]
    default fn visit_ruby_string<E>(self, string: RbString) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        use crate::Value;

        #[cfg(feature = "warn-encoding")]
        if !string.is_empty() {
            match string
                .fields
                .get("E")
                .or_else(|| string.fields.get("encoding"))
            {
                Some(f) => match f {
                    Value::Bool(b) if !*b => {
                        eprintln!("warning: converting ascii ruby string to utf8");
                    }
                    Value::Bool(b) if *b => {}
                    Value::String(s) => {
                        eprintln!(
                            "warning: converting non-utf8 ruby string to utf8: {}",
                            s.to_string_lossy()
                        );
                    }
                    v => eprintln!("warning: unexpected encoding type on ruby string: {v:?}"),
                },
                None => eprintln!(
                    "warning: converting ruby string with no encoding (likely binary data) to utf8"
                ),
            }
        }

        let str = string.to_string_lossy();

        match str {
            std::borrow::Cow::Borrowed(str) => self.visit_str(str),
            std::borrow::Cow::Owned(str) => self.visit_string(str),
        }
    }
}

/// Default implementation for [`VisitorExt`].
impl<'de> VisitorExt<'de> for serde::de::IgnoredAny {
    fn visit_userdata<E>(self, _userdata: Userdata) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_object<E>(self, _object: Object) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_symbol<E>(self, _sym: Symbol) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }

    fn visit_ruby_string<E>(self, _str: RbString) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(serde::de::IgnoredAny)
    }
}
