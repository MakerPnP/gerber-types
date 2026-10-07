//! Traits used in gerber-types.

use std::io::Write;

use crate::GerberResult;

/// All types that implement this trait can be converted to a complete Gerber
/// Code line. Generated code should end with a newline.
///
/// String values are emitted as printable ASCII using Gerber Unicode escapes
/// (`\uXXXX` or `\UXXXXXXXX`). Reserved characters and literal backslashes
/// are escaped; commas within attribute fields are escaped as well.
pub trait GerberCode<W: Write> {
    fn serialize(&self, writer: &mut W) -> GerberResult<()>;
}

/// All types that implement this trait can be converted to a Gerber Code
/// representation.
///
/// This is a crate-internal trait.
pub trait PartialGerberCode<W: Write> {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()>;
}
