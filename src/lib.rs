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
mod serializaion_tests {
    use super::traits::PartialGerberCode;
    use super::*;
    use chrono::DateTime;
    use std::io::BufWriter;
    use uuid::Uuid;

    #[test]
    fn test_comment() {
        //! The serialize method of the GerberCode trait should generate strings.
        let comment = GCode::Comment("testcomment".to_string());
        assert_code!(comment, "G04 testcomment*\n");
    }

    #[test]
    fn test_vec_of_comments() {
        //! A `Vec<T: GerberCode>` should also implement `GerberCode`.
        let mut v = Vec::new();
        v.push(GCode::Comment("comment 1".to_string()));
        v.push(GCode::Comment("another one".to_string()));
        assert_code!(v, "G04 comment 1*\nG04 another one*\n");
    }

    #[test]
    fn test_single_command() {
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
    fn test_polarity() {
        let d = ExtendedCode::LoadPolarity(Polarity::Dark);
        let c = ExtendedCode::LoadPolarity(Polarity::Clear);
        assert_code!(d, "%LPD*%\n");
        assert_code!(c, "%LPC*%\n");
    }

    #[test]
    fn test_step_and_repeat() {
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
    fn test_aperture_block_serialize() {
        let o = ExtendedCode::ApertureBlock(ApertureBlock::Open { code: 102 });
        let c = ExtendedCode::ApertureBlock(ApertureBlock::Close);
        assert_code!(o, "%AB102*%\n");
        assert_code!(c, "%AB*%\n");
    }

    #[test]
    fn test_delete_attribute() {
        let d = ExtendedCode::DeleteAttribute("foo".into());
        assert_code!(d, "%TDfoo*%\n");
    }

    #[test]
    fn test_file_attribute_part() {
        let part = ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other("Part 1".into())));
        assert_code!(part, "%TF.Part,Other,Part 1*%\n");
    }

    #[test]
    fn test_file_attribute_generation_software() {
        let gensw1 = ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(
            GenerationSoftware::new("Vendor 1", "App 1", None),
        ));
        assert_code!(gensw1, "%TF.GenerationSoftware,Vendor 1,App 1*%\n");

