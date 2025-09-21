//! Example from Gerber Specification
//! 2025.05 - 2.11.1 "Example: Two Square Boxes"

use std::io::stdout;

use gerber_types::{Aperture, ApertureDefinition, Circle, Command, CommentContent, CoordinateFormat, CoordinateMode, Coordinates, DCode, ExtendedCode, FileAttribute, FunctionCode, GCode, GenerationSoftware, GerberCode, InterpolationMode, MCode, Operation, Part, Polarity, Unit, ZeroOmission};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let cf = CoordinateFormat::new(ZeroOmission::Leading, CoordinateMode::Absolute, 2, 6);
    let commands: Vec<Command> = vec![
        FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "Ucamco ex. 1: Two square boxes".to_string(),
        )))
        .into(),
        ExtendedCode::Unit(Unit::Millimeters).into(),
        ExtendedCode::CoordinateFormat(cf).into(),
        ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(GenerationSoftware::new(
            "MakerPnP",
            "gerber-types",
            Some(VERSION),
        )))
        .into(),
        ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other("example".to_string()))).into(),
        ExtendedCode::LoadPolarity(Polarity::Dark).into(),
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle {
                diameter: 0.01,
                hole_diameter: None,
            }),
        })
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::new(
            0, 0, cf,
        )))))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::new(5, 0, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(5, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(0, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(0, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Some(Coordinates::at_x(
            6, cf,
        )))))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(11, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(5, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_x(6, cf)),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Some(Coordinates::at_y(0, cf)),
            None,
        )))
        .into(),
        FunctionCode::MCode(MCode::EndOfFile).into(),
    ];
    let mut stdout = stdout();
    commands.serialize(&mut stdout).unwrap();
}
