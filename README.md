# alox-48

alox-48 (short for aluminum oxide 48) is a crate for deserializing (soon serializing as well) Ruby's Marshal data format, using serde.
It requires rust nightly as of 2/8/23 for `min-specialization`.

alox-48 intends to provide almost perfect round-trip deserialization, with some exceptions:
 - Object links are not preserved.
   Object links are a way for ruby to compact data in Marshal. They rely heavily on RUby having a GC and thus do not map well to Rust.
 - Classes and Modules are unsupported.
 - Bignum is unsupported.

# Why min-specialization

alox-48 uses `min-specialization` to extend serde in order to preserve types that may be lost in deserialization.
There are a lot of data types in Marshal that do not map well to serde's data model, or rust's data model.

This includes:
 - Symbols
 - Userdata
 - Objects
 - Strings with non-utf8 encoding

It does this via the VistorExt trait:
```rs
pub trait VisitorExt<'de>: Visitor<'de> {
    fn visit_userdata<E>(self, class: &'de str, data: &'de [u8]) -> Result<Self::Value, E>
    where
        E: SerdeError;

    fn visit_object<A>(self, class: &'de str, fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>;

    fn visit_symbol<E>(self, sym: &'de str) -> Result<Self::Value, E>
    where
        E: SerdeError;

    fn visit_ruby_string<A>(self, str: &'de [u8], fields: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>;
}
```
You are free to implement this trait on any of your types. It'll only work with the deserializer from this crate, though.

# Value

alox-48 provides a `Value` enum to work with untyped data.
```rs
pub enum Value {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(RbString),
    Symbol(Symbol),
    Array(RbArray),
    Hash(RbHash),
    Userdata(Userdata),
    Object(Object),
}
```

`Value` relies *heavily* on `VistorExt` to make sure types stay the same and data is not lost in deserialization.