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
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, enum_as_inner::EnumAsInner)]
#[repr(u8)]
pub enum Tag {
    Nil = b'0',

    True = b'T',

    False = b'F',

    Integer = b'i',

    Float = b'f',

    String = b'\"',

    Array = b'[',

    Hash = b'{',

    HashDefault = b'}',

    Symbol = b':',

    Symlink = b';',

    Instance = b'I',

    RawRegexp = b'/',

    ClassRef = b'c',

    ModuleRef = b'm',

    Object = b'o',

    ObjectLink = b'@',

    UserDef = b'u',
}

impl TryFrom<u8> for Tag {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'0' => Ok(Self::Nil),
            b'T' => Ok(Self::True),
            b'F' => Ok(Self::False),
            b'i' => Ok(Self::Integer),
            b'f' => Ok(Self::Float),
            b'\"' => Ok(Self::String),
            b'[' => Ok(Self::Array),
            b'{' => Ok(Self::Hash),
            b'}' => Ok(Self::HashDefault),
            b':' => Ok(Self::Symbol),
            b';' => Ok(Self::Symlink),
            b'I' => Ok(Self::Instance),
            b'/' => Ok(Self::RawRegexp),
            b'c' => Ok(Self::ClassRef),
            b'm' => Ok(Self::ModuleRef),
            b'o' => Ok(Self::Object),
            b'@' => Ok(Self::ObjectLink),
            b'u' => Ok(Self::UserDef),
            _ => Err(crate::Error::WrongTag(value)),
        }
    }
}

impl From<Tag> for u8 {
    fn from(value: Tag) -> Self {
        value as _
    }
}
