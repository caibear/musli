# musli-storage

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-storage.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-storage)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--storage-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-storage)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Super simple storage encoding for [Müsli]

The storage encoding is partially upgrade safe:

* ✔ Can tolerate missing fields if they are annotated with
  `#[musli(default)]`.
* ✗ Cannot skip over extra unrecognized fields.

This means that it's suitable as a storage format, since the data model only
evolves in one place. But unsuitable as a wire format since it cannot allow
clients to upgrade independent of each other.

See [musli-wire] for a fully upgrade safe format.

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

let version2 = musli_storage::to_vec(&Version2 {
    name: String::from("Aristotle"),
    age: Some(62),
})?;

assert!(musli_storage::decode::<_, Version1>(version2.as_slice()).is_err());

let version1 = musli_storage::to_vec(&Version1 {
    name: String::from("Aristotle"),
})?;

let version2: Version2 = musli_storage::decode(version1.as_slice())?;

assert_eq!(version2, Version2 {
    name: String::from("Aristotle"),
    age: None,
});
```

<br>

## Configuring

To tweak the behavior of the storage format you can use the
[Encoding] type:

```rust
use musli_storage::Encoding;
use musli_storage::int::{Fixed, Variable};
use musli::mode::DefaultMode;
use musli::{Encode, Decode};

const CONFIG: Encoding<DefaultMode, Fixed, Variable> = Encoding::new()
    .with_fixed_integers();

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct<'a> {
    name: &'a str,
    age: u32,
}

let mut out = Vec::new();

let expected = Struct {
    name: "Aristotle",
    age: 61,
};

CONFIG.encode(&mut out, &expected)?;
let actual = CONFIG.decode(&out[..])?;

assert_eq!(expected, actual);
```

[default encoding format]: https://docs.rs/musli-storage/latest/musli-storage/struct.Encoding.html
[musli-wire]: https://docs.rs/musli-wire
[Müsli]: https://docs.rs/musli
[Encoding]: https://docs.rs/musli-storage/latest/musli-storage/struct.Encoding.html
