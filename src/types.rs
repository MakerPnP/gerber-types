//! Types for Gerber code generation.
//!
//! All types are stateless, meaning that they contain all information in order
//! to render themselves. This means for example that each `Coordinates`
//! instance contains a reference to the coordinate format to be used.

use crate::attributes;
use crate::coordinates;
use crate::extended_codes;
use crate::function_codes;
use crate::macros;
use chrono::{DateTime, FixedOffset};
use std::convert::From;

// Helper macros

macro_rules! impl_from {
    ($from:ty, $target:ty, $variant:expr) => {
        impl From<$from> for $target {
            fn from(val: $from) -> Self {
                $variant(val)
            }
        }
    };
}

// Root type

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    FunctionCode(FunctionCode),
    ExtendedCode(ExtendedCode),
}

impl_from!(FunctionCode, Command, Command::FunctionCode);
impl_from!(ExtendedCode, Command, Command::ExtendedCode);

macro_rules! impl_command_fromfrom {
    ($from:ty, $inner:path) => {
        impl From<$from> for Command {
            fn from(val: $from) -> Self {
                Command::from($inner(val))
            }
        }
    };
}

// Main categories

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCode {
    DCode(function_codes::DCode),
    GCode(function_codes::GCode),
    MCode(function_codes::MCode),
}

impl_from!(function_codes::DCode, FunctionCode, FunctionCode::DCode);
impl_from!(function_codes::GCode, FunctionCode, FunctionCode::GCode);
impl_from!(function_codes::MCode, FunctionCode, FunctionCode::MCode);

impl_command_fromfrom!(function_codes::DCode, FunctionCode::from);
impl_command_fromfrom!(function_codes::GCode, FunctionCode::from);
impl_command_fromfrom!(function_codes::MCode, FunctionCode::from);

#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedCode {
    /// FS
    CoordinateFormat(coordinates::CoordinateFormat),
    /// MO
    Unit(extended_codes::Unit),
    /// AD
    ApertureDefinition(extended_codes::ApertureDefinition),
    /// AM
    ApertureMacro(macros::ApertureMacro),
    /// LP
    LoadPolarity(extended_codes::Polarity),
    /// LM
    LoadMirroring(extended_codes::Mirroring),
    /// LR
    LoadRotation(extended_codes::Rotation),
    /// LS
    LoadScaling(extended_codes::Scaling),
    /// SR
    StepAndRepeat(extended_codes::StepAndRepeat),
    /// AB
    ApertureBlock(extended_codes::ApertureBlock),
    /// TF
    FileAttribute(attributes::FileAttribute),
    /// TO
    ObjectAttribute(attributes::ObjectAttribute),
    /// TA
    ApertureAttribute(attributes::ApertureAttribute),
    /// TD
    DeleteAttribute(attributes::AttributeDeletionCriterion),
    /// MI (deprecated in Gerber spec since December 2012)
    MirrorImage(extended_codes::ImageMirroring),
    /// OF (deprecated in Gerber spec since December 2012)
    OffsetImage(extended_codes::ImageOffset),
    /// SF (deprecated in Gerber spec since December 2012)
    ScaleImage(extended_codes::ImageScaling),
    /// IR (deprecated in Gerber spec since December 2012)
    RotateImage(extended_codes::ImageRotation),
    /// IP (deprecated in gerber spec since December 2012)
    ImagePolarity(extended_codes::ImagePolarity),
    /// AS (deprecated in gerber spec since December 2012)
    AxisSelect(extended_codes::AxisSelect),
    /// IN (deprecated in gerber spec since October 2013)
    ImageName(extended_codes::ImageName),
}

impl_from!(
    coordinates::CoordinateFormat,
    ExtendedCode,
    ExtendedCode::CoordinateFormat
);
impl_from!(extended_codes::Unit, ExtendedCode, ExtendedCode::Unit);
impl_from!(
    extended_codes::ApertureDefinition,
    ExtendedCode,
    ExtendedCode::ApertureDefinition
);
impl_from!(
    macros::ApertureMacro,
    ExtendedCode,
    ExtendedCode::ApertureMacro
);
impl_from!(
    extended_codes::Polarity,
    ExtendedCode,
    ExtendedCode::LoadPolarity
);
impl_from!(
    extended_codes::Mirroring,
    ExtendedCode,
    ExtendedCode::LoadMirroring
);
impl_from!(
    extended_codes::Rotation,
    ExtendedCode,
    ExtendedCode::LoadRotation
);
impl_from!(
    extended_codes::Scaling,
    ExtendedCode,
    ExtendedCode::LoadScaling
);
impl_from!(
    extended_codes::StepAndRepeat,
    ExtendedCode,
    ExtendedCode::StepAndRepeat
);
impl_from!(
    extended_codes::ApertureBlock,
    ExtendedCode,
    ExtendedCode::ApertureBlock
);
impl_from!(
    attributes::FileAttribute,
    ExtendedCode,
    ExtendedCode::FileAttribute
);
impl_from!(
    attributes::ApertureAttribute,
    ExtendedCode,
    ExtendedCode::ApertureAttribute
);