        let gensw2 = ExtendedCode::FileAttribute(FileAttribute::GenerationSoftware(
            GenerationSoftware::new("Vendor 1", "App 1", Some("1.2.3")),
        ));
        assert_code!(gensw2, "%TF.GenerationSoftware,Vendor 1,App 1,1.2.3*%\n");
    }

    #[test]
    fn test_file_attribute_creation_date() {
        let date = DateTime::parse_from_rfc3339("2025-06-10T16:25:00+02:00").unwrap();
        let date = ExtendedCode::FileAttribute(FileAttribute::CreationDate(date));
        assert_code!(date, "%TF.CreationDate,2025-06-10T16:25:00+02:00*%\n");
    }

    #[test]
    fn test_file_attribute_project_id() {
        let proj = ExtendedCode::FileAttribute(FileAttribute::ProjectId {
            id: "Project".into(),
            guid: Uuid::max(),
            revision: "rev1".into(),
        });
        assert_code!(
            proj,
            "%TF.ProjectId,Project,ffffffff-ffff-ffff-ffff-ffffffffffff,rev1*%\n"
        );
    }

    #[test]
    fn test_file_attribute_file_function_copper() {
        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Copper {
            layer: 1,
            pos: ExtendedPosition::Top,
            copper_type: None,
        }));
        assert_code!(func, "%TF.FileFunction,Copper,L1,Top*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Copper {
            layer: 2,
            pos: ExtendedPosition::Bottom,
            copper_type: Some(CopperType::Hatched),
        }));
        assert_code!(func, "%TF.FileFunction,Copper,L2,Bot,Hatched*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Copper {
            layer: 3,
            pos: ExtendedPosition::Bottom,
            copper_type: Some(CopperType::Mixed),
        }));
        assert_code!(func, "%TF.FileFunction,Copper,L3,Bot,Mixed*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Copper {
            layer: 4,
            pos: ExtendedPosition::Bottom,
            copper_type: Some(CopperType::Plane),
        }));
        assert_code!(func, "%TF.FileFunction,Copper,L4,Bot,Plane*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Copper {
            layer: 5,
            pos: ExtendedPosition::Bottom,
            copper_type: Some(CopperType::Signal),
        }));
        assert_code!(func, "%TF.FileFunction,Copper,L5,Bot,Signal*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_plated() {
        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 1,
            to_layer: 2,
            drill: Drill::Blind,
            label: None,
        }));
        assert_code!(func, "%TF.FileFunction,Plated,1,2,Blind*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 1,
            to_layer: 4,
            drill: Drill::ThroughHole,
            label: None,
        }));
        assert_code!(func, "%TF.FileFunction,Plated,1,4,PTH*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 2,
            to_layer: 3,
            drill: Drill::Buried,
            label: None,
        }));
        assert_code!(func, "%TF.FileFunction,Plated,2,3,Buried*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 1,
            to_layer: 2,
            drill: Drill::ThroughHole,
            label: Some(DrillRouteType::Drill),
        }));
        assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Drill*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 1,
            to_layer: 2,
            drill: Drill::ThroughHole,
            label: Some(DrillRouteType::Mixed),
        }));
        assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Mixed*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Plated {
            from_layer: 1,
            to_layer: 2,
            drill: Drill::ThroughHole,
            label: Some(DrillRouteType::Route),
        }));
        assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Rout*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_non_plated() {
        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 1,
                to_layer: 2,
                drill: Drill::Blind,
                label: None,
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,1,2,Blind*%\n");

        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 1,
                to_layer: 4,
                drill: Drill::ThroughHole,
                label: None,
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,1,4,PTH*%\n");

        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 2,
                to_layer: 3,
                drill: Drill::Buried,
                label: None,
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,2,3,Buried*%\n");

        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 1,
                to_layer: 2,
                drill: Drill::ThroughHole,
                label: Some(DrillRouteType::Drill),
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,1,2,PTH,Drill*%\n");

        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 1,
                to_layer: 2,
                drill: Drill::ThroughHole,
                label: Some(DrillRouteType::Mixed),
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,1,2,PTH,Mixed*%\n");

        let func =
            ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::NonPlated {
                from_layer: 1,
                to_layer: 2,
                drill: Drill::ThroughHole,
                label: Some(DrillRouteType::Route),
            }));
        assert_code!(func, "%TF.FileFunction,NonPlated,1,2,PTH,Rout*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_profile() {
        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Profile(
            Profile::Plated,
        )));
        assert_code!(func, "%TF.FileFunction,Profile,P*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::Profile(
            Profile::NonPlated,
        )));
        assert_code!(func, "%TF.FileFunction,Profile,NP*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_keepout() {
        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::KeepOut(
            Position::Top,
        )));
        assert_code!(func, "%TF.FileFunction,Keepout,Top*%\n");

        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::KeepOut(
            Position::Bottom,
        )));
        assert_code!(func, "%TF.FileFunction,Keepout,Bot*%\n");
    }

    macro_rules! test_position_and_index {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff {
                        pos: Position::Top,
                        index: None,
                    }));
                assert_code!(func, &format!("%TF.FileFunction,{},Top*%\n", $value));

                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff {
                        pos: Position::Top,
                        index: Some(1),
                    }));
                assert_code!(func, &format!("%TF.FileFunction,{},Top,1*%\n", $value));

                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff {
                        pos: Position::Bottom,
                        index: None,
                    }));
                assert_code!(func, &format!("%TF.FileFunction,{},Bot*%\n", $value));
            }
        };
    }

    macro_rules! test_layer_and_position {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff {
                        pos: Position::Top,
                        layer: 1,
                    }));
                assert_code!(func, &format!("%TF.FileFunction,{},L{},Top*%\n", $value, 1));

                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff {
                        pos: Position::Bottom,
                        layer: 2,
                    }));
                assert_code!(func, &format!("%TF.FileFunction,{},L{},Bot*%\n", $value, 2));
            }
        };
    }

    macro_rules! test_position {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(Position::Top),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{},Top*%\n", $value));

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(Position::Bottom),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{},Bot*%\n", $value));
            }
        };
    }

    macro_rules! test_optional_position {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(None),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{}*%\n", $value));

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(Some(Position::Top)),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{},Top*%\n", $value));

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(Some(Position::Bottom)),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{},Bot*%\n", $value));
            }
        };
    }
    macro_rules! test_simple {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let func =
                    ExtendedCode::FileAttribute(FileAttribute::FileFunction(FileFunction::$ff));
                assert_code!(func, &format!("%TF.FileFunction,{}*%\n", $value));
            }
        };
    }

    macro_rules! test_string {
        ($test:ident, $ff:ident, $value:literal) => {
            #[test]
            fn $test() {
                let string = "A String".to_string();
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::$ff(string.clone()),
                ));
                assert_code!(func, &format!("%TF.FileFunction,{},{}*%\n", $value, string));
            }
        };
    }

    //
    // It should be noted that the gerber spec is very-inconsistent with casing, e.g. "Soldermask" vs "AssemblyDrawing"
    //

    test_position_and_index!(
        test_file_attribute_file_function_soldermask,
        SolderMask,
        "Soldermask"
    );
    test_position_and_index!(test_file_attribute_file_function_legend, Legend, "Legend");
    test_layer_and_position!(
        test_file_attribute_file_function_component,
        Component,
        "Component"
    );
    test_position!(test_file_attribute_file_function_paste, Paste, "Paste");
    test_position!(test_file_attribute_file_function_glue, Glue, "Glue");
    test_position_and_index!(
        test_file_attribute_file_function_carbonmask,
        CarbonMask,
        "Carbonmask"
    );
    test_position_and_index!(
        test_file_attribute_file_function_goldmask,
        GoldMask,
        "Goldmask"
    );
    test_position_and_index!(
        test_file_attribute_file_function_heatsinkmask,
        HeatsinkMask,
        "Heatsinkmask"
    );
    test_position_and_index!(
        test_file_attribute_file_function_peelablemask,
        PeelableMask,
        "Peelablemask"
    );
    test_position_and_index!(
        test_file_attribute_file_function_silvermask,
        SilverMask,
        "Silvermask"
    );
    test_position_and_index!(
        test_file_attribute_file_function_tinmask,
        TinMask,
        "Tinmask"
    );
    test_position!(
        test_file_attribute_file_function_depthroute,
        DepthRoute,
        "Depthrout"
    );
    test_optional_position!(test_file_attribute_file_function_vcut, VCut, "Vcut");
    test_simple!(
        test_file_attribute_file_function_viafill,
        ViaFill,
        "Viafill"
    );
    test_position!(test_file_attribute_file_function_pads, Pads, "Pads");
    test_string!(test_file_attribute_file_function_other, Other, "Other");
    test_simple!(
        test_file_attribute_file_function_drillmap,
        DrillMap,
        "Drillmap"
    );
    test_simple!(
        test_file_attribute_file_function_fabricationdrawing,
        FabricationDrawing,
        "FabricationDrawing"
    );
    test_simple!(
        test_file_attribute_file_function_vcutmap,
        VCutMap,
        "Vcutmap"
    );
    test_position!(
        test_file_attribute_file_function_assemblydrawing,
        AssemblyDrawing,
        "AssemblyDrawing"
    );
    test_simple!(
        test_file_attribute_file_function_arraydrawing,
        ArrayDrawing,
        "ArrayDrawing"
    );
    test_string!(
        test_file_attribute_file_function_otherdrawing,
        OtherDrawing,
        "OtherDrawing"
    );

    #[test]
    fn test_file_attribute_file_polarity() {
        let pol = ExtendedCode::FileAttribute(FileAttribute::FilePolarity(FilePolarity::Positive));
        assert_code!(pol, "%TF.FilePolarity,Positive*%\n");

        let pol = ExtendedCode::FileAttribute(FileAttribute::FilePolarity(FilePolarity::Negative));
        assert_code!(pol, "%TF.FilePolarity,Negative*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_md5() {
        let md5 = ExtendedCode::FileAttribute(FileAttribute::Md5("abcd1234".into()));
        assert_code!(md5, "%TF.MD5,abcd1234*%\n");
    }

    #[test]
    fn test_file_attribute_file_function_user_defined() {
        let owner = ExtendedCode::FileAttribute(FileAttribute::UserDefined {
            name: "Authors".to_string(),
            values: vec!["Author 1".to_string(), "Author 2".to_string()],
        });
        assert_code!(owner, "%TF.Authors,Author 1,Author 2*%\n");
    }

    #[test]
    fn test_aperture_attribute() {
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
