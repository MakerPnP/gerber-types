//! Function code types.

use std::io::Write;

use crate::coordinates::{CoordinateOffset, Coordinates};
use crate::errors::GerberResult;
use crate::traits::{GerberCode, PartialGerberCode};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GCode {
    InterpolationMode(InterpolationMode),
    RegionMode(bool),
    QuadrantMode(QuadrantMode),
    Comment(String),
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
            GCode::Comment(ref comment) => writeln!(writer, "G04 {}*", comment)?,
        };
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
