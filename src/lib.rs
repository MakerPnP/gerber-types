//! # Gerber commands
//!
//! This crate implements the basic building blocks of Gerber (RS-274X, aka
//! Extended Gerber version 2) code. It focusses on the low level types and does
//! not do any semantic checking.
//!
//! For example, you can use an aperture without defining it. This will
//! generate syntactically valid but semantially invalid Gerber code, but this
//! module won't complain.
//!
//! ## Traits: GerberCode and PartialGerberCode
//!
//! There are two main traits that are used for code generation:
//!
//! - [`GerberCode`](trait.GerberCode.html) generates a full Gerber code line,
//!   terminated with a newline character.
//! - `PartialGerberCode` (internal only) generates Gerber representation of a
//!   value, but does not represent a full line of code.
#![allow(clippy::new_without_default)]

#[cfg(test)]
#[macro_use]
mod test_macros;

mod attributes;
mod codegen;
mod coordinates;
mod errors;
mod extended_codes;
mod function_codes;
mod macros;
mod traits;
mod types;

pub use crate::attributes::*;
pub use crate::coordinates::*;
pub use crate::errors::*;
pub use crate::extended_codes::*;
pub use crate::function_codes::*;
pub use crate::macros::*;
pub use crate::traits::GerberCode;
pub use crate::types::*;

#[cfg(test)]
mod test {
    use std::io::BufWriter;

    use super::traits::PartialGerberCode;
    use super::*;

    #[test]
    fn test_serialize() {
        //! The serialize method of the GerberCode trait should generate strings.
        let comment = GCode::Comment("testcomment".to_string());
        assert_code!(comment, "G04 testcomment*\n");
    }

    #[test]
    fn test_vec_serialize() {
        //! A `Vec<T: GerberCode>` should also implement `GerberCode`.
        let mut v = Vec::new();
        v.push(GCode::Comment("comment 1".to_string()));
        v.push(GCode::Comment("another one".to_string()));
        assert_code!(v, "G04 comment 1*\nG04 another one*\n");
    }

    #[test]
    fn test_command_serialize() {
        //! A `Command` should implement `GerberCode`
        let c = Command::FunctionCode(FunctionCode::GCode(GCode::Comment("comment".to_string())));
        assert_code!(c, "G04 comment*\n");
    }

    #[test]
    fn test_interpolation_mode() {
        let mut commands = Vec::new();
        let c1 = GCode::InterpolationMode(InterpolationMode::Linear);
        let c2 = GCode::InterpolationMode(InterpolationMode::ClockwiseCircular);
        let c3 = GCode::InterpolationMode(InterpolationMode::CounterclockwiseCircular);
        commands.push(c1);
        commands.push(c2);
        commands.push(c3);
        assert_code!(commands, "G01*\nG02*\nG03*\n");
    }

    #[test]
    fn test_region_mode() {
        let mut commands = Vec::new();
        commands.push(GCode::RegionMode(true));
        commands.push(GCode::RegionMode(false));
        assert_code!(commands, "G36*\nG37*\n");
    }

    #[test]
    fn test_quadrant_mode() {
        let mut commands = Vec::new();
        commands.push(GCode::QuadrantMode(QuadrantMode::Single));
        commands.push(GCode::QuadrantMode(QuadrantMode::Multi));
        assert_code!(commands, "G74*\nG75*\n");
    }

    #[test]
    fn test_end_of_file() {
        let c = MCode::EndOfFile;
        assert_code!(c, "M02*\n");
    }

    #[test]
    fn test_operation_interpolate() {
        let cf = CoordinateFormat::new(2, 5);
        let c1 = Operation::Interpolate(
            Coordinates::new(1, 2, cf),
            Some(CoordinateOffset::new(5, 10, cf)),
        );
        assert_code!(c1, "X100000Y200000I500000J1000000D01*\n");
        let c2 = Operation::Interpolate(Coordinates::at_y(-2, CoordinateFormat::new(4, 4)), None);
        assert_code!(c2, "Y-20000D01*\n");
        let cf = CoordinateFormat::new(4, 4);
        let c3 = Operation::Interpolate(
            Coordinates::at_x(1, cf),
            Some(CoordinateOffset::at_y(2, cf)),
        );
        assert_code!(c3, "X10000J20000D01*\n");
    }

    #[test]
    fn test_operation_move() {
        let c = Operation::Move(Coordinates::new(23, 42, CoordinateFormat::new(6, 4)));
        assert_code!(c, "X230000Y420000D02*\n");
    }

