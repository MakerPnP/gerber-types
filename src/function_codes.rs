//! Function code types.

use crate::{attributes, CoordinateMode, Unit};
use crate::coordinates::{CoordinateOffset, Coordinates};
use crate::errors::GerberResult;
use crate::traits::{GerberCode, PartialGerberCode};
use std::io::Write;

// DCode

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DCode {
    Operation(Operation),
    SelectAperture(i32),
}

impl<W: Write> GerberCode<W> for DCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            DCode::Operation(ref operation) => operation.serialize(writer)?,
            DCode::SelectAperture(code) => writeln!(writer, "D{}*", code)?,
        };
        Ok(())
    }
}

// GCode

#[derive(Debug, Clone, PartialEq)]
pub enum GCode {
    InterpolationMode(InterpolationMode),
    RegionMode(bool),
    QuadrantMode(QuadrantMode),
    Comment(CommentContent),
    /// Deprecated since December 2012, but still in use
    Unit(Unit),
    /// Deprecated since December 2012, but still in use
    CoordinateMode(CoordinateMode),
    /// Deprecated since December 2012, but still in use
    SelectAperture,

}

impl<W: Write> GerberCode<W> for GCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            GCode::InterpolationMode(ref mode) => mode.serialize(writer)?,
            GCode::RegionMode(enabled) => {
                if enabled {
                    writeln!(writer, "G36*")?;
                } else {
                    writeln!(writer, "G37*")?;
                }
            }
            GCode::QuadrantMode(ref mode) => mode.serialize(writer)?,
            GCode::Comment(ref content) => {
                write!(writer, "G04 ")?;
                content.serialize_partial(writer)?;
                writeln!(writer, "*")?;
            }
            GCode::Unit(ref unit) => match unit {
                Unit::Inches => writeln!(writer, "G70*")?,
                Unit::Millimeters => writeln!(writer, "G71*")?,
            },
            GCode::CoordinateMode(ref mode) => match mode {
                CoordinateMode::Absolute => writeln!(writer, "G90*")?,
                CoordinateMode::Incremental => writeln!(writer, "G91*")?,
            },
            GCode::SelectAperture => writeln!(writer, "G54*")?,
        };
        Ok(())
    }
}

/// See Gerber spec 2024.05.
/// 1) 4.1 - Comment (G04)
/// 2) 5.1.1 - Comment attributes
#[derive(Debug, Clone, PartialEq)]
pub enum CommentContent {
    String(String),
    /// "Content starting with ”#@!“ is reserved for standard comments. The purpose of standard
    ///  comments is to add meta-information in a formally defined manner, without affecting image
    ///  generation. They can only be used if defined in this specification"
    Standard(StandardComment),
}

impl<W: Write> PartialGerberCode<W> for CommentContent {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            CommentContent::String(ref string) => {
                write!(writer, "{}", string)?;
            }
            CommentContent::Standard(ref standard) => {
                standard.serialize_partial(writer)?;
            }
        }
        Ok(())
    }
}

/// See Gerber spec 2024.05.
/// 1) 4.1 - Comment (G04)
/// 2) 5.1.1 - Comment attributes
#[derive(Debug, Clone, PartialEq)]
pub enum StandardComment {
    /// TF
    FileAttribute(attributes::FileAttribute),
    /// TO
    ObjectAttribute(attributes::ObjectAttribute),
    /// TA
    ApertureAttribute(attributes::ApertureAttribute),
    /// TD
    DeleteAttribute(String),
}

impl<W: Write> PartialGerberCode<W> for StandardComment {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "#@! ")?;
        match *self {
            StandardComment::FileAttribute(ref fa) => {
                write!(writer, "TF")?;
                fa.serialize_partial(writer)?;
            }
            StandardComment::ObjectAttribute(ref oa) => {
                write!(writer, "TO")?;
                oa.serialize_partial(writer)?;
            }
            StandardComment::ApertureAttribute(ref aa) => {
                write!(writer, "TA")?;
                aa.serialize_partial(writer)?;
            }
            StandardComment::DeleteAttribute(ref content) => {
                write!(writer, "TD")?;
                if !content.is_empty() {
                    write!(writer, ".{}", content)?;
                }
            }
        }
        Ok(())
    }
}

// MCode

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MCode {
    EndOfFile,
}

impl<W: Write> GerberCode<W> for MCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            MCode::EndOfFile => writeln!(writer, "M02*")?,
        };
        Ok(())
    }
}

// Operation

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// D01 Command
    /// `D01 = ['X' integer] ['Y' integer] ['I' integer 'J' integer] 'D01*';`
    Interpolate(Option<Coordinates>, Option<CoordinateOffset>),
    /// D02 Command
    /// `['X' integer] ['Y' integer] 'D02*';`
    Move(Option<Coordinates>),
    /// D03 Command
    /// `['X' integer] ['Y' integer] 'D03*';`
    Flash(Option<Coordinates>),
}

impl<W: Write> GerberCode<W> for Operation {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Operation::Interpolate(ref coords, ref offset) => {
                coords.serialize_partial(writer)?;
                offset.serialize_partial(writer)?;
                writeln!(writer, "D01*")?;
            }
            Operation::Move(ref coords) => {
                coords.serialize_partial(writer)?;
                writeln!(writer, "D02*")?;
            }
            Operation::Flash(ref coords) => {
                coords.serialize_partial(writer)?;
                writeln!(writer, "D03*")?;
            }
        };
        Ok(())
    }
}

// InterpolationMode

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMode {
    Linear,
    ClockwiseCircular,
    CounterclockwiseCircular,
}

impl<W: Write> GerberCode<W> for InterpolationMode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            InterpolationMode::Linear => writeln!(writer, "G01*")?,
            InterpolationMode::ClockwiseCircular => writeln!(writer, "G02*")?,
            InterpolationMode::CounterclockwiseCircular => writeln!(writer, "G03*")?,
        };
        Ok(())
    }
}

// QuadrantMode

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadrantMode {
    Single,
    Multi,
}

impl<W: Write> GerberCode<W> for QuadrantMode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            QuadrantMode::Single => writeln!(writer, "G74*")?,
            QuadrantMode::Multi => writeln!(writer, "G75*")?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {}
