//! Attributes.

use std::io::Write;
use strum_macros::{IntoStaticStr, VariantArray, VariantNames};
use uuid::Uuid;

use crate::errors::GerberResult;
use crate::traits::PartialGerberCode;
use crate::GerberDate;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ident {
    // Aka 'Guid'
    Uuid(Uuid),
    Name(String),
}

impl<W: Write> PartialGerberCode<W> for Ident {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            Ident::Uuid(guid) => {
                write!(writer, "{}", guid)?;
            }
            Ident::Name(value) => {
                write!(writer, "{}", value)?;
            }
        }

        Ok(())
    }
}

// FileAttribute

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileAttribute {
    /// "%TF.Part,(Single|Array|FabricationPanel|Coupon|Other,<mandatory description>)>*%
    Part(Part),
    /// "%TF.FileFunction,<args>*%
    FileFunction(FileFunction),
    /// "%TF.FilePolarity,(Positive|Negative)>*%
    FilePolarity(FilePolarity),
    /// "%TF.SameCoordinates[,<ident>]*%"
    SameCoordinates(Option<Ident>),
    /// "%TF.CreationDate,2015-02-23T15:59:51+01:00*%" ISO8601 + TZ
    CreationDate(GerberDate),
    /// "%TF.GenerationSoftware,<vendor>,<application>,<version>*%"
    GenerationSoftware(GenerationSoftware),
    /// "%TF.ProjectId,<Name>,<GUID>,<Revision>*%"
    ProjectId {
        id: String,
        uuid: Uuid,
        revision: String,
    },
    /// "%TF.MD5,6ab9e892830469cdff7e3e346331d404*%"
    Md5(String),
    UserDefined {
        name: String,
        values: Vec<String>,
    },
}

