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
use super::{Deserialize, DeserializerTrait, Result, Visitor};

macro_rules! primitive_int_impl {
    ($($primitive:ty),*) => {
        $(impl<'de> Deserialize<'de> for $primitive {
            fn deserialize<D>(deserializer: D) -> Result<Self>
            where
                D: DeserializerTrait<'de>,
            {
                struct IVisitor;

                impl<'de> Visitor<'de> for IVisitor {
                    type Value = $primitive;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("an integer")
                    }

                    fn visit_i32(self, v: i32) -> Result<Self::Value> {
                        Ok(v as $primitive)
                    }
                }

                deserializer.deserialize(IVisitor)
            }
        })*
    };
}

primitive_int_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
