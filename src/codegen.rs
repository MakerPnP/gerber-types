//! Generic code generation, e.g. implementations of `PartialGerberCode` for
//! bool or Vec<G: GerberCode>.

use std::io::Write;

use crate::errors::GerberResult;
use crate::traits::{GerberCode, PartialGerberCode};
use crate::types::*;

/// Implement `PartialGerberCode` for booleans
impl<W: Write> PartialGerberCode<W> for bool {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if *self {
            write!(writer, "1")?;
        } else {
            write!(writer, "0")?;
        };
        Ok(())
    }
}

/// Implement `GerberCode` for Vectors of types that are `GerberCode`.
impl<W: Write, G: GerberCode<W>> GerberCode<W> for Vec<G> {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        for item in self.iter() {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

/// Implement `PartialGerberCode` for `Option<T: PartialGerberCode>`
impl<T: PartialGerberCode<W>, W: Write> PartialGerberCode<W> for Option<T> {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if let Some(ref val) = *self {
            val.serialize_partial(writer)?;
        }
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for Command {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Command::FunctionCode(ref code) => code.serialize(writer)?,
            Command::ExtendedCode(ref code) => code.serialize(writer)?,
        };
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for FunctionCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            FunctionCode::DCode(ref code) => code.serialize(writer)?,
            FunctionCode::GCode(ref code) => code.serialize(writer)?,
            FunctionCode::MCode(ref code) => code.serialize(writer)?,
        };
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for ExtendedCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            ExtendedCode::CoordinateFormat(ref cf) => {
                writeln!(writer, "%FSLAX{0}{1}Y{0}{1}*%", cf.integer, cf.decimal)?;
            }
            ExtendedCode::Unit(ref unit) => {
                write!(writer, "%MO")?;
                unit.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureDefinition(ref def) => {
                write!(writer, "%ADD")?;
                def.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureMacro(ref am) => {
                write!(writer, "%")?;
                am.serialize_partial(writer)?;
                writeln!(writer, "%")?;
            }
            ExtendedCode::LoadPolarity(ref polarity) => {
                write!(writer, "%LP")?;
                polarity.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadMirroring(ref mirroring) => {
                write!(writer, "%LM")?;
                mirroring.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadRotation(ref rotation) => {
                write!(writer, "%LR")?;
                rotation.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadScaling(ref scaling) => {
                write!(writer, "%LS")?;
                scaling.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::StepAndRepeat(ref sar) => {
                write!(writer, "%SR")?;
                sar.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::FileAttribute(ref attr) => {
                write!(writer, "%TF")?;
                attr.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::DeleteAttribute(ref attr) => {
                writeln!(writer, "%TD{}*%", attr)?;
            }
            ExtendedCode::ApertureBlock(ref ab) => {
                write!(writer, "%AB")?;
                ab.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureAttribute(ref aa) => {
                write!(writer, "%TA")?;
                aa.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ObjectAttribute(ref oa) => {
                write!(writer, "%TO")?;
                oa.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
        };
        Ok(())
    }
}