impl<W: Write> PartialGerberCode<W> for FileAttribute {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            FileAttribute::Part(ref part) => {
                write!(writer, ".Part,")?;
                part.serialize_partial(writer)?;
            }
            FileAttribute::FileFunction(ref function) => {
                write!(writer, ".FileFunction,")?;
                match function {
                    FileFunction::Copper {
                        ref layer,
                        ref pos,
                        ref copper_type,
                    } => {
                        write!(writer, "Copper,L{},", layer)?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref t) = *copper_type {
                            write!(writer, ",")?;
                            t.serialize_partial(writer)?;
                        }
                    }
                    FileFunction::Plated {
                        from_layer,
                        to_layer,
                        drill,
                        label,
                    } => {
                        write!(writer, "Plated,{},{},", from_layer, to_layer)?;
                        drill.serialize_partial(writer)?;
                        if let Some(ref l) = label {
                            write!(writer, ",")?;
                            l.serialize_partial(writer)?;
                        }
                    }
                    FileFunction::NonPlated {
                        from_layer,
                        to_layer,
                        drill,
                        label,
                    } => {
                        write!(writer, "NonPlated,{},{},", from_layer, to_layer)?;
                        drill.serialize_partial(writer)?;
                        if let Some(ref l) = label {
                            write!(writer, ",")?;
                            l.serialize_partial(writer)?;
                        }
                    }
                    FileFunction::Profile(ref plating) => {
                        write!(writer, "Profile")?;
                        if let Some(ref plating) = plating {
                            write!(writer, ",")?;
                            plating.serialize_partial(writer)?;
                        }
                    }
                    FileFunction::KeepOut(ref pos) => {
                        write!(writer, "Keepout,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::SolderMask { ref pos, ref index } => {
                        write!(writer, "Soldermask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::Legend { ref pos, ref index } => {
                        write!(writer, "Legend,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::Component { layer, pos } => {
                        write!(writer, "Component,L{},", layer)?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::Paste(pos) => {
                        write!(writer, "Paste,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::Glue(pos) => {
                        write!(writer, "Glue,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::CarbonMask { pos, index } => {
                        write!(writer, "Carbonmask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::GoldMask { pos, index } => {
                        write!(writer, "Goldmask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::HeatsinkMask { pos, index } => {
                        write!(writer, "Heatsinkmask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::PeelableMask { pos, index } => {
                        write!(writer, "Peelablemask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::SilverMask { pos, index } => {
                        write!(writer, "Silvermask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::TinMask { pos, index } => {
                        write!(writer, "Tinmask,")?;
                        pos.serialize_partial(writer)?;
                        if let Some(ref i) = index {
                            write!(writer, ",{}", *i)?;
                        }
                    }
                    FileFunction::DepthRoute(pos) => {
                        write!(writer, "Depthrout,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::VCut(pos) => {
                        write!(writer, "Vcut")?;
                        if let Some(pos) = pos {
                            write!(writer, ",")?;
                            pos.serialize_partial(writer)?;
                        }
                    }
                    FileFunction::ViaFill => {
                        write!(writer, "Viafill")?;
                    }
                    FileFunction::Pads(pos) => {
                        write!(writer, "Pads,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::Other(value) => {
                        write!(writer, "Other,{}", value)?;
                    }

                    // "Drawing layers"
                    FileFunction::DrillMap => {
                        write!(writer, "Drillmap")?;
                    }
                    FileFunction::FabricationDrawing => {
                        write!(writer, "FabricationDrawing")?;
                    }
                    FileFunction::VCutMap => {
                        write!(writer, "Vcutmap")?;
                    }
                    FileFunction::AssemblyDrawing(pos) => {
                        write!(writer, "AssemblyDrawing,")?;
                        pos.serialize_partial(writer)?;
                    }
                    FileFunction::ArrayDrawing => {
                        write!(writer, "ArrayDrawing")?;
                    }
                    FileFunction::OtherDrawing(value) => {
                        write!(writer, "OtherDrawing,{}", value)?;
                    }
                }
            }
            FileAttribute::FilePolarity(ref p) => {
                write!(writer, ".FilePolarity,")?;
                p.serialize_partial(writer)?;
            }
            FileAttribute::SameCoordinates(ident) => {
                write!(writer, ".SameCoordinates")?;
                if let Some(ident) = ident {
                    write!(writer, ",")?;
                    ident.serialize_partial(writer)?;
                }
            }
            FileAttribute::CreationDate(date) => {
                write!(writer, ".CreationDate,{}", date.to_rfc3339())?;
            }
            FileAttribute::GenerationSoftware(ref gs) => {
                write!(writer, ".GenerationSoftware,")?;
                gs.serialize_partial(writer)?;
            }
            FileAttribute::ProjectId { id, uuid, revision } => {
                write!(writer, ".ProjectId,{},{},{}", id, uuid, revision)?;
            }
            FileAttribute::Md5(ref hash) => write!(writer, ".MD5,{}", hash)?,
            FileAttribute::UserDefined { name, values } => {
                write!(writer, "{}", name)?;
                for value in values {
                    write!(writer, ",{}", value)?;
                }
            }
        };
        Ok(())
    }
}

// TextMode
#[derive(Debug, Copy, Clone, PartialEq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TextMode {
    #[strum(serialize = "B")]
    BarCode,
    #[strum(serialize = "C")]
    Characters,
}

impl_partial_gerber_code_via_strum!(TextMode);

// TextMirroring
#[derive(Debug, Copy, Clone, PartialEq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TextMirroring {
    #[strum(serialize = "R")]
    Readable,
    #[strum(serialize = "M")]
    Mirrored,
}

impl_partial_gerber_code_via_strum!(TextMirroring);

// ApertureAttribute

#[derive(Debug, Clone, PartialEq)]
pub enum ApertureAttribute {
    ApertureFunction(ApertureFunction),
    DrillTolerance {
        plus: f64,
        minus: f64,
    },
    FlashText {
        text: String,
        mode: TextMode,
        mirroring: Option<TextMirroring>,
        font: Option<String>,
        size: Option<i32>,
        comment: Option<String>,
    },
    UserDefined {
        name: String,
        values: Vec<String>,
    },
}

impl<W: Write> PartialGerberCode<W> for ApertureAttribute {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            ApertureAttribute::ApertureFunction(ref af) => {
                write!(writer, ".AperFunction,")?;
                match af {
                    // "Drill and rout layers"
                    ApertureFunction::ViaDrill(ref value) => {
                        write!(writer, "ViaDrill")?;
                        if let Some(value) = value {
                            write!(writer, ",")?;
                            value.serialize_partial(writer)?;
                        }
                    }
                    ApertureFunction::BackDrill => {
                        write!(writer, "BackDrill")?;
                    }
                    ApertureFunction::ComponentDrill { ref function } => {
                        write!(writer, "ComponentDrill")?;
                        if let Some(function) = function {
                            write!(writer, ",")?;
                            function.serialize_partial(writer)?;
                        }
                    }
                    ApertureFunction::MechanicalDrill { function } => {
                        write!(writer, "MechanicalDrill")?;
                        if let Some(ref function) = function {
                            write!(writer, ",")?;
                            function.serialize_partial(writer)?;
                        }
                    }
                    ApertureFunction::CastellatedDrill => {
                        write!(writer, "CastellatedDrill")?;
                    }
                    ApertureFunction::OtherDrill(ref value) => {
                        write!(writer, "OtherDrill,{}", value)?;
                    }

                    // "Copper layers"
                    ApertureFunction::ComponentPad => {
                        write!(writer, "ComponentPad")?;
                    }
                    ApertureFunction::SmdPad(ref value) => {
                        write!(writer, "SMDPad,")?;
                        value.serialize_partial(writer)?;
                    }
                    ApertureFunction::BgaPad(ref value) => {
                        write!(writer, "BGAPad,")?;
                        value.serialize_partial(writer)?;
                    }
                    ApertureFunction::ConnectorPad => {
                        write!(writer, "ConnectorPad")?;
                    }
                    ApertureFunction::HeatsinkPad => {
                        write!(writer, "HeatsinkPad")?;
                    }
                    ApertureFunction::ViaPad => {
                        write!(writer, "ViaPad")?;
                    }
                    ApertureFunction::TestPad => {
                        write!(writer, "TestPad")?;
                    }
                    ApertureFunction::CastellatedPad => {
                        write!(writer, "CastellatedPad")?;
                    }
                    ApertureFunction::FiducialPad(ref value) => {
                        write!(writer, "FiducialPad,")?;
                        value.serialize_partial(writer)?;
                    }
                    ApertureFunction::ThermalReliefPad => {
                        write!(writer, "ThermalReliefPad")?;
                    }
                    ApertureFunction::WasherPad => {
                        write!(writer, "WasherPad")?;
                    }
                    ApertureFunction::AntiPad => {
                        write!(writer, "AntiPad")?;
                    }
                    ApertureFunction::OtherPad(ref value) => {
                        write!(writer, "OtherPad,{}", value)?;
                    }
                    ApertureFunction::Conductor => {
                        write!(writer, "Conductor")?;
                    }
                    ApertureFunction::EtchedComponent => {
                        write!(writer, "EtchedComponent")?;
                    }
                    ApertureFunction::NonConductor => {
                        write!(writer, "NonConductor")?;
                    }
                    ApertureFunction::CopperBalancing => {
                        write!(writer, "CopperBalancing")?;
                    }
                    ApertureFunction::Border => {
                        write!(writer, "Border")?;
                    }
                    ApertureFunction::OtherCopper(ref value) => {
                        write!(writer, "OtherCopper,{}", value)?;
                    }

                    // "Component layers"
                    ApertureFunction::ComponentMain => {
                        write!(writer, "ComponentMain")?;
                    }
                    ApertureFunction::ComponentOutline(ref value) => {
                        write!(writer, "ComponentOutline,")?;
                        value.serialize_partial(writer)?;
                    }
                    ApertureFunction::ComponentPin => {
                        write!(writer, "ComponentPin")?;
                    }

                    // "All data layers"
                    ApertureFunction::Profile => {
                        write!(writer, "Profile")?;
                    }
                    ApertureFunction::NonMaterial => {
                        write!(writer, "NonMaterial")?;
                    }
                    ApertureFunction::Material => {
                        write!(writer, "Material")?;
                    }
                    ApertureFunction::Other(value) => {
                        write!(writer, "Other,{}", value)?;
                    }

                    // 2024.05 - 8.4 - "Deprecated attribute values"
                    ApertureFunction::Slot => {
                        write!(writer, "Slot")?;
                    }
                    ApertureFunction::Cavity => {
                        write!(writer, "Cavity")?;
                    }
                    ApertureFunction::CutOut => {
                        write!(writer, "CutOut")?;
                    }
                    ApertureFunction::Drawing => {
                        write!(writer, "Drawing")?;
                    }
                }
            }
            ApertureAttribute::DrillTolerance { plus, minus } => {
                write!(writer, ".DrillTolerance,{},{}", plus, minus)?;
            }
            ApertureAttribute::FlashText {
                text,
                mode,
                mirroring,
                font,
                size,
                comment,
            } => {
                write!(writer, ".FlashText,{},", text)?;
                mode.serialize_partial(writer)?;
                write!(writer, ",")?;
                mirroring.serialize_partial(writer)?;
                write!(writer, ",")?;
                if let Some(font) = font {
                    write!(writer, "{}", font)?;
                }
                write!(writer, ",")?;
                if let Some(size) = size {
                    write!(writer, "{}", size)?;
                }
                write!(writer, ",")?;
                if let Some(comment) = comment {
                    write!(writer, "{}", comment)?;
                }
            }
            ApertureAttribute::UserDefined { name, values } => {
                write!(writer, "{}", name)?;
                for value in values {
                    write!(writer, ",{}", value)?;
                }
            }
        }
        Ok(())
    }
}

// Part

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Part {
    /// Single PCB
    Single,
    /// A.k.a. customer panel, assembly panel, shipping panel, biscuit
    Array,
    /// A.k.a. working panel, production panel
    FabricationPanel,
    /// A test coupon
    Coupon,
    /// None of the above
    Other(String),
}

impl<W: Write> PartialGerberCode<W> for Part {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Part::Single => write!(writer, "Single")?,
            Part::Array => write!(writer, "Array")?,
            Part::FabricationPanel => write!(writer, "FabricationPanel")?,
            Part::Coupon => write!(writer, "Coupon")?,
            Part::Other(ref description) => write!(writer, "Other,{}", description)?,
        };
        Ok(())
    }
}

// Position

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    IntoStaticStr,
    VariantNames,
    VariantArray,
)]
#[strum(serialize_all = "PascalCase")]
pub enum Position {
    Top,
    #[strum(serialize = "Bot")]
    Bottom,
}

impl_partial_gerber_code_via_strum!(Position);

// ExtendedPosition

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum ExtendedPosition {
    Top,
    #[strum(serialize = "Inr")]
    Inner,
    #[strum(serialize = "Bot")]
    Bottom,
}

impl_partial_gerber_code_via_strum!(ExtendedPosition);

// CopperType

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum CopperType {
    Plane,
    Signal,
    Mixed,
    Hatched,
}

impl_partial_gerber_code_via_strum!(CopperType);

// PlatedDrill

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum PlatedDrill {
    #[strum(serialize = "PTH")]
    PlatedThroughHole,
    Blind,
    Buried,
}

impl_partial_gerber_code_via_strum!(PlatedDrill);

// NonPlatedDrill

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum NonPlatedDrill {
    #[strum(serialize = "NPTH")]
    NonPlatedThroughHole,
    Blind,
    Buried,
}

impl_partial_gerber_code_via_strum!(NonPlatedDrill);

// DrillRouteType

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum DrillRouteType {
    Drill,
    #[strum(serialize = "Rout")]
    Route,
    Mixed,
}

impl_partial_gerber_code_via_strum!(DrillRouteType);

// Profile

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
pub enum Profile {
    #[strum(serialize = "P")]
    Plated,
    #[strum(serialize = "NP")]
    NonPlated,
}

impl_partial_gerber_code_via_strum!(Profile);

// FileFunction

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileFunction {
    //
    // "Data layers"
    //
    Copper {
        layer: i32,
        pos: ExtendedPosition,
        copper_type: Option<CopperType>,
    },
    Plated {
        from_layer: i32,
        to_layer: i32,
        drill: PlatedDrill,
        label: Option<DrillRouteType>,
    },
    NonPlated {
        from_layer: i32,
        to_layer: i32,
        drill: NonPlatedDrill,
        label: Option<DrillRouteType>,
    },
    /// Apparently, this should be used instead of `KeepOut` since 2017.11, see "11.15 Revision 2017.11" but this makes no sense
    /// Since keep-out has a `Position` but Profile does not...
    /// Additionally, DipTrace does not specify the 'N/NP', e.g. "%TF.FileFunction,Profile*%"
    Profile(Option<Profile>),
    KeepOut(Position),
    SolderMask {
        pos: Position,
        index: Option<i32>,
    },
    Legend {
        pos: Position,
        index: Option<i32>,
    },
    Component {
        layer: i32,
        pos: Position,
    },
    Paste(Position),
    Glue(Position),
    CarbonMask {
        pos: Position,
        index: Option<i32>,
    },
    GoldMask {
        pos: Position,
        index: Option<i32>,
    },
    HeatsinkMask {
        pos: Position,
        index: Option<i32>,
    },
    PeelableMask {
        pos: Position,
        index: Option<i32>,
    },
    SilverMask {
        pos: Position,
        index: Option<i32>,
    },
    TinMask {
        pos: Position,
        index: Option<i32>,
    },
    DepthRoute(Position),
    VCut(Option<Position>),
    /// Contains the via’s that must be filled (usually with some form of epoxy)
    ViaFill,
    Pads(Position),
    Other(String),

    // "Drawing layers"
    DrillMap,
    FabricationDrawing,
    VCutMap,
    AssemblyDrawing(Position),
    ArrayDrawing,
    OtherDrawing(String),
}

// FilePolarity

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum FilePolarity {
    Positive,
    Negative,
}

impl_partial_gerber_code_via_strum!(FilePolarity);

// GenerationSoftware

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenerationSoftware {
    pub vendor: String,
    pub application: String,
    pub version: Option<String>,
}

impl GenerationSoftware {
    pub fn new<S: Into<String>>(vendor: S, application: S, version: Option<S>) -> Self {
        GenerationSoftware {
            vendor: vendor.into(),
            application: application.into(),
            version: version.map(|s| s.into()),
        }
    }
}

impl<W: Write> PartialGerberCode<W> for GenerationSoftware {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self.version {
            Some(ref v) => write!(writer, "{},{},{}", self.vendor, self.application, v)?,
            None => write!(writer, "{},{}", self.vendor, self.application)?,
        };
        Ok(())
    }
}

/// ApertureFunction
///
/// 2024.05 - 5.6.10 ".AperFunction"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApertureFunction {
    // "Drill and rout layers"
    ViaDrill(Option<IPC4761ViaProtection>),
    BackDrill,
    ComponentDrill { function: Option<ComponentDrill> },
    MechanicalDrill { function: Option<DrillFunction> },
    CastellatedDrill,
    OtherDrill(String),

