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
use super::{Result, Serialize, SerializerTrait};

macro_rules! primitive_int_impl {
    ($($primitive:ty),*) => {
        $(impl Serialize for $primitive {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
            where
                S: SerializerTrait
            {
                serializer.serialize_i32(*self as i32)
            }
        })*
    };
}

primitive_int_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl Serialize for bool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_bool(*self)
    }
}

impl<'a, T> Serialize for &'a T
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        T::serialize(self, serializer)
    }
}

impl<'a, T> Serialize for &'a mut T
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        T::serialize(self, serializer)
    }
}

impl Serialize for str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_rust_string(self)
    }
}

impl<T> Serialize for [T]
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.collect_array(self)
    }
}
