//! Attributes.

use std::io::Write;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::GerberResult;
use crate::traits::PartialGerberCode;

// FileAttribute

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAttribute {
    Part(Part),
    FileFunction(FileFunction),
    FilePolarity(FilePolarity),
    GenerationSoftware(GenerationSoftware),
    CreationDate(DateTime<Utc>),
    ProjectId {
        id: String,
        guid: Uuid,
        revision: String,
    },
    Md5(String),
    UserDefined {
        name: String,
        value: Vec<String>,
    },
}

impl<W: Write> PartialGerberCode<W> for FileAttribute {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            FileAttribute::Part(ref part) => {
                write!(writer, "Part,")?;
                part.serialize_partial(writer)?;
            }
            FileAttribute::FileFunction(ref function) => {
                write!(writer, "FileFunction,")?;
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
                    FileFunction::Profile(ref plating) => {
                        write!(writer, "Profile,")?;
                        plating.serialize_partial(writer)?;
                    }
                    FileFunction::Soldermask { ref pos, ref index } => {
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
                    _ => unimplemented!(),
                }
            }
            FileAttribute::GenerationSoftware(ref gs) => {
                write!(writer, "GenerationSoftware,")?;
                gs.serialize_partial(writer)?;
            }
            FileAttribute::FilePolarity(ref p) => {
                write!(writer, "FilePolarity,")?;
                p.serialize_partial(writer)?;
            }
            FileAttribute::Md5(ref hash) => write!(writer, "MD5,{}", hash)?,
            _ => unimplemented!(),
        };
        Ok(())
    }
}

// ApertureAttribute

#[derive(Debug, Clone, PartialEq)]
pub enum ApertureAttribute {
    ApertureFunction(ApertureFunction),
    CustomAttribute(String, Option<String>),
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
                    ApertureFunction::ComponentDrill { press_fit } => {
                        write!(writer, "ComponentDrill")?;
                        if matches!(press_fit, Some(true)) {
                            write!(writer, ",PressFit")?;
                        }
                    }
                    ApertureFunction::MechanicalDrill { function } => {
                        write!(writer, "MechanicalDrill")?;
                        if let Some(ref function) = function {
                            write!(writer, ",")?;
                            match function {
                                DrillFunction::Tooling => {
                                    write!(writer, "Tooling")?;
                                }
                                DrillFunction::BreakOut => {
                                    write!(writer, "BreakOut")?;
                                }
                                DrillFunction::Other => {
                                    write!(writer, "Other")?;
                                }
                            }
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
                        write!(writer, "ComponentOutline")?;
                        if let Some(value) = value {
                            write!(writer, ",")?;
                            value.serialize_partial(writer)?;
                        }
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
            ApertureAttribute::CustomAttribute(name, value) => {
                write!(writer, "{}", name)?;
                if let Some(value) = value {
                    write!(writer, ",{}", value)?;
                }
            }
        }
        Ok(())
    }
}

// Part

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
}

impl<W: Write> PartialGerberCode<W> for Position {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Position::Top => write!(writer, "Top")?,
            Position::Bottom => write!(writer, "Bot")?,
        };
        Ok(())
    }
}

// ExtendedPosition

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedPosition {
    Top,
    Inner,
    Bottom,
}

impl<W: Write> PartialGerberCode<W> for ExtendedPosition {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            ExtendedPosition::Top => write!(writer, "Top")?,
            ExtendedPosition::Inner => write!(writer, "Inr")?,
            ExtendedPosition::Bottom => write!(writer, "Bot")?,
        };
        Ok(())
    }
}

// CopperType

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopperType {
    Plane,
    Signal,
    Mixed,
    Hatched,
}

impl<W: Write> PartialGerberCode<W> for CopperType {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            CopperType::Plane => write!(writer, "Plane")?,
            CopperType::Signal => write!(writer, "Signal")?,
            CopperType::Mixed => write!(writer, "Mixed")?,
            CopperType::Hatched => write!(writer, "Hatched")?,
        };
        Ok(())
    }
}

// Drill

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drill {
    ThroughHole,
    Blind,
    Buried,
}

// DrillRouteType

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrillRouteType {
    Drill,
    Route,
    Mixed,
}

// Profile

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Profile {
    Plated,
    NonPlated,
}

impl<W: Write> PartialGerberCode<W> for Profile {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Profile::Plated => write!(writer, "P")?,
            Profile::NonPlated => write!(writer, "NP")?,
        };
        Ok(())
    }
}

