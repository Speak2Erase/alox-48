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
use serde::ser::Error;

pub trait SerializeExt: serde::Serializer {
    fn serialize_symbol(self, symbol: &str) -> Result<Self::Ok, Self::Error>;

    fn serialize_userdata(self, class: &str, data: &[u8]) -> Result<Self::Ok, Self::Error>;

    fn serialize_ruby_string(self, string: &crate::RbString) -> Result<Self::Ok, Self::Error>;
}

impl<T> SerializeExt for T
where
    T: serde::Serializer,
{
    default fn serialize_symbol(self, _symbol: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(
            "this serializer is not from alox-48 and thus does not support serializing symbols.",
        ))
    }

    default fn serialize_userdata(
        self,
        _class: &str,
        _data: &[u8],
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(
            "this serializer is not from alox-48 and thus does not support serializing userdata.",
        ))
    }

    default fn serialize_ruby_string(
        self,
        _string: &crate::RbString,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(
            "this serializer is not from alox-48 and thus does not support serializing ruby strings.",
        ))
    }
}
