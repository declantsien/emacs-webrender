# lazy-bytes-cast

[![Build](https://github.com/DoumanAsh/lazy-bytes-cast/workflows/Rust/badge.svg)](https://github.com/DoumanAsh/lazy-bytes-cast/actions?query=workflow%3ARust)
[![Crates.io](https://img.shields.io/crates/v/lazy-bytes-cast.svg)](https://crates.io/crates/lazy-bytes-cast)
[![Docs.rs](https://docs.rs/lazy-bytes-cast/badge.svg)](https://docs.rs/crate/lazy-bytes-cast/)

This crate provides simple methods to cast from and into byte arrays.

# Example

```rust
use lazy_bytes_cast::{FromByteArray, IntoByteArray, AsByteSlice, FromByteSlice};

let val = 9999999u32;
let bytes = [127u8, 150, 152, 0];
assert_eq!(val.as_slice(), bytes);
assert_eq!(val.into_byte_array(), bytes);

assert_eq!(u32::from_slice(&bytes).unwrap(), val);
assert_eq!(u32::from_byte_array(bytes), val);
```