    // "Copper layers"
    ComponentPad,
    SmdPad(SmdPadType),
    BgaPad(SmdPadType),
    ConnectorPad,
    HeatsinkPad,
    ViaPad,
    TestPad,
    CastellatedPad,
    FiducialPad(FiducialScope),
    ThermalReliefPad,
    WasherPad,
    AntiPad,
    OtherPad(String),
    Conductor,
    EtchedComponent,
    NonConductor,
    CopperBalancing,
    Border,
    OtherCopper(String),

    // "All data layers"
    Profile,
    Material,
    NonMaterial,
    Other(String),

    // "Component layers"
    ComponentMain,
    ComponentOutline(ComponentOutline),
    ComponentPin,

    // 2024.05 - 8.4 - "Deprecated attribute values"
    Slot,
    CutOut,
    Cavity,
    Drawing,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum IPC4761ViaProtection {
    Ia,
    Ib,
    IIa,
    IIb,
    #[strum(serialize = "IIIa")]
    IIIa,
    #[strum(serialize = "IIIb")]
    IIIb,
    IVa,
    IVb,
    V,
    #[strum(serialize = "VI")]
    VI,
    #[strum(serialize = "VII")]
    VII,
    None,
}

impl_partial_gerber_code_via_strum!(IPC4761ViaProtection);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum ComponentOutline {
    Body,
    Lead2Lead,
    Footprint,
    Courtyard,
}

impl_partial_gerber_code_via_strum!(ComponentOutline);

// DrillFunction

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum DrillFunction {
    #[strum(serialize = "Breakout")]
    BreakOut,
    Tooling,
    Other,
}

impl_partial_gerber_code_via_strum!(DrillFunction);

// ComponentDrill

/// 2024.05 spec mismatch warning: Aperture function ".AperFunction.ComponentDill" has "PressFit" (uppercase F) whereas Component Characteristics ".CMnt" has "Pressfit" (lowercase f)"
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
pub enum ComponentDrill {
    #[strum(serialize = "PressFit")]
    PressFit,
}

impl_partial_gerber_code_via_strum!(ComponentDrill);

// SmdPadType

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum SmdPadType {
    #[strum(serialize = "CuDef")]
    CopperDefined,
    #[strum(serialize = "SMDef")]
    SoldermaskDefined,
}

impl_partial_gerber_code_via_strum!(SmdPadType);

// FiducialScope

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "PascalCase")]
pub enum FiducialScope {
    Local,
    Global,
    Panel,
}