impl_command_fromfrom!(coordinates::CoordinateFormat, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::Unit, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::ApertureDefinition, ExtendedCode::from);
impl_command_fromfrom!(macros::ApertureMacro, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::Polarity, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::Scaling, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::Mirroring, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::Rotation, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::StepAndRepeat, ExtendedCode::from);
impl_command_fromfrom!(extended_codes::ApertureBlock, ExtendedCode::from);
impl_command_fromfrom!(attributes::FileAttribute, ExtendedCode::from);
impl_command_fromfrom!(attributes::ApertureAttribute, ExtendedCode::from);

#[cfg(test)]
mod test {
    use super::*;

    use std::io::BufWriter;

    use crate::extended_codes::Polarity;
    use crate::function_codes::GCode;
    use crate::traits::GerberCode;
    use crate::{
        ApertureBlock, AttributeDeletionCriterion, CommentContent, Mirroring, Rotation, Scaling,
        StepAndRepeat,
    };

    #[test]
    fn test_debug() {
        //! The debug representation should work properly.
        let c = Command::FunctionCode(FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "test".to_string(),
        ))));
        let debug = format!("{:?}", c);
        assert_eq!(debug, "FunctionCode(GCode(Comment(String(\"test\"))))");
    }

    #[test]
    fn test_function_code_serialize() {
        //! A `FunctionCode` should implement `GerberCode`
        let c = FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "comment".to_string(),
        )));
        assert_code!(c, "G04 comment*\n");
    }

    #[test]
    fn test_function_code_from_gcode() {
        let comment = GCode::Comment(CommentContent::String("hello".into()));
        let f1: FunctionCode = FunctionCode::GCode(comment.clone());
        let f2: FunctionCode = comment.into();
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_command_from_function_code() {
        let comment = FunctionCode::GCode(GCode::Comment(CommentContent::String("hello".into())));
        let c1: Command = Command::FunctionCode(comment.clone());
        let c2: Command = comment.into();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_command_from_extended_code() {
        let delete_attr = ExtendedCode::DeleteAttribute(
            AttributeDeletionCriterion::SingleApertureAttribute("test".to_string()),
        );
        let c1: Command = Command::ExtendedCode(delete_attr.clone());
        let c2: Command = delete_attr.into();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_extended_code_from_step_and_repeat() {
        let e1: ExtendedCode = ExtendedCode::StepAndRepeat(StepAndRepeat::Close);
        let e2: ExtendedCode = StepAndRepeat::Close.into();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_extended_code_from_aperture_block() {
        let e1: ExtendedCode = ExtendedCode::ApertureBlock(ApertureBlock::Open { code: 102 });
        let e2: ExtendedCode = ApertureBlock::Open { code: 102 }.into();
        assert_eq!(e1, e2);
    }
    #[test]
    fn test_extended_code_from_polarity() {
        let e1: ExtendedCode = ExtendedCode::LoadPolarity(Polarity::Dark);
        let e2: ExtendedCode = Polarity::Dark.into();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_extended_code_from_mirroring() {
        let e1: ExtendedCode = ExtendedCode::LoadMirroring(Mirroring::XY);
        let e2: ExtendedCode = Mirroring::XY.into();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_extended_code_from_scaling() {
        let e1: ExtendedCode = ExtendedCode::LoadScaling(Scaling { scale: 50.0 });
        let e2: ExtendedCode = Scaling { scale: 50.0 }.into();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_extended_code_from_rotation() {
        let e1: ExtendedCode = ExtendedCode::LoadRotation(Rotation { rotation: 90.0 });
        let e2: ExtendedCode = Rotation { rotation: 90.0 }.into();
        assert_eq!(e1, e2);
    }
}

// Date/Time
pub type GerberDate = DateTime<FixedOffset>;
