# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

### v0.4.0 (2025-06-10) - (unreleased)

- [added] Support for macro expressions.  See `MacroDecimal`.
- [added] `MacroBoolean`.
- [added] `MacroInteger`.
- [added] Serialization of `ApertureAttribute` (previously it would panic on serializing them).
- [added] Serialization of `FileAttribute` (previously it would panic on serializing them).
- [changed] Replaced `Aperture::Other` with `Aperture::Macro`, the latter having option arguments.
- [changed] Use `MacroBoolean` instead of `bool` for `exposure` on macro primitives.
- [changed] `PolygonPrimitive::new` uses a `MacroInteger` for the `vertices` argument.
- [changed] Replaced dependency on `conv::TryFrom` with `std::convert::TryFrom` (since TryFrom was stabilized in Rust 1.34).
- [changed] Improved the API of `Coordinates`/`CoordinateOffsets` so that it's easier to build partial coordinates by 
  using a trait to convert from integer types or optional integer types.
- [changed] `Scoring` replaced with `VCut` to match spec.
- [changed] Various other changes to `ApertureAttribute`, `FileFunction` and `FileAttribute` to match spec.
- [deprecated] `*Primitive::exposure_on`, replaced with `*Primitive::with_exposure`.

### v0.3.0 (2022-07-05)

- [fixed] Fix whitespace in G04 comment serialization (#33)
- [changed] Updated dependencies
- [changed] A fixed MSRV was dropped

Thanks @NemoAndrea for contributions!

### v0.2.0 (2021-01-06)

This release requires at least Rust 1.31 (2018 edition).

- [added] Implement constructors for `Circle` and `Rectangular`
- [added] Derive Clone for all structs and enums (#16)
- [added] Derive PartialEq and Eq where possible
- [added] Implement `From<FunctionCode>` and `From<ExtendedCode>` for Command
- [added] Impl `From<>` for Command, FunctionCode and ExtendedCode
- [added] New builder-style constructors (#22)
- [added] Support for more FileFunction and FileAttribute variants (#26, #30)
- [changed] Derive Copy for some trivial enums
- [changed] Create new internal `PartialGerberCode` trait (#18)
- [changed] Split up code into more modules
- [changed] Upgraded all dependencies
- [changed] Require Rust 1.31+

Thanks @connorkuehl and @twitchyliquid64 for contributions!

### v0.1.1 (2017-06-10)

- First crates.io release
