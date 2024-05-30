// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    Struct = b'S',

    UserClass = b'C',

    Extended = b'e',

    UserMarshal = b'U',

    Data = b'd',
}

impl Tag {
    pub fn from_u8(value: u8) -> Option<Tag> {
        match value {
            b'0' => Some(Tag::Nil),
            b'T' => Some(Tag::True),
            b'F' => Some(Tag::False),
            b'i' => Some(Tag::Integer),
            b'f' => Some(Tag::Float),
            b'\"' => Some(Tag::String),
            b'[' => Some(Tag::Array),
            b'{' => Some(Tag::Hash),
            b'}' => Some(Tag::HashDefault),
            b':' => Some(Tag::Symbol),
            b';' => Some(Tag::Symlink),
            b'I' => Some(Tag::Instance),
            b'/' => Some(Tag::RawRegexp),
            b'c' => Some(Tag::ClassRef),
            b'm' => Some(Tag::ModuleRef),
            b'o' => Some(Tag::Object),
            b'@' => Some(Tag::ObjectLink),
            b'u' => Some(Tag::UserDef),
            b'S' => Some(Tag::Struct),
            b'C' => Some(Tag::UserClass),
            b'e' => Some(Tag::Extended),
            b'U' => Some(Tag::UserMarshal),
            b'd' => Some(Tag::Data),
            _ => None,
        }
    }

    pub fn is_object_link_referenceable(self) -> bool {
        !matches!(
            self,
            Self::Nil
                | Self::True
                | Self::False
                | Self::Integer
                | Self::Symbol
                | Self::Symlink
                | Self::ObjectLink
        )
    }
}

impl From<Tag> for u8 {
    fn from(value: Tag) -> Self {
        value as _
    }
}