// FileFunction

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFunction {
    Copper {
        layer: i32,
        pos: ExtendedPosition,
        copper_type: Option<CopperType>,
    },
    Soldermask {
        pos: Position,
        index: Option<i32>,
    },
    Legend {
        pos: Position,
        index: Option<i32>,
    },
    Goldmask {
        pos: Position,
        index: Option<i32>,
    },
    Silvermask {
        pos: Position,
        index: Option<i32>,
    },
    Tinmask {
        pos: Position,
        index: Option<i32>,
    },
    Carbonmask {
        pos: Position,
        index: Option<i32>,
    },
    Peelablesoldermask {
        pos: Position,
        index: Option<i32>,
    },
    Glue {
        pos: Position,
        index: Option<i32>,
    },
    Viatenting(Position),
    Viafill,
    Heatsink(Position),
    Paste(Position),
    KeepOut(Position),
    Pads(Position),
    Scoring(Position),
    Plated {
        from_layer: i32,
        to_layer: i32,
        drill: Drill,
        label: Option<DrillRouteType>,
    },
    NonPlated {
        from_layer: i32,
        to_layer: i32,
        drill: Drill,
        label: Option<DrillRouteType>,
    },
    Profile(Profile),
    Drillmap,
    FabricationDrawing,
    ArrayDrawing,
    AssemblyDrawing(Position),
    Drawing(String),
    Other(String),
}

// FilePolarity

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilePolarity {
    Positive,
    Negative,
}

impl<W: Write> PartialGerberCode<W> for FilePolarity {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            FilePolarity::Positive => write!(writer, "Positive")?,
            FilePolarity::Negative => write!(writer, "Negative")?,
        };
        Ok(())
    }
}

// GenerationSoftware

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApertureFunction {
    // "Drill and rout layers"
    ViaDrill(Option<IPC4761ViaProtection>),
    BackDrill,
    ComponentDrill { press_fit: Option<bool> },
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
    ComponentOutline(Option<ComponentOutline>),
    ComponentPin,

    // 2024.05 - 8.4 - "Deprecated attribute values"
    Slot,
    CutOut,
    Cavity,
    Drawing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IPC4761ViaProtection {
    Ia,
    Ib,
    IIa,
    IIb,
    IIIa,
    IIIb,
    IVa,
    IVb,
    V,
    VI,
    VII,
    None,
}

impl<W: Write> PartialGerberCode<W> for IPC4761ViaProtection {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        let code = match self {
            IPC4761ViaProtection::Ia => "Ia",
            IPC4761ViaProtection::Ib => "Ib",
            IPC4761ViaProtection::IIa => "IIa",
            IPC4761ViaProtection::IIb => "IIb",
            IPC4761ViaProtection::IIIa => "IIIa",
            IPC4761ViaProtection::IIIb => "IIIb",
            IPC4761ViaProtection::IVa => "IVa",
            IPC4761ViaProtection::IVb => "IVb",
            IPC4761ViaProtection::V => "V",
            IPC4761ViaProtection::VI => "VI",
            IPC4761ViaProtection::VII => "VII",
            IPC4761ViaProtection::None => "None",
        };
        write!(writer, "{}", code)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentOutline {
    Body,
    Lead2Lead,
    Footprint,
    Courtyard,
}

impl<W: Write> PartialGerberCode<W> for ComponentOutline {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        let code = match self {
            ComponentOutline::Body => "Body",
            ComponentOutline::Lead2Lead => "Lead2Lead",
            ComponentOutline::Footprint => "Footprint",
            ComponentOutline::Courtyard => "Courtyard",
        };
        write!(writer, "{}", code)?;
        Ok(())
    }
}
// DrillFunction

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrillFunction {
    BreakOut,
    Tooling,
    Other,
}

// SmdPadType

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmdPadType {
    CopperDefined,
    SoldermaskDefined,
}

impl<W: Write> PartialGerberCode<W> for SmdPadType {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            SmdPadType::CopperDefined => write!(writer, "CuDef")?,
            SmdPadType::SoldermaskDefined => write!(writer, "SMDef")?,
        };
        Ok(())
    }
}

// FiducialScope

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FiducialScope {
    Local,
    Global,
    Panel,
}

impl<W: Write> PartialGerberCode<W> for FiducialScope {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            FiducialScope::Global => write!(writer, "Global")?,
            FiducialScope::Local => write!(writer, "Local")?,
            FiducialScope::Panel => write!(writer, "Panel")?,
        };
        Ok(())
    }
}