    #[test]
    fn test_operation_flash() {
        let c = Operation::Flash(Coordinates::new(23, 42, CoordinateFormat::new(4, 4)));
        assert_code!(c, "X230000Y420000D03*\n");
    }

    #[test]
    fn test_select_aperture() {
        let c1 = DCode::SelectAperture(10);
        assert_code!(c1, "D10*\n");
        let c2 = DCode::SelectAperture(2147483647);
        assert_code!(c2, "D2147483647*\n");
    }

    #[test]
    fn test_coordinate_format() {
        let c = ExtendedCode::CoordinateFormat(CoordinateFormat::new(2, 5));
        assert_code!(c, "%FSLAX25Y25*%\n");
    }

    #[test]
    fn test_unit() {
        let c1 = ExtendedCode::Unit(Unit::Millimeters);
        let c2 = ExtendedCode::Unit(Unit::Inches);
        assert_code!(c1, "%MOMM*%\n");
        assert_code!(c2, "%MOIN*%\n");
    }

    #[test]
    fn test_aperture_circle_definition() {
        let ad1 = ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle {
                diameter: 4.0,
                hole_diameter: Some(2.0),
            }),
        };
        let ad2 = ApertureDefinition {
            code: 11,
            aperture: Aperture::Circle(Circle {
                diameter: 4.5,
                hole_diameter: None,
            }),
        };
        assert_partial_code!(ad1, "10C,4X2");
        assert_partial_code!(ad2, "11C,4.5");
    }

    #[test]
    fn test_aperture_rectangular_definition() {
        let ad1 = ApertureDefinition {
            code: 12,
            aperture: Aperture::Rectangle(Rectangular {
                x: 1.5,
                y: 2.25,
                hole_diameter: Some(3.8),
            }),
        };
        let ad2 = ApertureDefinition {
            code: 13,
            aperture: Aperture::Rectangle(Rectangular {
                x: 1.0,
                y: 1.0,
                hole_diameter: None,
            }),
        };
        let ad3 = ApertureDefinition {
            code: 14,
            aperture: Aperture::Obround(Rectangular {
                x: 2.0,
                y: 4.5,
                hole_diameter: None,
            }),
        };
        assert_partial_code!(ad1, "12R,1.5X2.25X3.8");
        assert_partial_code!(ad2, "13R,1X1");
        assert_partial_code!(ad3, "14O,2X4.5");
    }

    #[test]
    fn test_aperture_polygon_definition() {
        let ad1 = ApertureDefinition {
            code: 15,
            aperture: Aperture::Polygon(Polygon {
                diameter: 4.5,
                vertices: 3,
                rotation: None,
                hole_diameter: None,
            }),
        };
        let ad2 = ApertureDefinition {
            code: 16,
            aperture: Aperture::Polygon(Polygon {
                diameter: 5.0,
                vertices: 4,
                rotation: Some(30.6),
                hole_diameter: None,
            }),
        };
        let ad3 = ApertureDefinition {
            code: 17,
            aperture: Aperture::Polygon(Polygon {
                diameter: 5.5,
                vertices: 5,
                rotation: None,
                hole_diameter: Some(1.8),
            }),
        };
        assert_partial_code!(ad1, "15P,4.5X3");
        assert_partial_code!(ad2, "16P,5X4X30.6");
        assert_partial_code!(ad3, "17P,5.5X5X0X1.8");
    }

    #[test]
    fn test_aperture_macro_definition() {
        let m1 = ApertureDefinition {
            code: 42,
            aperture: Aperture::Macro("NO_ARGS1".to_string(), None),
        };
        let m2 = ApertureDefinition {
            code: 69,
            aperture: Aperture::Macro(
                "With_Args2".to_string(),
                Some(vec![
                    MacroDecimal::Variable(1),
                    MacroDecimal::Value(0.25),
                    MacroDecimal::Expression("$1x$2".to_string()),
                ]),
            ),
        };
        assert_partial_code!(m1, "42NO_ARGS1");
        assert_partial_code!(m2, "69With_Args2,$1X0.25X$1x$2");
    }

    #[test]
    fn test_polarity_serialize() {
        let d = ExtendedCode::LoadPolarity(Polarity::Dark);
        let c = ExtendedCode::LoadPolarity(Polarity::Clear);
        assert_code!(d, "%LPD*%\n");
        assert_code!(c, "%LPC*%\n");
    }

    #[test]
    fn test_step_and_repeat_serialize() {
        let o = ExtendedCode::StepAndRepeat(StepAndRepeat::Open {
            repeat_x: 2,
            repeat_y: 3,
            distance_x: 2.0,
            distance_y: 3.0,
        });
        let c = ExtendedCode::StepAndRepeat(StepAndRepeat::Close);
        assert_code!(o, "%SRX2Y3I2J3*%\n");
        assert_code!(c, "%SR*%\n");
    }

    #[test]
    fn test_delete_attribute_serialize() {
        let d = ExtendedCode::DeleteAttribute("foo".into());
        assert_code!(d, "%TDfoo*%\n");
    }

    #[test]
    fn test_file_attribute_serialize() {
        let part = ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other("foo".into())));
        assert_code!(part, "%TF.Part,Other,foo*%\n");

        let gensw1 = ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(
            GenerationSoftware::new("Vend0r", "superpcb", None),
        ));
        assert_code!(gensw1, "%TF.GenerationSoftware,Vend0r,superpcb*%\n");

        let gensw2 = ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(
            GenerationSoftware::new("Vend0r", "superpcb", Some("1.2.3")),
        ));
        assert_code!(gensw2, "%TF.GenerationSoftware,Vend0r,superpcb,1.2.3*%\n");
    }

    #[test]
    fn test_aperture_attribute_serialize() {
        // Test with Profile (found in "All data layers" section of the enum)
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Profile,
        ));
        assert_code!(function, "%TA.AperFunction,Profile*%\n");

        // "Drill and rout layers"
        // ViaDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(None),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::None)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,None*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::Ia)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,Ia*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::Ib)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,Ib*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IIa)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IIa*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IIb)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IIb*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IIIa)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IIIa*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IIIb)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IIIb*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IVa)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IVa*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::IVb)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,IVb*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::V)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,V*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::VI)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,VI*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaDrill(Some(IPC4761ViaProtection::VII)),
        ));
        assert_code!(function, "%TA.AperFunction,ViaDrill,VII*%\n");

        // BackDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::BackDrill,
        ));
        assert_code!(function, "%TA.AperFunction,BackDrill*%\n");

        // ComponentDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentDrill { press_fit: None },
        ));
        assert_code!(function, "%TA.AperFunction,ComponentDrill*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentDrill {
                press_fit: Some(true),
            },
        ));
        assert_code!(function, "%TA.AperFunction,ComponentDrill,PressFit*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentDrill {
                press_fit: Some(false),
            },
        ));
        assert_code!(function, "%TA.AperFunction,ComponentDrill*%\n");

        // MechanicalDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::MechanicalDrill { function: None },
        ));
        assert_code!(function, "%TA.AperFunction,MechanicalDrill*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::MechanicalDrill {
                function: Some(DrillFunction::Other),
            },
        ));
        assert_code!(function, "%TA.AperFunction,MechanicalDrill,Other*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::MechanicalDrill {
                function: Some(DrillFunction::BreakOut),
            },
        ));
        assert_code!(function, "%TA.AperFunction,MechanicalDrill,BreakOut*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::MechanicalDrill {
                function: Some(DrillFunction::Tooling),
            },
        ));
        assert_code!(function, "%TA.AperFunction,MechanicalDrill,Tooling*%\n");

        // CastellatedDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::CastellatedDrill,
        ));
        assert_code!(function, "%TA.AperFunction,CastellatedDrill*%\n");

        // OtherDrill
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::OtherDrill("CustomDrill".to_string()),
        ));
        assert_code!(function, "%TA.AperFunction,OtherDrill,CustomDrill*%\n");

        // "Copper layers"
        // ComponentPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentPad,
        ));
        assert_code!(function, "%TA.AperFunction,ComponentPad*%\n");

        // SmdPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::SmdPad(SmdPadType::CopperDefined),
        ));
        assert_code!(function, "%TA.AperFunction,SMDPad,CuDef*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::SmdPad(SmdPadType::SoldermaskDefined),
        ));
        assert_code!(function, "%TA.AperFunction,SMDPad,SMDef*%\n");

        // BgaPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::BgaPad(SmdPadType::CopperDefined),
        ));
        assert_code!(function, "%TA.AperFunction,BGAPad,CuDef*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::BgaPad(SmdPadType::SoldermaskDefined),
        ));
        assert_code!(function, "%TA.AperFunction,BGAPad,SMDef*%\n");

        // ConnectorPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ConnectorPad,
        ));
        assert_code!(function, "%TA.AperFunction,ConnectorPad*%\n");

        // HeatsinkPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::HeatsinkPad,
        ));
        assert_code!(function, "%TA.AperFunction,HeatsinkPad*%\n");

        // ViaPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ViaPad,
        ));
        assert_code!(function, "%TA.AperFunction,ViaPad*%\n");

        // TestPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::TestPad,
        ));
        assert_code!(function, "%TA.AperFunction,TestPad*%\n");

        // CastellatedPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::CastellatedPad,
        ));
        assert_code!(function, "%TA.AperFunction,CastellatedPad*%\n");

        // FiducialPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::FiducialPad(FiducialScope::Global),
        ));
        assert_code!(function, "%TA.AperFunction,FiducialPad,Global*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::FiducialPad(FiducialScope::Local),
        ));
        assert_code!(function, "%TA.AperFunction,FiducialPad,Local*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::FiducialPad(FiducialScope::Panel),
        ));
        assert_code!(function, "%TA.AperFunction,FiducialPad,Panel*%\n");

        // ThermalReliefPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ThermalReliefPad,
        ));
        assert_code!(function, "%TA.AperFunction,ThermalReliefPad*%\n");

        // WasherPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::WasherPad,
        ));
        assert_code!(function, "%TA.AperFunction,WasherPad*%\n");

        // AntiPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::AntiPad,
        ));
        assert_code!(function, "%TA.AperFunction,AntiPad*%\n");

        // OtherPad
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::OtherPad("CustomPad".to_string()),
        ));
        assert_code!(function, "%TA.AperFunction,OtherPad,CustomPad*%\n");

        // Conductor
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Conductor,
        ));
        assert_code!(function, "%TA.AperFunction,Conductor*%\n");

        // EtchedComponent
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::EtchedComponent,
        ));
        assert_code!(function, "%TA.AperFunction,EtchedComponent*%\n");

        // NonConductor
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::NonConductor,
        ));
        assert_code!(function, "%TA.AperFunction,NonConductor*%\n");

        // CopperBalancing
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::CopperBalancing,
        ));
        assert_code!(function, "%TA.AperFunction,CopperBalancing*%\n");

        // Border
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Border,
        ));
        assert_code!(function, "%TA.AperFunction,Border*%\n");

        // OtherCopper
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::OtherCopper("CustomCopper".to_string()),
        ));
        assert_code!(function, "%TA.AperFunction,OtherCopper,CustomCopper*%\n");

        // "All data layers"
        // Profile - already tested at the beginning

        // Material
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Material,
        ));
        assert_code!(function, "%TA.AperFunction,Material*%\n");

        // NonMaterial
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::NonMaterial,
        ));
        assert_code!(function, "%TA.AperFunction,NonMaterial*%\n");

        // Other
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Other("CustomFunction".to_string()),
        ));
        assert_code!(function, "%TA.AperFunction,Other,CustomFunction*%\n");

        // "Component layers"
        // ComponentMain
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentMain,
        ));
        assert_code!(function, "%TA.AperFunction,ComponentMain*%\n");

        // ComponentOutline
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentOutline(None),
        ));
        assert_code!(function, "%TA.AperFunction,ComponentOutline*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentOutline(Some(ComponentOutline::Body)),
        ));
        assert_code!(function, "%TA.AperFunction,ComponentOutline,Body*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentOutline(Some(ComponentOutline::Lead2Lead)),
        ));
        assert_code!(function, "%TA.AperFunction,ComponentOutline,Lead2Lead*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentOutline(Some(ComponentOutline::Footprint)),
        ));
        assert_code!(function, "%TA.AperFunction,ComponentOutline,Footprint*%\n");

        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentOutline(Some(ComponentOutline::Courtyard)),
        ));
        assert_code!(function, "%TA.AperFunction,ComponentOutline,Courtyard*%\n");

        // ComponentPin
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::ComponentPin,
        ));
        assert_code!(function, "%TA.AperFunction,ComponentPin*%\n");

        // "2024.05 - 8.4 - Deprecated attribute values"
        // Slot
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Slot,
        ));
        assert_code!(function, "%TA.AperFunction,Slot*%\n");

        // CutOut
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::CutOut,
        ));
        assert_code!(function, "%TA.AperFunction,CutOut*%\n");

        // Cavity
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Cavity,
        ));
        assert_code!(function, "%TA.AperFunction,Cavity*%\n");

        // Drawing
        let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
            ApertureFunction::Drawing,
        ));
        assert_code!(function, "%TA.AperFunction,Drawing*%\n");
    }
}
