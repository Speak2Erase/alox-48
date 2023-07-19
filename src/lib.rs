#![warn(rust_2018_idioms, clippy::pedantic)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::all
)]
#![feature(min_specialization)]

//! alox-48
//! (short for aluminum oxide 48)
//!
//! alox-48 is a crate using serde designed to deserialize ruby marshal data.
//! It uses the currently nightly feature `min_specialization` to extend serde's data model,
//! preventing the loss of information in (de)serialization.
//!
//! alox-48 supports both serialization and deserialization,
//! but some types present in rust's type system and ruby's marshal format are unsupported.
//!
//! Most notably, alox-48 does NOT support object links. Object links are marshal's way of saving space,
//! if an object was serialized already a "link" indicating when it was serialized is serialized instead.
//!
//! ```rb
//! class MyClass
//!  def initialize
//!    @var = 1
//!    @string = "hiya!"
//!  end
//! end
//!
//! a = MyClass.new
//! Marshal.dump([a, a, a])
//! # The array here has 3 indices all "pointing" to the same object.
//! # Instead of serializing MyClass 3 times, Marshal will serialize it once and replace the other 2 occurences with object links.
//! # When deserializing, Marshal will preserve object links and all 3 elements in the array will point to the same object.
//! # In alox-48, this is not the case. Each index will be a "unique" ""object"".
//! ```
//!
//! This does not map well to rust, as it inherently requires a garbage collector.
//! alox-48 will still deserialize object links, however it will simply deserialize them as a copy instead.

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

pub(crate) mod tag;

/// Deserialization via marshal.
///
/// [`crate::VisitorExt`] is responsible for extending serde.
pub mod de;
/// Marshal serialization.
///
/// [`crate::SerializeExt`] is responsible for extending serde.
pub mod ser;
/// Untyped ruby values and rust equivalents of some ruby types (Hash, Array, etc).
///
/// Useful for deserializing untyped data.
pub mod value;

pub use de::{Deserializer, Error as DeError, VisitorExt};
pub use ser::{Error as SerError, SerializeExt, Serializer};
pub use value::{Object, RbArray, RbHash, RbString, Symbol, Userdata, Value};

/// Deserialize data from some bytes.
/// It's a convenience function over [`Deserializer::new`] and [`serde::Deserialize`].
#[allow(clippy::missing_errors_doc)]
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T, DeError>
where
    T: serde::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data)?;
    T::deserialize(&mut deserializer)
}

/// Serialize the type into bytes.
///
/// # Errors
/// Errors if the type contains data `alox_48` does not support.
/// These include:
/// - Enums
/// - Newtype Structs
/// - Unit Structs
pub fn to_bytes<T>(data: T) -> Result<Vec<u8>, SerError>
where
    T: serde::Serialize,
{
    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;
    Ok(serializer.output)
}

#[cfg(test)]
mod ints {
    #[test]
    fn deserialize() {
        let bytes = &[0x04, 0x08, 0x69, 0x19];

        let int: u8 = crate::from_bytes(bytes).unwrap();

        assert_eq!(int, 20);
    }

    #[test]
    fn round_trip() {
        let int = 123;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_value() {
        let value = crate::Value::Integer(123);

        let bytes = crate::to_bytes(&value).unwrap();

        let value2: crate::Value = crate::from_bytes(&bytes).unwrap();

        assert_eq!(value, value2);
    }

    #[test]
    fn negatives() {
        let bytes = &[0x04, 0x08, 0x69, 0xfd, 0x1d, 0xf0, 0xfc];

        let int: i32 = crate::from_bytes(bytes).unwrap();

        assert_eq!(int, -200_675);
    }
}

#[cfg(test)]
mod strings {
    #[test]
    fn deserialize() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let str: &str = crate::from_bytes(bytes).unwrap();

        assert_eq!(str, "hello there!");
    }

    #[test]
    fn round_trip() {
        let str = "round trip!!";

        let bytes = crate::to_bytes(str).unwrap();

        let str2: &str = crate::from_bytes(&bytes).unwrap();

        assert_eq!(str, str2);
    }

    #[test]
    fn weird_encoding() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x0d, 0x65, 0x6e, 0x63, 0x6f, 0x64, 0x69, 0x6e, 0x67,
            0x22, 0x09, 0x42, 0x69, 0x67, 0x35,
        ];

        let str: crate::RbString = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            str.encoding().unwrap().as_string().unwrap().data, // this is a mess lol, i should fix it
            "Big5".as_bytes()
        );
    }

    #[test]
    fn weird_encoding_round_trip() {
        let bytes: &[_] = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x0d, 0x65, 0x6e, 0x63, 0x6f, 0x64, 0x69, 0x6e, 0x67,
            0x22, 0x09, 0x42, 0x69, 0x67, 0x35,
        ];

        let str: crate::RbString = crate::from_bytes(bytes).unwrap();

        let bytes2 = crate::to_bytes(&str).unwrap();

        assert_eq!(bytes, bytes2);
    }
}

