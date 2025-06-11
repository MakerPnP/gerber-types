//! Attributes.

use chrono::{DateTime, FixedOffset};
use std::io::Write;
use uuid::Uuid;

use crate::errors::GerberResult;
use crate::traits::PartialGerberCode;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    CreationDate(DateTime<FixedOffset>),
    /// "%TF.GenerationSoftware,<vendor>,<application>,<version>*%"
    GenerationSoftware(GenerationSoftware),
    /// "%TF.ProjectId,<Name>,<GUID>,<Revision>*%"
    ProjectId {
        id: String,
        guid: Uuid,
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
                        write!(writer, "Profile,")?;
                        plating.serialize_partial(writer)?;
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
            FileAttribute::ProjectId { id, guid, revision } => {
                write!(writer, ".ProjectId,{},{},{}", id, guid, revision)?;
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
#[derive(Debug, Clone, PartialEq)]
pub enum TextMode {
    BarCode,
    Characters,
}

impl<W: Write> PartialGerberCode<W> for TextMode {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            Self::Characters => write!(writer, "C")?,
            Self::BarCode => write!(writer, "B")?,
        }
        Ok(())
    }
}

// TextMirroring
#[derive(Debug, Clone, PartialEq)]
pub enum TextMirroring {
    Readable,
    Mirrored,
}

impl<W: Write> PartialGerberCode<W> for TextMirroring {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            Self::Readable => write!(writer, "R")?,
            Self::Mirrored => write!(writer, "M")?,
        }
        Ok(())
    }
}

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

// PlatedDrill

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatedDrill {
    PlatedThroughHole,
    Blind,
    Buried,
}

impl<W: Write> PartialGerberCode<W> for PlatedDrill {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            PlatedDrill::PlatedThroughHole => write!(writer, "PTH")?,
            PlatedDrill::Blind => write!(writer, "Blind")?,
            PlatedDrill::Buried => write!(writer, "Buried")?,
        }
        Ok(())
    }
}

// NonPlatedDrill

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonPlatedDrill {
    NonPlatedThroughHole,
    Blind,
    Buried,
}

impl<W: Write> PartialGerberCode<W> for NonPlatedDrill {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            NonPlatedDrill::NonPlatedThroughHole => write!(writer, "NPTH")?,
            NonPlatedDrill::Blind => write!(writer, "Blind")?,
            NonPlatedDrill::Buried => write!(writer, "Buried")?,
        }
        Ok(())
    }
}

// DrillRouteType

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrillRouteType {
    Drill,
    Route,
    Mixed,
}

impl<W: Write> PartialGerberCode<W> for DrillRouteType {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            DrillRouteType::Drill => write!(writer, "Drill")?,
            DrillRouteType::Route => write!(writer, "Rout")?,
            DrillRouteType::Mixed => write!(writer, "Mixed")?,
        }
        Ok(())
    }
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
    Profile(Profile),
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

// ObjectAttribute
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectAttribute {
    ComponentCharacteristics(ComponentCharacteristics),
    UserDefined { name: String, values: Vec<String> },
}

impl<W: Write> PartialGerberCode<W> for ObjectAttribute {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
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

// ComponentCharacteristics
// 2024.05 - 5.6.1.6 "Cxxx (Component Characteristics)"
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentMounting {
    ThroughHole,
    /// Surface mount device
    SMD,
    PressFit,
    Other,
}

impl<W: Write> PartialGerberCode<W> for ComponentMounting {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self {
            ComponentMounting::ThroughHole => write!(writer, "TH")?,
            ComponentMounting::SMD => write!(writer, "SMD")?,
            ComponentMounting::PressFit => write!(writer, "Pressfit")?,
            ComponentMounting::Other => write!(writer, "Other")?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
