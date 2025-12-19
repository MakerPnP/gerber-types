//! Example from an older version of the Gerber Specification
//! chapter 2.12.2
use std::io::stdout;

use std::convert::TryFrom;

use gerber_types::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let cf = CoordinateFormat::new(ZeroOmission::Leading, CoordinateMode::Absolute, 2, 6);
    let commands: Vec<Command> = vec![
        FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "Ucamco ex. 2: Shapes".to_string(),
        )))
        .into(),
        ExtendedCode::CoordinateFormat(cf).into(),
        ExtendedCode::Unit(Unit::Inches).into(),
        ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(GenerationSoftware::new(
            "Rust Gerber",
            "gerber-types",
            Some(VERSION),
        )))
        .into(),
        ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other(
            "Only an example".to_string(),
        )))
        .into(),
        ExtendedCode::LoadPolarity(Polarity::Dark).into(),
        FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "Define Apertures".to_string(),
        )))
        .into(),
        ExtendedCode::ApertureMacro(ApertureMacro::new("TARGET125").add_content(MoirePrimitive {
            center: (0.0.into(), 0.0.into()),
            diameter: 0.125.into(),
            ring_thickness: 0.01.into(),
            gap: 0.01.into(),
            max_rings: 3,
            cross_hair_thickness: 0.003.into(),
            cross_hair_length: 0.150.into(),
            angle: 0.0.into(),
        }))
        .into(),
        ExtendedCode::ApertureMacro(ApertureMacro::new("THERMAL80").add_content(
            ThermalPrimitive {
                center: (0.0.into(), 0.0.into()),
                outer_diameter: 0.08.into(),
                inner_diameter: 0.055.into(),
                gap: 0.0125.into(),
                angle: 45.0.into(),
            },
        ))
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle {
                diameter: 0.01,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 11,
            aperture: Aperture::Circle(Circle {
                diameter: 0.06,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 12,
            aperture: Aperture::Rectangle(Rectangular {
                x: 0.06,
                y: 0.06,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 13,
            aperture: Aperture::Rectangle(Rectangular {
                x: 0.04,
                y: 0.1,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 14,
            aperture: Aperture::Rectangle(Rectangular {
                x: 0.1,
                y: 0.04,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 15,
            aperture: Aperture::Obround(Rectangular {
                x: 0.04,
                y: 0.1,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 16,
            aperture: Aperture::Polygon(Polygon {
                diameter: 0.1,
                vertices: 3,
                rotation: None,
                hole_diameter: None,
            }),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 18,
            aperture: Aperture::Macro("TARGET125".to_string(), None),
        })
        .into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 19,
            aperture: Aperture::Macro("THERMAL80".to_string(), None),
        })
        .into(),
        FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "Start image generation".to_string(),
        )))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            0,
            CoordinateNumber::try_from(0.25).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(0, 0, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(
                CoordinateNumber::try_from(0.25).unwrap(),
                0,
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            1, 1, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(
                CoordinateNumber::try_from(1.5).unwrap(),
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(
                2,
                CoordinateNumber::try_from(1.5).unwrap(),
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::at_x(
            CoordinateNumber::try_from(2.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(1, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(11)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            1, 1, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            2, 1, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(2.5).unwrap(),
            1,
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(2.5).unwrap(),
            CoordinateNumber::try_from(1.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            2,
            CoordinateNumber::try_from(1.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(12)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            1,
            CoordinateNumber::try_from(1.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(13)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            3,
            CoordinateNumber::try_from(1.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(14)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            3,
            CoordinateNumber::try_from(1.25).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(15)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            3, 1, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            CoordinateNumber::try_from(3.75).unwrap(),
            1,
            cf,
        )))))
        .into(),
        FunctionCode::GCode(GCode::QuadrantMode(QuadrantMode::Multi)).into(),
        FunctionCode::GCode(GCode::InterpolationMode(
            InterpolationMode::CounterclockwiseCircular,
        ))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(
                CoordinateNumber::try_from(3.75).unwrap(),
                1,
                cf,
            )),
            Some(CoordinateOffset::new(
                CoordinateNumber::try_from(0.25).unwrap(),
                0,
                cf,
            )),
        )))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(16)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(3.4).unwrap(),
            1,
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(3.5).unwrap(),
            CoordinateNumber::try_from(0.9).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::GCode(GCode::RegionMode(true)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            CoordinateNumber::try_from(0.5).unwrap(),
            2,
            cf,
        )))))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(
                CoordinateNumber::try_from(3.75).unwrap(),
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(
                CoordinateNumber::try_from(3.75).unwrap(),
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(2, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(
                CoordinateNumber::try_from(0.5).unwrap(),
                cf,
            )),
            None,
        )))
        .into(),
        FunctionCode::GCode(GCode::RegionMode(false)).into(),
        FunctionCode::DCode(DCode::SelectAperture(18)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            0,
            CoordinateNumber::try_from(3.875).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(3.875).unwrap(),
            CoordinateNumber::try_from(3.875).unwrap(),
            cf,
        )))))
        .into(),
        ExtendedCode::LoadPolarity(Polarity::Clear).into(),
        FunctionCode::GCode(GCode::RegionMode(true)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            1,
            CoordinateNumber::try_from(2.5).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(3, cf)),
            None,
        )))
        .into(),
        FunctionCode::GCode(GCode::QuadrantMode(QuadrantMode::Single)).into(),
        FunctionCode::GCode(GCode::InterpolationMode(
            InterpolationMode::ClockwiseCircular,
        ))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(
                CoordinateNumber::try_from(1.25).unwrap(),
                CoordinateNumber::try_from(3.25).unwrap(),
                cf,
            )),
            Some(CoordinateOffset::new(
                CoordinateNumber::try_from(0.25).unwrap(),
                0,
                cf,
            )),
        )))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(3, cf)),
            None,
        )))
        .into(),
        FunctionCode::GCode(GCode::QuadrantMode(QuadrantMode::Multi)).into(),
        FunctionCode::GCode(GCode::InterpolationMode(
            InterpolationMode::ClockwiseCircular,
        ))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(
                3,
                CoordinateNumber::try_from(2.5).unwrap(),
                cf,
            )),
            Some(CoordinateOffset::new(
                0,
                CoordinateNumber::try_from(0.375).unwrap(),
                cf,
            )),
        )))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(1, cf)),
            None,
        )))
        .into(),
        FunctionCode::GCode(GCode::RegionMode(false)).into(),
        ExtendedCode::LoadPolarity(Polarity::Dark).into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            CoordinateNumber::try_from(1.5).unwrap(),
            CoordinateNumber::try_from(2.875).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(2, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(11)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(1.5).unwrap(),
            CoordinateNumber::try_from(2.875).unwrap(),
            cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::at_x(
            2, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(19)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Flash(Some(Coordinates::new(
            CoordinateNumber::try_from(2.875).unwrap(),
            CoordinateNumber::try_from(2.875).unwrap(),
            cf,
        )))))
        .into(),
        ExtendedCode::FileAttribute(FileAttribute::Md5(
            "6ab9e892830469cdff7e3e346331d404".to_string(),
        ))
        .into(),
        FunctionCode::MCode(MCode::EndOfFile).into(),
    ];
    let mut stdout = stdout();
    commands.serialize(&mut stdout).unwrap();
}