#[cfg(test)]
mod floats {
    #[test]
    fn deserialize() {
        let bytes = &[0x04, 0x08, 0x66, 0x07, 0x31, 0x35];

        let float: f64 = crate::from_bytes(bytes).unwrap();

        assert!((float - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn round_trip() {
        let float = 20870.15;

        let bytes = crate::to_bytes(float).unwrap();

        let float2: f64 = crate::from_bytes(&bytes).unwrap();

        assert!((float - float2).abs() < f64::EPSILON);
    }

    #[test]
    fn nan() {
        let bytes = &[0x04, 0x08, 0x66, 0x08, 0x6e, 0x61, 0x6e];

        let float: f64 = crate::from_bytes(bytes).unwrap();

        assert!(float.is_nan());
    }

    #[test]
    fn round_trip_nan() {
        let float = f64::NAN;

        let bytes = crate::to_bytes(float).unwrap();

        let float2: f64 = crate::from_bytes(&bytes).unwrap();

        assert!(float.is_nan());
        assert_eq!(
            bytemuck::cast::<_, u64>(float),
            bytemuck::cast::<_, u64>(float2)
        );
    }
}

#[cfg(test)]
mod arrays {
    #[test]
    fn deserialize() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x69, 0x00, 0x69, 0x06, 0x69, 0x07, 0x69, 0x08, 0x69, 0x09,
        ];

        let ary: Vec<u8> = crate::from_bytes(bytes).unwrap();

        assert_eq!(ary, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn round_trip() {
        let ary = vec!["hi!", "goodbye!", "pain"];

        let bytes = crate::to_bytes(&ary).unwrap();
        let ary2: Vec<&str> = crate::from_bytes(&bytes).unwrap();

        assert_eq!(ary, ary2);
    }
}

#[cfg(test)]
mod structs {
    #[test]
    fn deserialize_borrowed() {
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
        struct Test<'d> {
            field1: bool,
            field2: &'d str,
        }

        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: Test<'_> = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            obj,
            Test {
                field1: true,
                field2: "hello there"
            }
        );
    }

    #[test]
    fn userdata() {
        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        #[serde(from = "crate::Userdata")]
        struct MyUserData {
            field: [char; 4],
        }

        impl From<crate::Userdata> for MyUserData {
            fn from(value: crate::Userdata) -> Self {
                assert_eq!(value.class, "MyUserData");
                let field = std::array::from_fn(|i| value.data[i] as char);
                Self { field }
            }
        }
        let bytes = &[
            0x04, 0x08, 0x75, 0x3a, 0x0f, 0x4d, 0x79, 0x55, 0x73, 0x65, 0x72, 0x44, 0x61, 0x74,
            0x61, 0x09, 0x61, 0x62, 0x63, 0x64,
        ];
        let data: MyUserData = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            data,
            MyUserData {
                field: ['a', 'b', 'c', 'd']
            }
        );
    }
}

#[cfg(test)]
mod misc {
    #[test]
    fn symbol() {
        let sym = crate::Symbol::from("symbol");

        let bytes = crate::to_bytes(&sym).unwrap();

        let sym2: crate::Symbol = crate::from_bytes(&bytes).unwrap();

        assert_eq!(sym, sym2);
    }

    // Testing for zero copy symlink deserialization
    // ALL symbols should be the same reference
    #[test]
    fn symlink() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x3a, 0x09, 0x74, 0x65, 0x73, 0x74, 0x3b, 0x00, 0x3b, 0x00,
            0x3b, 0x00, 0x3b, 0x00,
        ];

        let symbols: Vec<&str> = crate::from_bytes(bytes).unwrap();

        for sym in symbols.windows(2) {
            assert_eq!(sym[0].as_ptr(), sym[1].as_ptr());
        }
    }
}

#[cfg(test)]
mod value_test {
    #[test]
    fn untyped_object() {
        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();
        let obj = obj.into_object().unwrap();

        assert_eq!(obj.class, "Test");
        assert_eq!(obj.fields["field1"], true);
    }

    #[test]
    fn untyped_to_borrowed() {
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
        struct Test<'d> {
            field1: bool,
            field2: &'d str,
        }

        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();

        let test: Test<'_> = serde::Deserialize::deserialize(&obj).unwrap();

        assert_eq!(
            test,
            Test {
                field1: true,
                field2: "hello there"
            }
        );
    }
}
