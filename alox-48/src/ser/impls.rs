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
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    ffi::{CStr, CString},
    hash::Hash,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
        AtomicU64, AtomicU8, AtomicUsize, Ordering,
    },
};

use super::{Error, Kind, Result, Serialize, SerializeArray, SerializerTrait};

// some of these macros are lifted directly from serde.
// serde is under a fairly permissive license (and any macro i would write would likely look identical) so this is okay.
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

primitive_int_impl! {
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}

impl Serialize for f32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_f64(f64::from(*self))
    }
}

impl Serialize for f64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_f64(*self)
    }
}

impl Serialize for bool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_bool(*self)
    }
}

impl Serialize for char {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_rust_string(self.encode_utf8(&mut [0; 4]))
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

impl Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_rust_string(self)
    }
}

impl Serialize for CStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_string(self.to_bytes())
    }
}

impl Serialize for CString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        serializer.serialize_string(self.to_bytes())
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        match self {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_nil(),
        }
    }
}

macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> Serialize for ($($name,)+)
            where
                $($name: Serialize,)+
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
                where
                    S: SerializerTrait,
                {
                    let mut array = serializer.serialize_array($len)?;
                    $(
                        array.serialize_element(&self.$n)?;
                    )+
                    array.end()
                }
            }
        )+
    }
}

// tuple pyramid! rust has no variadic generics so this is the best we can do :(
tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

macro_rules! array_impl {
    (
        $(#[$attr:meta])*
        <$($desc:tt)+
    ) => {
        $(#[$attr])*
        impl <$($desc)+
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
            where
                S: SerializerTrait,
            {
                serializer.collect_array(self)
            }
        }
    };
}

array_impl!(<T> Serialize for [T] where T: Serialize);

// we can actually oneup serde here!
// serde won't/can't bump rust versions to when min const generics were stabilized so only a subset of arrays have serialize/deserialize impls.
// not us though!
array_impl!(<T, const SIZE: usize> Serialize for [T; SIZE] where T: Serialize);

array_impl!(<T> Serialize for Vec<T> where T: Serialize);

array_impl!(<T> Serialize for BinaryHeap<T> where T: Serialize + Ord);

array_impl!(<T> Serialize for BTreeSet<T> where T: Serialize + Ord);

array_impl!(<T> Serialize for HashSet<T> where T: Serialize + Hash);

array_impl!(<T> Serialize for VecDeque<T> where T: Serialize);

macro_rules! map_impl {
    (
        $(#[$attr:meta])*
        <$($desc:tt)+
    ) => {
        $(#[$attr])*
        impl <$($desc)+
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
            where
                S: SerializerTrait,
            {
                serializer.collect_hash(self)
            }
        }
    }
}

map_impl!(<K, V> Serialize for BTreeMap<K, V> where K: Ord + Serialize, V: Serialize);

map_impl!(<K, V> Serialize for HashMap<K, V> where K: Hash + Serialize, V: Serialize);

macro_rules! deref_impl {
    (
        $(#[$attr:meta])*
        <$($desc:tt)+
    ) => {
        $(#[$attr])*
        impl <$($desc)+ {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
            where
                S: SerializerTrait,
            {
                (**self).serialize(serializer)
            }
        }
    };
}

deref_impl! {
    <'a, T: ?Sized> Serialize for &'a T where T: Serialize
}

deref_impl! {
    <'a, T: ?Sized> Serialize for &'a mut T where T: Serialize
}

deref_impl! {
    <'a, T: ?Sized> Serialize for std::borrow::Cow<'a, T> where T: Serialize + ToOwned
}

deref_impl! {
    <T: ?Sized> Serialize for Box<T> where T: Serialize
}

deref_impl! {
    <T: ?Sized> Serialize for std::rc::Rc<T> where T: Serialize
}

deref_impl! {
    <T: ?Sized> Serialize for std::sync::Arc<T> where T: Serialize
}

impl<T: ?Sized> Serialize for std::rc::Weak<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        self.upgrade().serialize(serializer)
    }
}

impl<T: ?Sized> Serialize for std::sync::Weak<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        self.upgrade().serialize(serializer)
    }
}

macro_rules! nonzero_integers {
    ($($T:ident,)+) => {
        $(
            impl Serialize for $T {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
                where
                    S: SerializerTrait,
                {
                    self.get().serialize(serializer)
                }
            }
        )+
    }
}

nonzero_integers! {
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
}

impl<T: ?Sized + Copy> Serialize for Cell<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        self.get().serialize(serializer)
    }
}

impl<T: ?Sized> Serialize for RefCell<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
    where
        S: SerializerTrait,
    {
        match self.try_borrow() {
            Ok(v) => v.serialize(serializer),
            Err(_) => Err(Error {
                kind: Kind::Message("already mutably borrowed".to_string()),
            }),
        }
    }
}

macro_rules! atomic_impl {
    ($($ty:ident $size:expr)*) => {
        $(
            #[cfg(any(no_target_has_atomic, target_has_atomic = $size))]
            #[cfg_attr(doc_cfg, doc(cfg(target_has_atomic = $size)))]
            impl Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok>
                where
                    S: SerializerTrait,
                {
                    // Matches the atomic ordering used in libcore for the Debug impl
                    self.load(Ordering::Relaxed).serialize(serializer)
                }
            }
        )*
    }
}

atomic_impl! {
    AtomicBool "8"
    AtomicI8 "8"
    AtomicI16 "16"
    AtomicI32 "32"
    AtomicI64 "64"
    AtomicIsize "ptr"
    AtomicU8 "8"
    AtomicU16 "16"
    AtomicU32 "32"
    AtomicU64 "64"
    AtomicUsize "ptr"
}
