# Rust Gerber Library
[![Build status][build-status-badge]][build-status]
[![Crates.io][crates-io-badge]][crates-io]
[![MakerPnP Discord][discord-badge]][discord]

- [Docs (released)](https://docs.rs/gerber-types/)

This crate implements the basic building blocks of Gerber X2 (compatible with Gerber RS-274X) code. It focuses on the
low-level types (to be used like an AST) and code generation and does not do any semantic checking.

For example, you can use an aperture without defining it. This will generate syntactically valid but semantically
invalid Gerber code, but this module won't complain.

Current Gerber X2 spec: https://www.ucamco.com/files/downloads/file/81/the_gerber_file_format_specification.pdf

## Example

You can find an example in the [`examples` directory](https://github.com/MakerPnP/gerber-types/blob/main/examples/polarities-apertures.rs).

This library has a low-level focus and is quite verbose.  Other external crates can provide a high-level API.

To generate Gerber code for that example:

    $ cargo run --example polarities-apertures

## Related crates

### Gerber Parser

A rust crate for parsing gerber files that uses this crate as a dependency:

Crates.io: https://crates.io/crates/gerber-parser
Github: https://github.com/MakerPnP/gerber-parser

### Gerber Viewer

A rust crate for rendering gerber layers, also uses this crate as a dependency:

Github: https://github.com/MakerPnP/gerber-viewer

## Related projects

For a list of other projects that use this crate you can check the github 'dependents' page.

https://github.com/MakerPnP/gerber-types/network/dependents


## Authors

* Danilo Bargen - Original author.
* Dominic Clifton - Current maintainer (2025/05/05 onwards).

See the [contributors](https://github.com/MakerPnP/gerber-types/graphs/contributors) page on github for full list.

## License

Licensed under _either_ of the following licenses, at your option. 

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

<!-- Badges -->

[build-status]: https://github.com/makerpnp/gerber-types/actions/workflows/ci.yml
[build-status-badge]: https://github.com/makerpnp/gerber-types/workflows/CI/badge.svg
[crates-io]: https://crates.io/crates/gerber-types
[crates-io-badge]: https://img.shields.io/crates/v/gerber-types.svg
[discord]: https://discord.gg/ffwj5rKZuf
[discord-badge]: https://img.shields.io/discord/1255867192503832688?label=MakerPnP%20discord&color=%2332c955 
