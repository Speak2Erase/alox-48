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
#![allow(dead_code, unused_variables)]

use std::borrow::Cow;

use serde::de;

type Error = crate::Error;

pub fn from_bytes<'de, T>(data: &'de [u8]) -> super::Result<T>
where
    T: serde::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data)?;
    T::deserialize(&mut deserializer)
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    cursor: usize,
    objtable: Vec<usize>,
    symlink: Vec<&'de str>,
    remove_ivar_prefix: bool,
}

#[derive(Debug, enum_as_inner::EnumAsInner)]
enum StringType<'de> {
    String(Cow<'de, str>),
    Bytes(&'de [u8]),
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de [u8]) -> crate::Result<Self> {
        if input[0..=1] != [4, 8] {
            return Err(Error::VersionError([input[0], input[1]]));
        }

        Ok(Self {
            input,
            cursor: 2,
            objtable: vec![],
            symlink: vec![],
            remove_ivar_prefix: false,
        })
    }

    /// FIXME: Make these into nom parsers
    fn read(&mut self) -> Result<u8, Error> {
        self.cursor += 1;
        if self.cursor > self.input.len() {
            Err(Error::Eof)
        } else {
            Ok(self.input[self.cursor - 1])
        }
    }

    #[inline]
    fn peek(&self) -> u8 {
        self.input[self.cursor]
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Error> {
        if self.cursor + len > self.input.len() {
            Err(Error::Eof)
        } else {
            self.cursor += len;
            Ok(&self.input[(self.cursor - len)..self.cursor])
        }
    }

    #[inline]
    fn read_int<T: Copy + 'static>(&mut self) -> Result<T, Error>
    where
        i128: num_traits::AsPrimitive<T>,
    {
        use num_traits::AsPrimitive;

        let c = self.read()? as i8;

        let int: T = (match c {
            0 => 0,
            5..=127 => (c - 5) as _,
            -128..=-5 => (c + 5) as _,
            c => {
                let mut x = 0;

                for i in 0..c {
                    x |= (self.read()? as i128) << (8 * i);
                }

                x
            }
        })
        .as_();

        Ok(int)
    }

    #[inline]
    fn read_len_bytes(&mut self) -> Result<&'de [u8], Error> {
        let len = self.read_int::<usize>()?;
        self.read_bytes(len)
    }

    fn read_string(&mut self) -> Result<StringType<'de>, Error> {
        let bytes = self.read_len_bytes()?;

        let str = std::str::from_utf8(bytes)
            .map(|s| StringType::String(s.into()))
            .unwrap_or_else(|_| StringType::Bytes(bytes));

        Ok(str)
    }

    fn parse_int<T: Copy + 'static>(&mut self) -> Result<T, Error>
    where
        i128: num_traits::AsPrimitive<T>,
    {
        let kind = self.read()?;

        self.read_int()
    }

    fn parse_float(&mut self) -> Result<f64, Error> {
        let res = match self.read()? {
            b'f' => match self.read_string()?.into_string().unwrap().as_ref() {
                "inf" => f64::INFINITY,
                "-inf" => f64::NEG_INFINITY,
                "nan" => f64::NAN,
                str => str.parse()?,
            },
            b'i' => self.read_int()?,
            b'l' => return Err(Error::Unsupported),
            kind => return Err(Self::type_error(kind, "float or bignum/fixnum")),
        };

        Ok(res)
    }

    fn parse_sym(&mut self) -> Result<&'de str, Error> {
        match self.read()? {
            b':' => {
                let str = self.read_string()?.into_string().unwrap();

                let mut str = match str {
                    Cow::Borrowed(str) => str,
                    Cow::Owned(_) => unreachable!(),
                };

                if self.remove_ivar_prefix {
                    str = &str[1..];
                }

                self.symlink.push(str);

                Ok(str)
            }
            b';' => self.read_symtable(),
            kind => Err(Self::type_error(kind, "symbol")),
        }
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        match self.read()? {
            b'T' => Ok(true),
            b'F' => Ok(false),
            kind => Err(Self::type_error(kind, "bool")),
        }
    }

    fn read_symtable(&mut self) -> Result<&'de str, super::Error> {
        let index = self.read_int::<usize>()?;

        Ok(self.symlink[index])
    }

    fn parse_string(&mut self) -> Result<StringType<'de>, super::Error> {
        let bytes = self.read_len_bytes()?;

        let instance_var_num = self.read_int::<usize>()?;

        if instance_var_num > 1 {
            return Err(Error::StringExtraIvars);
        }
        let _enc = self.parse_sym()?;

        match self.read()? {
            b'T' => {
                let str = std::str::from_utf8(bytes)
                    .map(|s| StringType::String(s.into()))
                    .unwrap_or_else(|_| StringType::Bytes(bytes));

                Ok(str)
            }
            b'F' => Ok(StringType::Bytes(bytes)),
            b'"' => {
                let encoding = self.read_string()?.into_string().unwrap();
                println!("warning: non utf8 string {encoding}");

                let str = std::str::from_utf8(bytes)
                    .map(|s| StringType::String(s.into()))
                    .unwrap_or_else(|_| StringType::Bytes(bytes));

                Ok(str)
            }
            b'@' => {
                let _index = self.read_int::<usize>()?;

                println!("warning: non utf8 string");

                let str = std::str::from_utf8(bytes)
                    .map(|s| StringType::String(s.into()))
                    .unwrap_or_else(|_| StringType::Bytes(bytes));

                Ok(str)
            }
            kind => Err(Self::type_error(kind, "bool/string")),
        }
    }

    fn parse_instance<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        use de::Deserializer as _;

        match self.read()? {
            b'I' => match self.read()? {
                b'"' => {
                    let str = self.parse_string()?;

                    match str {
                        StringType::String(str) => match str {
                            Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
                            Cow::Owned(str) => visitor.visit_string(str),
                        },
                        StringType::Bytes(bytes) => visitor.visit_borrowed_bytes(bytes),
                    }
                }
                kind => Err(Self::type_error(kind, "object/string")),
            },
            b'@' => {
                // FIXME: This is slow!
                let index = self.read_int::<usize>()? - 1;

                let cursor = self.cursor;
                self.cursor = self.objtable[index];

                let result = self.deserialize_any(visitor);

                self.cursor = cursor;

                result
            }
            kind => Err(Self::type_error(kind, "string/object")),
        }
    }

    fn type_error(kind: u8, typ: &'static str) -> super::Error {
        let msg = match kind {
            b'I' | b'@' => "unexpected object instance",
            b'o' => "unexpected object",
            b'i' => "unexpected fixnum",
            b'l' => "unexpected bignum",
            b'f' => "unexpected float",
            b'"' => "unexpected string",
            b':' | b';' => "unexpected symbol",
            b'T' => "unexpected bool (true)",
            b'F' => "unexpected bool (false)",
            b'0' => "unexpected nil",
            b'[' => "unexpected array",
            b'{' | b'}' => "unexpected hash",
            b'u' => "unexpected userdata",
            _ => return super::Error::UnsupportedType(kind),
        };

        super::Error::TypeError(format!("{msg}, expected {typ}"))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.objtable.push(self.cursor);
        match self.peek() {
            b':' | b';' => visitor.visit_str(self.parse_sym()?),
            b'T' | b'F' => visitor.visit_bool(self.parse_bool()?),
            b'f' => visitor.visit_f64(self.parse_float()?),
            b'i' => visitor.visit_i128(self.parse_int()?),
            b'l' => Err(Error::Unsupported),
            b'I' | b'@' => self.parse_instance(visitor),
            b'[' => {
                self.read()?;
                let length = self.read_int()?;

                visitor.visit_seq(ArraySeq::new(self, length))
            }
            b'{' => {
                self.read()?;
                let length = self.read_int()?;

                visitor.visit_map(ArraySeq::new(self, length))
            }
            b'}' => {
                self.read()?;
                let length = self.read_int()?;

                let res = visitor.visit_map(ArraySeq::new(self, length))?;

                self.deserialize_any(serde::de::IgnoredAny)?;

                Ok(res)
            }
            b'u' => {
                self.read()?;
                let _name = self.parse_sym()?;

                visitor.visit_borrowed_bytes(self.read_len_bytes()?)
            }
            b'"' => {
                self.read()?;

                let str = self.read_string()?;

                match str {
                    StringType::String(str) => match str {
                        Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
                        Cow::Owned(str) => visitor.visit_string(str),
                    },
                    StringType::Bytes(bytes) => visitor.visit_borrowed_bytes(bytes),
                }
            }
            b'o' => {
                self.read()?;
                let class = self.parse_sym()?;

                let length = self.read_int::<usize>()?;

                visitor.visit_map(ClassSeq::new(self, length))
            }
            b'0' => {
                self.read()?;

                visitor.visit_unit()
            }
            kind => Err(Deserializer::type_error(kind, "any")),
        }
    }

    serde::forward_to_deserialize_any! {
        bool char str string
        bytes byte_buf newtype_struct seq tuple
        tuple_struct enum map identifier ignored_any
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i128(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u128(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()? as _)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.peek() {
            b'0' => {
                self.read()?;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read()? {
            0 => visitor.visit_unit(),
            kind => Err(Deserializer::type_error(kind, "() (unit)")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read()? {
            b'o' => {
                let class = self.parse_sym()?.split("::").last().unwrap();
                if name != class {
                    return Err(Error::WrongClass(name, class.to_string()));
                }

                let ivar_count = self.read_int::<usize>()?;
                if ivar_count > 0 {
                    return Err(Error::WrongInstanceVarCount(ivar_count));
                }

                visitor.visit_unit()
            }
            kind => Err(Deserializer::type_error(kind, "object")),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let kind = self.read()?;
        if kind != b'o' {
            return Err(Deserializer::type_error(kind, "object"));
        }

        let class = self.parse_sym()?.split("::").last().unwrap();
        if name != class {
            return Err(Error::WrongClass(name, class.to_string()));
        }

        let length = self.read_int::<usize>()?;

        visitor.visit_map(ClassSeq::new(self, length))
    }
}

struct ArraySeq<'a, 'de> {
    length: usize,
    index: usize,
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ArraySeq<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, length: usize) -> Self {
        Self {
            length,
            index: 0,
            de,
        }
    }
}

impl<'a, 'de> de::SeqAccess<'de> for ArraySeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.index += 1;

        if self.index > self.length {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'a, 'de> de::MapAccess<'de> for ArraySeq<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.index += 1;

        if self.index > self.length {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct ClassSeq<'a, 'de> {
    length: usize,
    index: usize,
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ClassSeq<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, length: usize) -> Self {
        Self {
            length,
            index: 0,

            de,
        }
    }
}

impl<'a, 'de> de::MapAccess<'de> for ClassSeq<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.index += 1;

        if self.index > self.length {
            return Ok(None);
        }

        self.de.remove_ivar_prefix = true;

        let res = seed.deserialize(&mut *self.de).map(Some);

        self.de.remove_ivar_prefix = false;

        res
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}