impl_partial_gerber_code_via_strum!(FiducialScope);

// ObjectAttribute
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectAttribute {
    Net(Net),
    Pin(Pin),
    /// aka 'RefDes'
    Component(String),
    ComponentCharacteristics(ComponentCharacteristics),
    UserDefined {
        name: String,
        values: Vec<String>,
    },
}

impl<W: Write> PartialGerberCode<W> for ObjectAttribute {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            ObjectAttribute::Net(net) => {
                net.serialize_partial(writer)?;
            }
            ObjectAttribute::Pin(pin) => {
                pin.serialize_partial(writer)?;
            }
            ObjectAttribute::Component(ref_des) => {
                write!(writer, ".C,{}", ref_des)?;
            }
            ObjectAttribute::ComponentCharacteristics(cc) => {
                cc.serialize_partial(writer)?;
            }
            ObjectAttribute::UserDefined { name, values } => {
                write!(writer, "{}", name)?;
                for value in values {
                    write!(writer, ",{}", value)?;
                }
            }
        };
        Ok(())
    }
}

/// ComponentCharacteristics
/// 2024.05 - 5.6.1.6 "Cxxx (Component Characteristics)"
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentCharacteristics {
    /// ".CRot,<decimal>"
    Rotation(f64),
    /// ".CMfr,<field>"
    Manufacturer(String),
    /// ".CMPN,<field>"
    MPN(String),
    /// ".CVal,<field>"
    Value(String),
    /// ".CMnt,(TH|SMD|Pressfit|Other)"
    Mount(ComponentMounting),
    /// ".CFtp,<field>"
    Footprint(String),
    /// ".CPgN,<field>"
    PackageName(String),
    /// ".CPgD,<field>"
    PackageDescription(String),
    /// ".CHgt,<decimal>"
    Height(f64),
    /// ".CLbN,<field>"
    LibraryName(String),
    /// ".CLbD,<field>"
    LibraryDescription(String),
    /// The specification requires at least one supplier part.  Do not add the attribute if there are no supplier parts.
    /// ".CSup,<SN>,<SPN>,{<SN>,<SPN>}"
    Supplier(Vec<SupplierPart>),
}

