# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

### v0.6.0 (2025-07-10)

- [added] Added support for G04 'standard comments' where comment attributes are placed in G04 commands.
  e.g. 'G04 #@! TA.AperFunction,SMDPad,CuDef*'
  This means that when you're looking for attributes, you now have to look in two places:
  1) `Command::ExtendedCode(ExtendedCode::[FileAttribute|ObjectAttribute|ApertureAttribute])` and 
  2) `Command::FunctionCode(FunctionCode::GCode(GCode::Comment(CommentContent::Standard(StandardComment::[FileAttribute|ObjectAttribute|ApertureAttribute]))))`
  It also means when you're making/serializing gerber files you need to choose where to put the attributes.
  Refer to Gerber spec 2024.05 - "4.1 Comment (G04)" and "5.1.1 Comment attributes".
  Unfortunately, in 2025, manufacturing files containing comment attributes are still widespread.
- [changed] Removed 'Eq' from `FunctionCode`, due to use of `f64` in attributes (`ExtendedCode` wasn't `Eq` either)

### v0.5.0 (2025-07-10)

- [added] Support for legacy/deprecated gerber commands: `IP`, `MI`, `SF`, `OF`, `IR`, and `AS`.

### v0.4.0 (2025-06-30)

- [added] Support for macro expressions.  See `MacroDecimal`.
- [added] `MacroBoolean`.
- [added] `MacroInteger`.
- [added] `ApertureBlock`.
- [added] Serialization of `FileAttribute` (previously it would panic on serializing them).
- [added] Serialization of `ApertureAttribute` (previously it would panic on serializing them).
- [added] `VariantNames` and `VariantArray` support for simple enums, via 'strum' crate.
- [changed] Replaced `Aperture::Other` with `Aperture::Macro`, the latter having option arguments.
- [changed] Use `MacroBoolean` instead of `bool` for `exposure` on macro primitives.
- [changed] `PolygonPrimitive::new` uses a `MacroInteger` for the `vertices` argument.
- [changed] Replaced dependency on `conv::TryFrom` with `std::convert::TryFrom` (since TryFrom was stabilized in Rust 1.34).
- [changed] Improved the API of `Coordinates`/`CoordinateOffsets` so that it's easier to build partial coordinates by 
  using a trait to convert from integer types or optional integer types.
- [changed] `Scoring` replaced with `VCut` to match spec.
- [changed] Various other changes to `ApertureAttribute`, `FileFunction` and `FileAttribute` to match spec.
- [changed] Bumped MSRV to 1.661 (due to use of 'strum').
- [changed] All coordinates in `Operation` are now Optional. See https://github.com/MakerPnP/gerber-types/pull/47
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
