# Serde MTProto

[![Latest Version]][crates.io]
[![Latest Docs]][docs.rs]
![Supported Rust Versions]
![License]
[![Travis Build Status]][travis]

[Latest Version]: https://img.shields.io/crates/v/serde\_mtproto.svg
[crates.io]: https://crates.io/crates/serde\_mtproto
[Latest Docs]: https://docs.rs/serde_mtproto/badge.svg
[docs.rs]: https://docs.rs/serde\_mtproto
[Supported Rust Versions]: https://img.shields.io/badge/rustc-1.20+-red.svg
[License]: https://img.shields.io/crates/l/serde\_mtproto.svg
[Travis Build Status]: https://api.travis-ci.org/hcpl/serde\_mtproto.svg?branch=master
[travis]: https://travis-ci.org/hcpl/serde\_mtproto

[MTProto](https://core.telegram.org/mtproto) [de]serialization for Rust which
utilizes [Serde](https://serde.rs) framework.

```toml,no_sync
[dependencies]
serde_mtproto = { git = "https://github.com/hcpl/serde_mtproto" }
```

You may be looking for:

- [Serde MTProto API documentation (latest published)](https://docs.rs/serde_mtproto/)
- [Serde MTProto API documentation (master)](https://hcpl.github.io/serde_mtproto/master/)
- [Serde API documentation](https://docs.rs/serde/)
- [Detailed documentation about Serde](https://serde.rs/)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/codegen.html)

Supports Rust 1.20 and newer.
Older versions may work, but are not guaranteed to.

### Optional Cargo features

- **`extprim`** — `MtProtoSized` implementations for `extprim::i128::i128` and
  `extprim::u128::u128`.
  Works on Rust 1.20+.
- **`quickcheck`** — `quickcheck::Arbitrary` implmentations for several types
  defined in `serde_mtproto`.
  For now, those only include wrapper types `Boxed`, `WithSize`.
  Works on Rust 1.20+.

## Changelog

Maintained in [CHANGELOG.md](CHANGELOG.md).


## License

Serde MTProto is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde MTProto by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