impl<W: Write> PartialGerberCode<W> for ComponentCharacteristics {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            ComponentCharacteristics::Rotation(rotation) => {
                write!(writer, ".CRot,{}", rotation)?;
            }
            ComponentCharacteristics::Manufacturer(manufacturer) => {
                write!(writer, ".CMfr,{}", manufacturer)?;
            }
            ComponentCharacteristics::MPN(mpn) => {
                write!(writer, ".CMPN,{}", mpn)?;
            }
            ComponentCharacteristics::Value(value) => {
                write!(writer, ".CVal,{}", value)?;
            }
            ComponentCharacteristics::Mount(mount) => {
                write!(writer, ".CMnt,")?;
                mount.serialize_partial(writer)?;
            }
            ComponentCharacteristics::Footprint(footprint) => {
                write!(writer, ".CFtp,{}", footprint)?;
            }
            ComponentCharacteristics::PackageName(package_name) => {
                write!(writer, ".CPgN,{}", package_name)?;
            }
            ComponentCharacteristics::PackageDescription(package_description) => {
                write!(writer, ".CPgD,{}", package_description)?;
            }
            ComponentCharacteristics::Height(height) => {
                write!(writer, ".CHgt,{}", height)?;
            }
            ComponentCharacteristics::LibraryName(library_name) => {
                write!(writer, ".CLbN,{}", library_name)?;
            }
            ComponentCharacteristics::LibraryDescription(library_description) => {
                write!(writer, ".CLbD,{}", library_description)?;
            }
            ComponentCharacteristics::Supplier(values) => {
                write!(writer, ".CSup")?;
                for value in values {
                    write!(writer, ",")?;
                    value.serialize_partial(writer)?;
                }
            }
        }
        Ok(())
    }
}

