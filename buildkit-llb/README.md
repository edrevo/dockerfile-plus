`buildkit-llb` - high-level API to create BuildKit LLB graphs
=======

[![Actions Status]][Actions Link]
[![buildkit-llb Crates Badge]][buildkit-llb Crates Link]
[![buildkit-llb Docs Badge]][buildkit-llb Docs Link]

# Usage

Please check [docs][buildkit-llb Docs Link] or examples on how to use the crate.

The LLB graph from stdout can easily be used with `buildctl`:
```
cargo run --example=scratch | buildctl build
```

# License

`buildkit-llb` is primarily distributed under the terms of both the MIT license and
the Apache License (Version 2.0), with portions covered by various BSD-like
licenses.

See LICENSE-APACHE, and LICENSE-MIT for details.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `buildkit-llb` by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

[Actions Link]: https://github.com/denzp/rust-buildkit/actions
[Actions Status]: https://github.com/denzp/rust-buildkit/workflows/CI/badge.svg
[buildkit-llb Docs Badge]: https://docs.rs/buildkit-llb/badge.svg
[buildkit-llb Docs Link]: https://docs.rs/buildkit-llb/
[buildkit-llb Crates Badge]: https://img.shields.io/crates/v/buildkit-llb.svg
[buildkit-llb Crates Link]: https://crates.io/crates/buildkit-llb
