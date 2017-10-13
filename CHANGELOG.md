# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).


## [Unreleased]

### Added

- Type aliases for `Boxed` (`WithId`) and `BoxedWithSize` (`WithIdAndSize`).
- `WithSize` wrapper type which attaches size hint of a `T: MtProtoSized` type.
- `BoxedWithSize` wrapper type which attaches id and size hint of a `T: Identifiable + MtProtoSized` type.
- `MtProtoSized` implementation for arrays up to length 32 and `Box<T>` where `T: MtProtoSized`.
- Size hints for `de::SeqAccess` and `de::MapAccess`.
- `UnsizedByteBuf` helper type and tests for it.
- `unsized_bytes_pad_to_bytes` and `unsized_bytes_pad_to_writer` convenience serialization functions that write bytes sequence without its length and also pads the sequence so that the length be divisible by 16.
- `from_bytes_reuse` and `from_reader_reuse` which return the derialized value coupled with the bytes reference/reader at the point where deserialization stopped respectively. This allows to use the leftover data afterwards.
- `impl MtProtoSized` for `extprim::i128::u128`, `extprim::u128::u128` and tuples up to arity 12 (like standard library does)
- Benchmarks for primitives, strings, custom types and `extprim` 128-bit types.
- Doctests.
- `ErrorKind::FloatCast`
- `Serializer::into_writer`
- `Deserializer::into_reader`
- `Deserializer::remaining_bytes`
- `Deserializer::remaining_length`

### Changed

- `Identifiable::type_id()` now returns `u32` instead of `i32`.
- Move `boxed` module to `wrappers`.
- Use shorter method names in `Identifiable` and `MtProtoSized` traits.
- Documentation covers all public items as enforced by `#[deny(missing_docs)]`
- Make dependency on `extprim` an optional feature.
- Now uses `error_chain` 0.11 (can be incompatible with other versions).

### Removed

- `helpers` module along with `Bytes` and `ByteBuf` types. These are now provided by `serde_bytes` crate and we reexport them for convenience.

### Fixed

- Size prediction for strings and byte sequences.
- A bug in `#[derive(MtProtoSized)]`
- Size prediction for 2-tuples.
- `ErrorKind::IntegerCast` now holds an `u64` value which failed to cast.
- Float deserialization: both `f32` and `f64` must be [de]serialized as `f64`.


## [0.3.1] - 2017-08-12

### Added

- `Bytes` and `ByteBuf` helper types to [de]serialize `&[u8]` and `Vec<u8>` using the `bytes` Serde data type instead of `seq`. Can be removed when specialization feature arrives to stable Rust.
- Size prediction via `MtProtoSized` trait with respective `#[derive(MtProtoSized)]`
- Documentation.
- Logging via `log` crate.
- `SerErrorKind::NotEnoughElements`
- `ErrorKind::SeqTooLong`
- `ErrorKind::ByteSeqTooLong`

### Changed

- Add checks to `SerializerFixedLengthSeq` implementations of `end` methods from `Serialize*` traits.

### Fixed

- Panics when serializing strings/byte sequences.


## [0.3.0] - 2017-08-09

### Added

- Replace `Wrapper` by `Boxed` which is actually meant for public use (`Wrapper` being public was an accident).
- Handle the edge case in bytes/string serialization when byte length is >= 2^24 == 16 MB because the serialized representation of bytes/string cannot hold more than 3 bytes of length info.
- Better error messages.
- Implement `Clone`, `Debug`, `Eq`, `Hash` and `PartialEq` for `Boxed`.
- Implement `Identifiable` for primitives, strings and `Vec<T>`.
- Basic documentation.

### Changed

- Rename `ser::SerializeMap` to `ser::SerializeFixedLengthMap` to reflect its nature.

### Removed

- `from_*_identifiable` and `to_*_identifiable` functions. Instead use `from_*` and `to_*` combined with wrapping data in `Boxed`.
- `error::DeErrorKind::ExpectedBool`
- `error::DeErrorKind::InvalidFirstByte255`


## [0.2.0] - 2017-08-08

### Added

- [De]serialization for `map` Serde data type.
- `error::SerErrorKind`
- `error::DeErrorKind`
- `error::SerSerdeType`
- `error::DeSerdeType`

### Changed

- Replace panics via `unreachable!()` or `unimplemented!()` by explicit error return values.
- Delegate the serialization of complex Serde data types (`seq`, `tuple`, `tuple_struct`, `tuple_variant`, `struct`, `struct_variant`) to the dedicated `ser::SerializedFixedLengthSeq` type.
- Use type-based errors instead of string-based ones.

### Removed

- `error::IntegerOverflowingCast`

### Fixed

- Handle `serde_mtproto_derive` panic gracefully.
- Properly support [de]serialization for `seq` Serde data type (only for those with length known ahead-of-time).
- Mitigate potential panics in lossy integer castings by returning errors in those cases.


## [0.1.0] - 2017-08-07

### Added

- [De]serialization data format for primitive and some complex Serde data types.
- `Identifiable` trait for types that have an id in MtProto type system.
- `from_bytes`, `from_reader`, `to_bytes` and `to_reader` convenience functions along with their `*_identifiable` counterparts.
- `#[derive(MtProtoIdentifiable)]` for structs and enums.


[Unreleased]: https://github.com/hcpl/serde_mtproto/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/hcpl/serde_mtproto/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/hcpl/serde_mtproto/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hcpl/serde_mtproto/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hcpl/serde_mtproto/tree/v0.1.0
