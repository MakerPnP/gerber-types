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

#[macro_use]
mod serialization_macros;

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

// re-export some types
pub use uuid::Uuid;

#[cfg(test)]
mod serialization_tests {
    use super::traits::PartialGerberCode;
    use super::*;
    use std::io::BufWriter;
    use uuid::Uuid;

    #[test]
    fn test_comment() {
        //! The serialize method of the GerberCode trait should generate strings.
        let comment = GCode::Comment(CommentContent::String("testcomment".to_string()));
        assert_code!(comment, "G04 testcomment*\n");
    }

    /// `standard comment` is a term defined in the gerber spec. See `2024.05 4.1 Comment (G04)`
    #[test]
    fn test_standard_comment_with_standard_attributes() {
        //! Attributes should be able to be stored in G04 comments starting with `#@!`
        let comment = GCode::Comment(CommentContent::Standard(
            StandardComment::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::SmdPad(SmdPadType::CopperDefined),
            )),
        ));
        assert_code!(comment, "G04 #@! TA.AperFunction,SMDPad,CuDef*\n");

        let comment = GCode::Comment(CommentContent::Standard(StandardComment::FileAttribute(
            FileAttribute::FileFunction(FileFunction::Profile(Some(Profile::NonPlated))),
        )));
        assert_code!(comment, "G04 #@! TF.FileFunction,Profile,NP*\n");

        let comment = GCode::Comment(CommentContent::Standard(StandardComment::ObjectAttribute(
            ObjectAttribute::Component("R1".to_string()),
        )));
        assert_code!(comment, "G04 #@! TO.C,R1*\n");
    }

    #[test]
    fn test_standard_comment_with_custom_attributes() {
        // custom attributes are not prefixed with a `.`.
        let comment = GCode::Comment(CommentContent::Standard(
            StandardComment::ApertureAttribute(ApertureAttribute::UserDefined {
                name: "Example".to_string(),
                values: vec!["value1".to_string(), "value2".to_string()],
            }),
        ));
        assert_code!(comment, "G04 #@! TAExample,value1,value2*\n");

        let comment = GCode::Comment(CommentContent::Standard(StandardComment::FileAttribute(
            FileAttribute::UserDefined {
                name: "Example".to_string(),
                values: vec!["value1".to_string(), "value2".to_string()],
            },
        )));
        assert_code!(comment, "G04 #@! TFExample,value1,value2*\n");

        let comment = GCode::Comment(CommentContent::Standard(StandardComment::ObjectAttribute(
            ObjectAttribute::UserDefined {
                name: "Example".to_string(),
                values: vec!["value1".to_string(), "value2".to_string()],
            },
        )));
        assert_code!(comment, "G04 #@! TOExample,value1,value2*\n");
    }

    #[test]
    fn test_vec_of_comments() {
        //! A `Vec<T: GerberCode>` should also implement `GerberCode`.
        let mut v = Vec::new();
        v.push(GCode::Comment(CommentContent::String(
            "comment 1".to_string(),
        )));
        v.push(GCode::Comment(CommentContent::String(
            "another one".to_string(),
        )));
        assert_code!(v, "G04 comment 1*\nG04 another one*\n");
    }

    #[test]
    fn test_single_command() {
        //! A `Command` should implement `GerberCode`
        let c = Command::FunctionCode(FunctionCode::GCode(GCode::Comment(CommentContent::String(
            "comment".to_string(),
        ))));
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
            Some(Coordinates::new(1, 2, cf)),
            Some(CoordinateOffset::new(5, 10, cf)),
        );
        assert_code!(c1, "X100000Y200000I500000J1000000D01*\n");
        let c2 = Operation::Interpolate(
            Some(Coordinates::at_y(-2, CoordinateFormat::new(4, 4))),
            None,
        );
        assert_code!(c2, "Y-20000D01*\n");
        let cf = CoordinateFormat::new(4, 4);
        let c3 = Operation::Interpolate(
            Some(Coordinates::at_x(1, cf)),
            Some(CoordinateOffset::at_y(2, cf)),
        );
        assert_code!(c3, "X10000J20000D01*\n");
    }

    #[test]
    fn test_operation_move() {
        let c = Operation::Move(Some(Coordinates::new(23, 42, CoordinateFormat::new(6, 4))));
        assert_code!(c, "X230000Y420000D02*\n");
    }

    #[test]
    fn test_operation_flash() {
        let c = Operation::Flash(Some(Coordinates::new(23, 42, CoordinateFormat::new(4, 4))));
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

    mod aperture_definition {
        use super::*;

        #[test]
        fn test_circle_definition() {
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
        fn test_rectangular_definition() {
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
        fn test_polygon_definition() {
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
        fn test_macro_definition() {
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
    fn test_aperture_block() {
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

    mod file_attribute {
        use super::*;
        #[test]
        fn test_part() {
            let part =
                ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other("Part 1".into())));
            assert_code!(part, "%TF.Part,Other,Part 1*%\n");
        }

        #[test]
        fn test_generation_software() {
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
        fn test_creation_date() {
            let date = GerberDate::parse_from_rfc3339("2025-06-10T16:25:00+02:00").unwrap();
            let date = ExtendedCode::FileAttribute(FileAttribute::CreationDate(date));
            assert_code!(date, "%TF.CreationDate,2025-06-10T16:25:00+02:00*%\n");
        }

        #[test]
        fn test_project_id() {
            let proj = ExtendedCode::FileAttribute(FileAttribute::ProjectId {
                id: "Project".into(),
                uuid: Uuid::max(),
                revision: "rev1".into(),
            });
            assert_code!(
                proj,
                "%TF.ProjectId,Project,ffffffff-ffff-ffff-ffff-ffffffffffff,rev1*%\n"
            );
        }

        mod file_function {
            use super::*;
            #[test]
            fn test_copper() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Copper {
                        layer: 1,
                        pos: ExtendedPosition::Top,
                        copper_type: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Copper,L1,Top*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Copper {
                        layer: 2,
                        pos: ExtendedPosition::Bottom,
                        copper_type: Some(CopperType::Hatched),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Copper,L2,Bot,Hatched*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Copper {
                        layer: 3,
                        pos: ExtendedPosition::Bottom,
                        copper_type: Some(CopperType::Mixed),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Copper,L3,Bot,Mixed*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Copper {
                        layer: 4,
                        pos: ExtendedPosition::Bottom,
                        copper_type: Some(CopperType::Plane),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Copper,L4,Bot,Plane*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Copper {
                        layer: 5,
                        pos: ExtendedPosition::Bottom,
                        copper_type: Some(CopperType::Signal),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Copper,L5,Bot,Signal*%\n");
            }

            #[test]
            fn test_plated() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: PlatedDrill::Blind,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,1,2,Blind*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 1,
                        to_layer: 4,
                        drill: PlatedDrill::PlatedThroughHole,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,1,4,PTH*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 2,
                        to_layer: 3,
                        drill: PlatedDrill::Buried,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,2,3,Buried*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: PlatedDrill::PlatedThroughHole,
                        label: Some(DrillRouteType::Drill),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Drill*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: PlatedDrill::PlatedThroughHole,
                        label: Some(DrillRouteType::Mixed),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Mixed*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Plated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: PlatedDrill::PlatedThroughHole,
                        label: Some(DrillRouteType::Route),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,Plated,1,2,PTH,Rout*%\n");
            }

            #[test]
            fn test_non_plated() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: NonPlatedDrill::Blind,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,1,2,Blind*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 1,
                        to_layer: 4,
                        drill: NonPlatedDrill::NonPlatedThroughHole,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,1,4,NPTH*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 2,
                        to_layer: 3,
                        drill: NonPlatedDrill::Buried,
                        label: None,
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,2,3,Buried*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: NonPlatedDrill::NonPlatedThroughHole,
                        label: Some(DrillRouteType::Drill),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,1,2,NPTH,Drill*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: NonPlatedDrill::NonPlatedThroughHole,
                        label: Some(DrillRouteType::Mixed),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,1,2,NPTH,Mixed*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::NonPlated {
                        from_layer: 1,
                        to_layer: 2,
                        drill: NonPlatedDrill::NonPlatedThroughHole,
                        label: Some(DrillRouteType::Route),
                    },
                ));
                assert_code!(func, "%TF.FileFunction,NonPlated,1,2,NPTH,Rout*%\n");
            }

            #[test]
            fn test_profile() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Profile(None),
                ));
                assert_code!(func, "%TF.FileFunction,Profile*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Profile(Some(Profile::Plated)),
                ));
                assert_code!(func, "%TF.FileFunction,Profile,P*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::Profile(Some(Profile::NonPlated)),
                ));
                assert_code!(func, "%TF.FileFunction,Profile,NP*%\n");
            }

            #[test]
            fn test_keepout() {
                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::KeepOut(Position::Top),
                ));
                assert_code!(func, "%TF.FileFunction,Keepout,Top*%\n");

                let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                    FileFunction::KeepOut(Position::Bottom),
                ));
                assert_code!(func, "%TF.FileFunction,Keepout,Bot*%\n");
            }

            macro_rules! test_position_and_index {
                ($test:ident, $ff:ident, $value:literal) => {
                    #[test]
                    fn $test() {
                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff {
                                pos: Position::Top,
                                index: None,
                            },
                        ));
                        assert_code!(func, &format!("%TF.FileFunction,{},Top*%\n", $value));

                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff {
                                pos: Position::Top,
                                index: Some(1),
                            },
                        ));
                        assert_code!(func, &format!("%TF.FileFunction,{},Top,1*%\n", $value));

                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff {
                                pos: Position::Bottom,
                                index: None,
                            },
                        ));
                        assert_code!(func, &format!("%TF.FileFunction,{},Bot*%\n", $value));
                    }
                };
            }

            macro_rules! test_layer_and_position {
                ($test:ident, $ff:ident, $value:literal) => {
                    #[test]
                    fn $test() {
                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff {
                                pos: Position::Top,
                                layer: 1,
                            },
                        ));
                        assert_code!(func, &format!("%TF.FileFunction,{},L{},Top*%\n", $value, 1));

                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff {
                                pos: Position::Bottom,
                                layer: 2,
                            },
                        ));
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
                        let func = ExtendedCode::FileAttribute(FileAttribute::FileFunction(
                            FileFunction::$ff,
                        ));
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

            test_position_and_index!(test_soldermask, SolderMask, "Soldermask");
            test_position_and_index!(test_legend, Legend, "Legend");
            test_layer_and_position!(test_component, Component, "Component");
            test_position!(test_paste, Paste, "Paste");
            test_position!(test_glue, Glue, "Glue");
            test_position_and_index!(test_carbonmask, CarbonMask, "Carbonmask");
            test_position_and_index!(test_goldmask, GoldMask, "Goldmask");
            test_position_and_index!(test_heatsinkmask, HeatsinkMask, "Heatsinkmask");
            test_position_and_index!(test_peelablemask, PeelableMask, "Peelablemask");
            test_position_and_index!(test_silvermask, SilverMask, "Silvermask");
            test_position_and_index!(test_tinmask, TinMask, "Tinmask");
            test_position!(test_depthroute, DepthRoute, "Depthrout");
            test_optional_position!(test_vcut, VCut, "Vcut");
            test_simple!(test_viafill, ViaFill, "Viafill");
            test_position!(test_pads, Pads, "Pads");
            test_string!(test_other, Other, "Other");
            test_simple!(test_drillmap, DrillMap, "Drillmap");
            test_simple!(
                test_fabricationdrawing,
                FabricationDrawing,
                "FabricationDrawing"
            );
            test_simple!(test_vcutmap, VCutMap, "Vcutmap");
            test_position!(test_assemblydrawing, AssemblyDrawing, "AssemblyDrawing");
            test_simple!(test_arraydrawing, ArrayDrawing, "ArrayDrawing");
            test_string!(test_otherdrawing, OtherDrawing, "OtherDrawing");
        }

        #[test]
        fn test_polarity() {
            let pol =
                ExtendedCode::FileAttribute(FileAttribute::FilePolarity(FilePolarity::Positive));
            assert_code!(pol, "%TF.FilePolarity,Positive*%\n");

            let pol =
                ExtendedCode::FileAttribute(FileAttribute::FilePolarity(FilePolarity::Negative));
            assert_code!(pol, "%TF.FilePolarity,Negative*%\n");
        }

        #[test]
        fn test_same_coordinates() {
            let same_coordiantes =
                ExtendedCode::FileAttribute(FileAttribute::SameCoordinates(None));
            assert_code!(same_coordiantes, "%TF.SameCoordinates*%\n");

            let same_coordiantes = ExtendedCode::FileAttribute(FileAttribute::SameCoordinates(
                Some(Ident::Name("Name 1".to_string())),
            ));
            assert_code!(same_coordiantes, "%TF.SameCoordinates,Name 1*%\n");

            let same_coordiantes = ExtendedCode::FileAttribute(FileAttribute::SameCoordinates(
                Some(Ident::Uuid(Uuid::max())),
            ));
            assert_code!(
                same_coordiantes,
                "%TF.SameCoordinates,ffffffff-ffff-ffff-ffff-ffffffffffff*%\n"
            );
        }

        #[test]
        fn test_md5() {
            let md5 = ExtendedCode::FileAttribute(FileAttribute::Md5("abcd1234".into()));
            assert_code!(md5, "%TF.MD5,abcd1234*%\n");
        }

        mod user_defined_attribute {
            use super::*;

            #[test]
            fn test_non_standard() {
                let function = ExtendedCode::FileAttribute(FileAttribute::UserDefined {
                    name: "NonStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there is no '.' prefix, spaces are not trimmed
                assert_code!(function, "%TFNonStandardAttribute,Value 1 , Value 2*%\n");
            }

            #[test]
            fn test_unsupported_standard() {
                let function = ExtendedCode::FileAttribute(FileAttribute::UserDefined {
                    name: ".UnsupportedStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there *is* a '.' prefix, spaces are not trimmed
                assert_code!(
                    function,
                    "%TF.UnsupportedStandardAttribute,Value 1 , Value 2*%\n"
                );
            }
        }
    }

    mod aperture_attribute {
        use super::*;
        //
        // "Drill and rout layers"
        //

        #[test]
        fn test_via_drill() {
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
        }

        #[test]
        fn test_backdrill() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::BackDrill,
            ));
            assert_code!(function, "%TA.AperFunction,BackDrill*%\n");
        }

        #[test]
        fn test_component_drill() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentDrill { function: None },
            ));
            assert_code!(function, "%TA.AperFunction,ComponentDrill*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentDrill {
                    function: Some(ComponentDrill::PressFit),
                },
            ));
            assert_code!(function, "%TA.AperFunction,ComponentDrill,PressFit*%\n");
        }

        #[test]
        fn test_mechanical_drill() {
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
            assert_code!(function, "%TA.AperFunction,MechanicalDrill,Breakout*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::MechanicalDrill {
                    function: Some(DrillFunction::Tooling),
                },
            ));
            assert_code!(function, "%TA.AperFunction,MechanicalDrill,Tooling*%\n");
        }

        #[test]
        fn test_catellated_drill() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::CastellatedDrill,
            ));
            assert_code!(function, "%TA.AperFunction,CastellatedDrill*%\n");
        }

        #[test]
        fn test_other_drill() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::OtherDrill("CustomDrill".to_string()),
            ));
            assert_code!(function, "%TA.AperFunction,OtherDrill,CustomDrill*%\n");
        }

        //
        // "Copper layers"
        //
        #[test]
        fn test_component_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentPad,
            ));
            assert_code!(function, "%TA.AperFunction,ComponentPad*%\n");
        }

        #[test]
        fn test_smd_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::SmdPad(SmdPadType::CopperDefined),
            ));
            assert_code!(function, "%TA.AperFunction,SMDPad,CuDef*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::SmdPad(SmdPadType::SoldermaskDefined),
            ));
            assert_code!(function, "%TA.AperFunction,SMDPad,SMDef*%\n");
        }

        #[test]
        fn test_bga_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::BgaPad(SmdPadType::CopperDefined),
            ));
            assert_code!(function, "%TA.AperFunction,BGAPad,CuDef*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::BgaPad(SmdPadType::SoldermaskDefined),
            ));
            assert_code!(function, "%TA.AperFunction,BGAPad,SMDef*%\n");
        }

        #[test]
        fn test_connector_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ConnectorPad,
            ));
            assert_code!(function, "%TA.AperFunction,ConnectorPad*%\n");
        }

        #[test]
        fn test_headsink_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::HeatsinkPad,
            ));
            assert_code!(function, "%TA.AperFunction,HeatsinkPad*%\n");
        }

        #[test]
        fn test_via_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ViaPad,
            ));
            assert_code!(function, "%TA.AperFunction,ViaPad*%\n");
        }

        #[test]
        fn test_test_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::TestPad,
            ));
            assert_code!(function, "%TA.AperFunction,TestPad*%\n");
        }

        #[test]
        fn test_castellated_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::CastellatedPad,
            ));
            assert_code!(function, "%TA.AperFunction,CastellatedPad*%\n");
        }

        #[test]
        fn test_fiducial_pad() {
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
        }

        #[test]
        fn test_thermal_relief_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ThermalReliefPad,
            ));
            assert_code!(function, "%TA.AperFunction,ThermalReliefPad*%\n");
        }

        #[test]
        fn test_washer_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::WasherPad,
            ));
            assert_code!(function, "%TA.AperFunction,WasherPad*%\n");
        }

        #[test]
        fn test_anti_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::AntiPad,
            ));
            assert_code!(function, "%TA.AperFunction,AntiPad*%\n");
        }

        #[test]
        fn test_other_pad() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::OtherPad("CustomPad".to_string()),
            ));
            assert_code!(function, "%TA.AperFunction,OtherPad,CustomPad*%\n");
        }

        #[test]
        fn test_conductor() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Conductor,
            ));
            assert_code!(function, "%TA.AperFunction,Conductor*%\n");
        }

        #[test]
        fn test_etched_component() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::EtchedComponent,
            ));
            assert_code!(function, "%TA.AperFunction,EtchedComponent*%\n");
        }

        #[test]
        fn test_non_conductor() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::NonConductor,
            ));
            assert_code!(function, "%TA.AperFunction,NonConductor*%\n");
        }

        #[test]
        fn test_copper_balancing() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::CopperBalancing,
            ));
            assert_code!(function, "%TA.AperFunction,CopperBalancing*%\n");
        }

        #[test]
        fn test_border() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Border,
            ));
            assert_code!(function, "%TA.AperFunction,Border*%\n");
        }

        #[test]
        fn test_other_copper() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::OtherCopper("CustomCopper".to_string()),
            ));
            assert_code!(function, "%TA.AperFunction,OtherCopper,CustomCopper*%\n");
        }

        //
        // "All data layers"
        //

        #[test]
        fn test_profile() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Profile,
            ));
            assert_code!(function, "%TA.AperFunction,Profile*%\n");
        }

        #[test]
        fn test_material() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Material,
            ));
            assert_code!(function, "%TA.AperFunction,Material*%\n");
        }

        #[test]
        fn test_non_material() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::NonMaterial,
            ));
            assert_code!(function, "%TA.AperFunction,NonMaterial*%\n");
        }

        #[test]
        fn test_other() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Other("CustomFunction".to_string()),
            ));
            assert_code!(function, "%TA.AperFunction,Other,CustomFunction*%\n");
        }

        //
        // "Component layers"
        //

        #[test]
        fn test_component_main() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentMain,
            ));
            assert_code!(function, "%TA.AperFunction,ComponentMain*%\n");
        }

        #[test]
        fn test_component_outline() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentOutline(ComponentOutline::Body),
            ));
            assert_code!(function, "%TA.AperFunction,ComponentOutline,Body*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentOutline(ComponentOutline::Lead2Lead),
            ));
            assert_code!(function, "%TA.AperFunction,ComponentOutline,Lead2Lead*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentOutline(ComponentOutline::Footprint),
            ));
            assert_code!(function, "%TA.AperFunction,ComponentOutline,Footprint*%\n");

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentOutline(ComponentOutline::Courtyard),
            ));
            assert_code!(function, "%TA.AperFunction,ComponentOutline,Courtyard*%\n");
        }

        #[test]
        fn test_component_pin() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::ComponentPin,
            ));
            assert_code!(function, "%TA.AperFunction,ComponentPin*%\n");
        }

        //
        // "2024.05 - 8.4 - Deprecated attribute values"
        //

        #[test]
        fn test_slot_deprecated() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Slot,
            ));
            assert_code!(function, "%TA.AperFunction,Slot*%\n");
        }
        #[test]
        fn test_cutout_deprecated() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::CutOut,
            ));
            assert_code!(function, "%TA.AperFunction,CutOut*%\n");
        }

        #[test]
        fn test_cavity_deprecated() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Cavity,
            ));
            assert_code!(function, "%TA.AperFunction,Cavity*%\n");
        }

        #[test]
        fn test_drawing_deprecated() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::ApertureFunction(
                ApertureFunction::Drawing,
            ));
            assert_code!(function, "%TA.AperFunction,Drawing*%\n");
        }

        mod user_defined_attribute {
            use super::*;

            #[test]
            fn test_non_standard() {
                let function = ExtendedCode::ApertureAttribute(ApertureAttribute::UserDefined {
                    name: "NonStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there is no '.' prefix, spaces are not trimmed
                assert_code!(function, "%TANonStandardAttribute,Value 1 , Value 2*%\n");
            }

            #[test]
            fn test_unsupported_standard() {
                let function = ExtendedCode::ApertureAttribute(ApertureAttribute::UserDefined {
                    name: ".UnsupportedStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there *is* a '.' prefix, spaces are not trimmed
                assert_code!(
                    function,
                    "%TA.UnsupportedStandardAttribute,Value 1 , Value 2*%\n"
                );
            }
        }
    }

    mod drill_tolerance {
        use super::*;

        #[test]
        fn test_attribute() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::DrillTolerance {
                plus: 1.0,
                minus: 2.0,
            });
            assert_code!(function, "%TA.DrillTolerance,1,2*%\n");
        }
    }

    mod flash_text {
        use super::*;
        #[test]
        fn test_attribute() {
            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::FlashText {
                text: "Test".to_string(),
                mode: TextMode::Characters,
                mirroring: Some(TextMirroring::Readable),
                font: Some("Font Name".to_string()),
                size: Some(10),
                comment: Some("A Comment".to_string()),
            });
            assert_code!(
                function,
                "%TA.FlashText,Test,C,R,Font Name,10,A Comment*%\n"
            );

            let function = ExtendedCode::ApertureAttribute(ApertureAttribute::FlashText {
                text: "Test".to_string(),
                mode: TextMode::BarCode,
                mirroring: Some(TextMirroring::Mirrored),
                font: None,
                size: None,
                comment: None,
            });
            // 2024.05 - 5.6.12 .FlashText - "An empty field means that the corresponding meta-data is not specified."
            assert_code!(function, "%TA.FlashText,Test,B,M,,,*%\n");
        }
    }

    mod object_attribute {
        use super::*;

        mod net {
            use super::*;
            #[test]
            fn test_none() {
                let function = ExtendedCode::ObjectAttribute(ObjectAttribute::Net(Net::None));
                assert_code!(function, "%TO.N,*%\n");
            }

            #[test]
            fn test_not_connected() {
                let function =
                    ExtendedCode::ObjectAttribute(ObjectAttribute::Net(Net::NotConnected));
                assert_code!(function, "%TO.N,N/C*%\n");
            }

            #[test]
            fn test_connected() {
                let function =
                    ExtendedCode::ObjectAttribute(ObjectAttribute::Net(Net::Connected(vec![
                        "Net1".to_string(),
                        "Net2".to_string(),
                        "Net3".to_string(),
                    ])));
                assert_code!(function, "%TO.N,Net1,Net2,Net3*%\n");
            }
        }

        mod pin {
            use super::*;

            #[test]
            fn test_pin() {
                let function = ExtendedCode::ObjectAttribute(ObjectAttribute::Pin(Pin {
                    refdes: "U1".to_string(),
                    name: "1".to_string(),
                    function: None,
                }));
                assert_code!(function, "%TO.P,U1,1*%\n");
            }

            #[test]
            fn test_pin_with_functin() {
                let function = ExtendedCode::ObjectAttribute(ObjectAttribute::Pin(Pin {
                    refdes: "Q1".to_string(),
                    name: "EP".to_string(),
                    function: Some("Thermal pad".to_string()),
                }));
                assert_code!(function, "%TO.P,Q1,EP,Thermal pad*%\n");
            }
        }

        mod ref_des {
            use super::*;

            #[test]
            fn test_component() {
                let function =
                    ExtendedCode::ObjectAttribute(ObjectAttribute::Component("R1".to_string()));
                assert_code!(function, "%TO.C,R1*%\n");
            }
        }

        mod component_characteristics {
            use super::*;

            macro_rules! test_string {
                ($test:ident, $cc:ident, $value:literal) => {
                    #[test]
                    fn $test() {
                        let string = "A String".to_string();
                        let func = ExtendedCode::ObjectAttribute(
                            ObjectAttribute::ComponentCharacteristics(
                                ComponentCharacteristics::$cc(string.clone()),
                            ),
                        );
                        assert_code!(func, &format!("%TO.{},{}*%\n", $value, string));
                    }
                };
            }

            macro_rules! test_decimal {
                ($test:ident, $cc:ident, $value:literal) => {
                    #[test]
                    fn $test() {
                        let decimal = 100.00;
                        let func = ExtendedCode::ObjectAttribute(
                            ObjectAttribute::ComponentCharacteristics(
                                ComponentCharacteristics::$cc(decimal),
                            ),
                        );
                        assert_code!(func, &format!("%TO.{},{}*%\n", $value, decimal));
                    }
                };
            }

            test_decimal!(test_rotation, Rotation, "CRot");
            test_string!(test_manufacturer, Manufacturer, "CMfr");
            test_string!(test_mpn, MPN, "CMPN");
            test_string!(test_val, Value, "CVal");

            mod mount_type {
                use super::*;

                #[test]
                fn test_through_hole() {
                    let function =
                        ExtendedCode::ObjectAttribute(ObjectAttribute::ComponentCharacteristics(
                            ComponentCharacteristics::Mount(ComponentMounting::ThroughHole),
                        ));
                    assert_code!(function, "%TO.CMnt,TH*%\n");
                }

                #[test]
                fn test_smd() {
                    let function =
                        ExtendedCode::ObjectAttribute(ObjectAttribute::ComponentCharacteristics(
                            ComponentCharacteristics::Mount(ComponentMounting::SMD),
                        ));
                    assert_code!(function, "%TO.CMnt,SMD*%\n");
                }

                #[test]
                fn test_press_fit() {
                    let function =
                        ExtendedCode::ObjectAttribute(ObjectAttribute::ComponentCharacteristics(
                            ComponentCharacteristics::Mount(ComponentMounting::PressFit),
                        ));
                    assert_code!(function, "%TO.CMnt,Pressfit*%\n");
                }

                #[test]
                fn test_press_other() {
                    let function =
                        ExtendedCode::ObjectAttribute(ObjectAttribute::ComponentCharacteristics(
                            ComponentCharacteristics::Mount(ComponentMounting::Other),
                        ));
                    assert_code!(function, "%TO.CMnt,Other*%\n");
                }
            }

            test_string!(test_footprint, Footprint, "CFtp");
            test_string!(test_packagename, PackageName, "CPgN");
            test_string!(test_packagedescription, PackageDescription, "CPgD");
            test_decimal!(test_height, Height, "CHgt");
            test_string!(test_libraryname, LibraryName, "CLbN");
            test_string!(test_librarydescription, LibraryDescription, "CLbD");

            #[test]
            fn test_supplier() {
                let function =
                    ExtendedCode::ObjectAttribute(ObjectAttribute::ComponentCharacteristics(
                        ComponentCharacteristics::Supplier(vec![
                            SupplierPart {
                                supplier_name: "Supplier Name 1".to_string(),
                                supplier_part_reference: "Reference 1".to_string(),
                            },
                            SupplierPart {
                                supplier_name: " Supplier Name 2 ".to_string(),
                                supplier_part_reference: "Reference 2".to_string(),
                            },
                        ]),
                    ));
                // NOTE spaces are not trimmed
                assert_code!(
                    function,
                    "%TO.CSup,Supplier Name 1,Reference 1, Supplier Name 2 ,Reference 2*%\n"
                );
            }
        }

        mod user_defined_attribute {
            use super::*;

            #[test]
            fn test_non_standard() {
                let function = ExtendedCode::ObjectAttribute(ObjectAttribute::UserDefined {
                    name: "NonStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there is no '.' prefix, spaces are not trimmed
                assert_code!(function, "%TONonStandardAttribute,Value 1 , Value 2*%\n");
            }

            #[test]
            fn test_unsupported_standard() {
                let function = ExtendedCode::ObjectAttribute(ObjectAttribute::UserDefined {
                    name: ".UnsupportedStandardAttribute".to_string(),
                    values: vec!["Value 1 ".to_string(), " Value 2".to_string()],
                });
                // NOTE there *is* a '.' prefix, spaces are not trimmed
                assert_code!(
                    function,
                    "%TO.UnsupportedStandardAttribute,Value 1 , Value 2*%\n"
                );
            }
        }
    }

    #[test]
    fn test_mirror_image() {
        let value = ExtendedCode::MirrorImage(ImageMirroring::None);
        assert_code!(value, "%MI*%\n");
        let value = ExtendedCode::MirrorImage(ImageMirroring::A);
        assert_code!(value, "%MIA1*%\n");
        let value = ExtendedCode::MirrorImage(ImageMirroring::B);
        assert_code!(value, "%MIB1*%\n");
        let value = ExtendedCode::MirrorImage(ImageMirroring::AB);
        assert_code!(value, "%MIA1B1*%\n");
    }

    #[test]
    fn test_offset_image() {
        let value = ExtendedCode::OffsetImage(ImageOffset { a: 0.0, b: 0.0 });
        assert_code!(value, "%OF*%\n");
        let value = ExtendedCode::OffsetImage(ImageOffset {
            a: 99999.99999,
            b: 0.0,
        });
        assert_code!(value, "%OFA99999.99999*%\n");
        let value = ExtendedCode::OffsetImage(ImageOffset {
            a: 0.0,
            b: 99999.99999,
        });
        assert_code!(value, "%OFB99999.99999*%\n");
        let value = ExtendedCode::OffsetImage(ImageOffset {
            a: -99999.99999,
            b: -99999.99999,
        });
        assert_code!(value, "%OFA-99999.99999B-99999.99999*%\n");
    }

    #[test]
    fn test_scale_image() {
        let value = ExtendedCode::ScaleImage(ImageScaling { a: 0.0, b: 0.0 });
        assert_code!(value, "%SF*%\n");
        let value = ExtendedCode::ScaleImage(ImageScaling {
            a: 999.99999,
            b: 0.0,
        });
        assert_code!(value, "%SFA999.99999*%\n");
        let value = ExtendedCode::ScaleImage(ImageScaling {
            a: 0.0,
            b: 999.99999,
        });
        assert_code!(value, "%SFB999.99999*%\n");
        let value = ExtendedCode::ScaleImage(ImageScaling {
            a: -999.99999,
            b: -999.99999,
        });
        assert_code!(value, "%SFA-999.99999B-999.99999*%\n");
    }

    #[test]
    fn test_rotate_image() {
        let value = ExtendedCode::RotateImage(ImageRotation::None);
        assert_code!(value, "%IR0*%\n");
        let value = ExtendedCode::RotateImage(ImageRotation::CCW_90);
        assert_code!(value, "%IR90*%\n");
        let value = ExtendedCode::RotateImage(ImageRotation::CCW_180);
        assert_code!(value, "%IR180*%\n");
        let value = ExtendedCode::RotateImage(ImageRotation::CCW_270);
        assert_code!(value, "%IR270*%\n");
    }

    #[test]
    fn test_image_polarity() {
        let value = ExtendedCode::ImagePolarity(ImagePolarity::Positive);
        assert_code!(value, "%IPPOS*%\n");
        let value = ExtendedCode::ImagePolarity(ImagePolarity::Negative);
        assert_code!(value, "%IPNEG*%\n");
    }

    #[test]
    fn test_axis_select() {
        let value = ExtendedCode::AxisSelect(AxisSelect::AXBY);
        assert_code!(value, "%ASAXBY*%\n");
        let value = ExtendedCode::AxisSelect(AxisSelect::AYBX);
        assert_code!(value, "%ASAYBX*%\n");
    }

    #[test]
    fn test_image_name() {
        let value = ExtendedCode::ImageName(ImageName {
            name: "PANEL_1".to_string(),
        });
        assert_code!(value, "%INPANEL_1*%\n");
    }
}
