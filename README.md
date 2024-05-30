# alox-48

alox-48 (short for aluminum oxide 48) is a crate for deserializing and serializing as well Ruby's Marshal data format, using a custom data format like serde.

alox-48 intends to provide almost perfect round-trip deserialization, with some exceptions:

- Object links are not preserved.
    Object links are a way for Ruby to compact data in Marshal. They rely heavily on Ruby having a GC and thus do not map well to Rust.
- Bignum is unsupported.

# Why a custom data format

Originally this crate relied on nightly to extend serde, using `min_speciailization`.
Unfortunately that had many shortcomings and the deserializer would frequently choke on valid inputs and the serializer would spit out invalid data.

Most issues revolved around symbols- ruby uses `@` prefixed symbols for instance variables, but also accepts variables *without* the prefix, silently discarding them.

I'm working on a separate serde adapter that can interface serde's data format with alox's, but that looks like it'll be nightly only.