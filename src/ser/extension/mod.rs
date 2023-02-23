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
#![allow(clippy::missing_errors_doc)]

use serde::ser::Error;

/// Serializer extension.
///
/// The default implementation will fail with [`crate::Error::Unsupported`].
pub trait SerializeExt: serde::Serializer {
    /// The type used to serialize Objects.
    ///
    /// This is mainly used to serialize [`crate::Object`].
    type SerializeObject: SerializeObject<Ok = Self::Ok, Error = Self::Error>;

    /// Serialize a symbol.
    ///
    /// Used to preserve types.
    fn serialize_symbol(self, symbol: &str) -> Result<Self::Ok, Self::Error>;

    /// Serialize userdata.
    ///
    /// This is mainly used for serializing [`crate::Userdata`].
    fn serialize_userdata(self, class: &str, data: &[u8]) -> Result<Self::Ok, Self::Error>;

    /// Serialize a ruby string, with extra fields.
    ///
    /// This is mainly used for serializing [`crate::RbString`].
    fn serialize_ruby_string(self, string: &crate::RbString) -> Result<Self::Ok, Self::Error>;

    /// Serialize an object.
    fn serialize_object(
        self,
        class: &str,
        len: usize,
    ) -> Result<Self::SerializeObject, Self::Error>;
}

/// This is essentially like [`serde::ser::SerializeStruct`] but for Objects.
///
/// `SerializeStruct` takes in a [`&'static str`] for field names, but that is simply not feasible for things like [`crate::Value`].
///
/// [`crate::Serializer`] will not add `@` to the beginning of field names too, like it does for [`serde::ser::SerializeStruct`].
///
/// ### Default impl
///
/// By default this is implemented for all type that implement [`serde::ser::SerializeStruct`].
/// It will error by default.
///
/// It is not recommended to interact with this API as it is highly volitale and open to change.
/// Please use [`crate::Value`] or [`crate::Object`] instead.
// FIXME: this is not the best solution. Maybe there could be another way?
pub trait SerializeObject {
    /// The ouput type produced by this serializer.
    type Ok;
    /// The error type produced by this serializer.
    type Error: serde::ser::Error;

    /// Serialize a field.
    /// The field name will **NOT** be prefixed by an `@`.
    fn serialize_field<T: ?Sized>(&mut self, key: &str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize;

    /// Indicate that a struct field has been skipped.
    #[inline]
    fn skip_field(&mut self, key: &str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }

    /// Finish serializing a struct.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<T> SerializeObject for T
where
    T: serde::ser::SerializeStruct,
{
    type Ok = <Self as serde::ser::SerializeStruct>::Ok;
    type Error = <Self as serde::ser::SerializeStruct>::Error;

    default fn serialize_field<A: ?Sized>(
        &mut self,
        _key: &str,
        _value: &A,
    ) -> Result<(), Self::Error>
    where
        A: serde::Serialize,
    {
        Err(Error::custom(
            "this serializer is not from alox-48. the default implementation of SerializeObject will fail with this error.",
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as serde::ser::SerializeStruct>::end(self)
    }
}

impl<T> SerializeExt for T
where
    T: serde::Serializer,
{
    type SerializeObject = Self::SerializeStruct;

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

    default fn serialize_object(
        self,
        _class: &str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Self::Error::custom(
            "this serializer is not from alox-48 and thus does not support serializing objects.",
        ))
    }
}