/// 2024.05 spec mismatch warning: Aperture function ".AperFunction.ComponentDill" has "PressFit" (uppercase F) whereas Component Characteristics ".CMnt" has "Pressfit" (lowercase f)"
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
pub enum ComponentMounting {
    #[strum(serialize = "TH")]
    ThroughHole,
    /// Surface mount device
    #[strum(serialize = "SMD")]
    SMD,
    #[strum(serialize = "Pressfit")]
    PressFit,
    #[strum(serialize = "Other")]
    Other,
}

impl_partial_gerber_code_via_strum!(ComponentMounting);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SupplierPart {
    /// The name of the supplier, e.g. 'Mouser', 'Digikey', 'LCSC', etc.
    pub supplier_name: String,
    /// The spec says 'supplier part name' but using 'reference' is more accurate, as that is what they use to look up the part
    pub supplier_part_reference: String,
}

impl<W: Write> PartialGerberCode<W> for SupplierPart {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(
            writer,
            "{},{}",
            self.supplier_name, self.supplier_part_reference
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Net {
    None,
    NotConnected,
    Connected(Vec<String>),
}

impl<W: Write> PartialGerberCode<W> for Net {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, ".N,")?;
        match self {
            Net::None => {}
            Net::NotConnected => {
                write!(writer, "N/C")?;
            }
            Net::Connected(nets) => {
                write!(writer, "{}", nets.join(","))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub refdes: String,
    /// 2024.05 spec calls this 'number' but a) it's defined as a string, and b) pins like 'EP' (for exposed pad) are not numbers.
    pub name: String,
    pub function: Option<String>,
}

impl<W: Write> PartialGerberCode<W> for Pin {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, ".P,{},{}", self.refdes, self.name)?;
        if let Some(function) = &self.function {
            write!(writer, ",{}", function)?;
        }
        Ok(())
    }
}
