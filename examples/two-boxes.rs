//! Example from spec chapter 2.12.1

use std::io::stdout;

use gerber_types::{
    Aperture, ApertureDefinition, Circle, Command, CoordinateFormat, Coordinates, DCode,
    ExtendedCode, FileAttribute, FunctionCode, GCode, GenerationSoftware, GerberCode,
    InterpolationMode, MCode, Operation, Part, Polarity, Unit,
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let cf = CoordinateFormat::new(2, 5);
    let commands: Vec<Command> = vec![
        FunctionCode::GCode(GCode::Comment("Ucamco ex. 1: Two square boxes".to_string())).into(),
        ExtendedCode::CoordinateFormat(cf).into(),
        ExtendedCode::Unit(Unit::Millimeters).into(),
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
        ExtendedCode::ApertureDefinition(ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle {
                diameter: 0.01,
                hole_diameter: None,
            }),
        })
        .into(),
        FunctionCode::DCode(DCode::SelectAperture(10)).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Coordinates::new(
            0, 0, cf,
        ))))
        .into(),
        FunctionCode::GCode(GCode::InterpolationMode(InterpolationMode::Linear)).into(),
        // TODO: The interpolate representation needs to take the coordinate
        // format into account!
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::new(5, 0, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_y(5, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_x(0, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_y(0, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Move(Coordinates::at_x(6, cf)))).into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_x(11, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_y(5, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_x(6, cf),
            None,
        )))
        .into(),
        FunctionCode::DCode(DCode::Operation(Operation::Interpolate(
            Coordinates::at_y(0, cf),
            None,
        )))
        .into(),
        FunctionCode::MCode(MCode::EndOfFile).into(),
    ];
    let mut stdout = stdout();
    commands.serialize(&mut stdout).unwrap();
}
