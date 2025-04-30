//! Aperture Macros.

use std::convert::From;
use std::io::Write;

use crate::errors::{GerberError, GerberResult};
use crate::traits::PartialGerberCode;

#[derive(Debug, Clone, PartialEq)]
pub struct ApertureMacro {
    pub name: String,
    pub content: Vec<MacroContent>,
}

impl ApertureMacro {
    pub fn new<S: Into<String>>(name: S) -> Self {
        ApertureMacro {
            name: name.into(),
            content: Vec::new(),
        }
    }

    pub fn add_content<C>(mut self, c: C) -> Self
    where
        C: Into<MacroContent>,
    {
        self.content.push(c.into());
        self
    }

    pub fn add_content_mut<C>(&mut self, c: C)
    where
        C: Into<MacroContent>,
    {
        self.content.push(c.into());
    }
}

impl<W: Write> PartialGerberCode<W> for ApertureMacro {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if self.content.is_empty() {
            return Err(GerberError::MissingDataError(
                "There must be at least 1 content element in an aperture macro".into(),
            ));
        }
        writeln!(writer, "AM{}*", self.name)?;
        let mut first = true;
        for content in &self.content {
            if first {
                first = false;
            } else {
                writeln!(writer)?;
            }
            content.serialize_partial(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A macro decimal can either be an f64 or a variable placeholder.
pub enum MacroDecimal {
    /// A decimal value.
    Value(f64),
    /// A variable placeholder.
    Variable(u32),
    /// An expression placeholder, e.g. "$3+($4/2)" or "$1x4". 
    /// 
    /// Gerber spec (4.5.4.2 Arithmetic Expressions) says:
    /// "The standard arithmetic precedence rules apply". i.e. *not* Rust/C precedence rules.
    /// Allowed elements: "integer and decimal constants, other variables, arithmetic operators and the brackets '(' and ')'"
    /// Allowed operators: '+', '-', 'x' (lower case), '/'.
    Expression(String),
}

impl MacroDecimal {
    fn is_negative(&self) -> bool {
        match *self {
            MacroDecimal::Value(v) => v < 0.0,
            MacroDecimal::Variable(_) => false,
            MacroDecimal::Expression(_) => false,
        }
    }
}

impl From<f32> for MacroDecimal {
    fn from(val: f32) -> Self {
        MacroDecimal::Value(val as f64)
    }
}

impl From<f64> for MacroDecimal {
    fn from(val: f64) -> Self {
        MacroDecimal::Value(val)
    }
}

impl<W: Write> PartialGerberCode<W> for MacroDecimal {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            MacroDecimal::Value(ref v) => write!(writer, "{}", v)?,
            MacroDecimal::Variable(ref v) => write!(writer, "${}", v)?,
            MacroDecimal::Expression(ref v) => write!(writer, "{}", v)?,
        };
        Ok(())
    }
}

/// The gerber specification (2021.02 - 2024.05) 3.4 Data Types does not define a boolean
/// However, there are various places where they are used in macros whey there are defined
/// as having a 0 or 1 value.  Such as the 'exposure' flag.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroBoolean {
    Value(bool),
    Variable(u32),
    Expression(String),
}

impl<W: Write> PartialGerberCode<W> for MacroBoolean {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            MacroBoolean::Value(ref v) => write!(writer, "{}", *v as u8)?,
            MacroBoolean::Variable(ref v) => write!(writer, "${}", v)?,
            MacroBoolean::Expression(ref v) => write!(writer, "{}", v)?,
        };
        Ok(())
    }
}

impl From<MacroDecimal> for MacroBoolean {
    fn from(value: MacroDecimal) -> Self {
        match value {
            MacroDecimal::Value(decimal) => Self::Value(decimal == 1.0),
            MacroDecimal::Variable(variable) => Self::Variable(variable),
            MacroDecimal::Expression(expressions) => Self::Expression(expressions),
        }
    }
}

#[test]
fn test_macro_boolean_from_decimal() {
    assert_eq!(MacroBoolean::from(MacroDecimal::Value(1.0)), MacroBoolean::Value(true));
    assert_eq!(MacroBoolean::from(MacroDecimal::Value(0.0)), MacroBoolean::Value(false));
    assert_eq!(MacroBoolean::from(MacroDecimal::Variable(42)), MacroBoolean::Variable(42));
    assert_eq!(MacroBoolean::from(MacroDecimal::Expression("$1x$2".to_string())), MacroBoolean::Expression("$1x$2".to_string()));
}

/// Gerber specification (2021.02 - 2024.05) 3.4.1 Integers
/// "Integers must fit in a 32-bit signed integer"
#[derive(Debug, Clone, PartialEq)]
pub enum MacroInteger {
    Value(u32),
    Variable(u32),
    Expression(String),
}

impl<W: Write> PartialGerberCode<W> for MacroInteger {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            MacroInteger::Value(ref v) => write!(writer, "{}", v)?,
            MacroInteger::Variable(ref v) => write!(writer, "${}", v)?,
            MacroInteger::Expression(ref v) => write!(writer, "{}", v)?,
        };
        Ok(())
    }
}

/// Gerber specification (2021.02 - 2024.05) 4.5.1.1 Primitives / Overview
/// "Except for the comment all the parameters can be a decimal, integer, macro variables or an arithmetic expression"
/// 
/// However, AFAICT (DC), it doesn't make sense for a the Vertices of the Outline to be a macro variable or expression.
/// Only a literal makes sense since the amount of co-ordinate arguments for the macro cannot change after it is defined
/// *and* the last point of an outline has to be the same as the first point.
///
/// FUTURE report this specification inconsistency with `UCamco` and provide a support reference or link here.
///
/// Fortunately for us the [`OutlinePrimitive`] doesn't need to store the amount of vertices since it uses a [`Vec`] to store the points.
///
#[derive(Debug, Clone, PartialEq)]
pub enum MacroContent {
    // Primitives
    Circle(CirclePrimitive),
    VectorLine(VectorLinePrimitive),
    CenterLine(CenterLinePrimitive),
    Outline(OutlinePrimitive),
    Polygon(PolygonPrimitive),
    Moire(MoirePrimitive),
    Thermal(ThermalPrimitive),

    // Variables
    VariableDefinition(VariableDefinition),

    // Comment
    Comment(String),
}

impl<W: Write> PartialGerberCode<W> for MacroContent {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            MacroContent::Circle(ref c) => c.serialize_partial(writer)?,
            MacroContent::VectorLine(ref vl) => vl.serialize_partial(writer)?,
            MacroContent::CenterLine(ref cl) => cl.serialize_partial(writer)?,
            MacroContent::Outline(ref o) => o.serialize_partial(writer)?,
            MacroContent::Polygon(ref p) => p.serialize_partial(writer)?,
            MacroContent::Moire(ref m) => m.serialize_partial(writer)?,
            MacroContent::Thermal(ref t) => t.serialize_partial(writer)?,
            MacroContent::Comment(ref s) => write!(writer, "0 {}*", &s)?,
            MacroContent::VariableDefinition(ref v) => v.serialize_partial(writer)?,
        };
        Ok(())
    }
}

macro_rules! impl_into {
    ($target:ty, $from:ty, $choice:expr) => {
        impl From<$from> for $target {
            fn from(val: $from) -> $target {
                $choice(val)
            }
        }
    };
}

impl_into!(MacroContent, CirclePrimitive, MacroContent::Circle);
impl_into!(MacroContent, VectorLinePrimitive, MacroContent::VectorLine);
impl_into!(MacroContent, CenterLinePrimitive, MacroContent::CenterLine);
impl_into!(MacroContent, OutlinePrimitive, MacroContent::Outline);
impl_into!(MacroContent, PolygonPrimitive, MacroContent::Polygon);
impl_into!(MacroContent, MoirePrimitive, MacroContent::Moire);
impl_into!(MacroContent, ThermalPrimitive, MacroContent::Thermal);
impl_into!(
    MacroContent,
    VariableDefinition,
    MacroContent::VariableDefinition
);

impl<T: Into<String>> From<T> for MacroContent {
    fn from(val: T) -> Self {
        MacroContent::Comment(val.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CirclePrimitive {
    /// Exposure off/on
    pub exposure: MacroBoolean,

    /// Diameter, a decimal >= 0
    pub diameter: MacroDecimal,

    /// X and Y coordinates of center position, decimals
    pub center: (MacroDecimal, MacroDecimal),

    /// Rotation angle.
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates.
    ///
    /// The rotation modifier is optional. The default is no rotation. (We
    /// recommend always to set the angle explicitly.
    pub angle: Option<MacroDecimal>,
}

impl CirclePrimitive {
    pub fn new(diameter: MacroDecimal) -> Self {
        CirclePrimitive {
            exposure: MacroBoolean::Value(true),
            diameter,
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            angle: None,
        }
    }

    pub fn centered_at(mut self, center: (MacroDecimal, MacroDecimal)) -> Self {
        self.center = center;
        self
    }

    pub fn with_exposure(mut self, exposure: MacroBoolean) -> Self {
        self.exposure = exposure;
        self
    }

    #[deprecated(since = "0.4.0", note = "Use `with_exposure` instead")]
    pub fn exposure_on(mut self, exposure: bool) -> Self {
        self.with_exposure(MacroBoolean::Value(exposure))
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = Some(angle);
        self
    }
}

impl<W: Write> PartialGerberCode<W> for CirclePrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "1,")?;
        self.exposure.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.diameter.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.1.serialize_partial(writer)?;
        if let Some(ref a) = self.angle {
            write!(writer, ",")?;
            a.serialize_partial(writer)?;
        }
        write!(writer, "*")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorLinePrimitive {
    /// Exposure off/on
    pub exposure: MacroBoolean,

    /// Line width, a decimal >= 0
    pub width: MacroDecimal,

    /// X and Y coordinates of start point, decimals
    pub start: (MacroDecimal, MacroDecimal),

    /// X and Y coordinates of end point, decimals
    pub end: (MacroDecimal, MacroDecimal),

    /// Rotation angle of the vector line primitive
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates.
    pub angle: MacroDecimal,
}

impl VectorLinePrimitive {
    pub fn new(start: (MacroDecimal, MacroDecimal), end: (MacroDecimal, MacroDecimal)) -> Self {
        VectorLinePrimitive {
            exposure: MacroBoolean::Value(true),
            width: MacroDecimal::Value(0.0),
            start,
            end,
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn with_exposure(mut self, exposure: MacroBoolean) -> Self {
        self.exposure = exposure;
        self
    }

    #[deprecated(since = "0.4.0", note = "Use `with_exposure` instead")]
    pub fn exposure_on(mut self, exposure: bool) -> Self {
        self.with_exposure(MacroBoolean::Value(exposure))
    }

    pub fn with_width(mut self, width: MacroDecimal) -> Self {
        self.width = width;
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for VectorLinePrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "20,")?;
        self.exposure.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.width.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.start.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.start.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.end.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.end.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CenterLinePrimitive {
    /// Exposure off/on (0/1)
    pub exposure: MacroBoolean,

    /// Rectangle dimensions (width/height)
    pub dimensions: (MacroDecimal, MacroDecimal),

    /// X and Y coordinates of center point, decimals
    pub center: (MacroDecimal, MacroDecimal),

    /// Rotation angle
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. (0, 0) point
    /// of macro coordinates.
    pub angle: MacroDecimal,
}

impl CenterLinePrimitive {
    pub fn new(dimensions: (MacroDecimal, MacroDecimal)) -> Self {
        CenterLinePrimitive {
            exposure: MacroBoolean::Value(true),
            dimensions,
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn with_exposure(mut self, exposure: MacroBoolean) -> Self {
        self.exposure = exposure;
        self
    }

    #[deprecated(since = "0.4.0", note = "Use `with_exposure` instead")]
    pub fn exposure_on(mut self, exposure: bool) -> Self {
        self.with_exposure(MacroBoolean::Value(exposure))
    }

    pub fn centered_at(mut self, center: (MacroDecimal, MacroDecimal)) -> Self {
        self.center = center;
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for CenterLinePrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "21,")?;
        self.exposure.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.dimensions.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.dimensions.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutlinePrimitive {
    /// Exposure off/on (0/1)
    pub exposure: MacroBoolean,

    /// Vector of coordinate pairs.
    ///
    /// The last coordinate pair must be equal to the first coordinate pair!
    pub points: Vec<(MacroDecimal, MacroDecimal)>,

    /// Rotation angle of the outline primitive
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates.
    pub angle: MacroDecimal,
}

impl OutlinePrimitive {
    pub fn new() -> Self {
        OutlinePrimitive {
            exposure: MacroBoolean::Value(true),
            points: Vec::new(),
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn from_points(points: Vec<(MacroDecimal, MacroDecimal)>) -> Self {
        let mut outline_prim = Self::new();
        outline_prim.points = points;
        outline_prim
    }

    pub fn add_point(mut self, point: (MacroDecimal, MacroDecimal)) -> Self {
        self.points.push(point);
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for OutlinePrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        // Points invariants
        if self.points.len() < 2 {
            return Err(GerberError::MissingDataError(
                "There must be at least 1 subsequent point in an outline".into(),
            ));
        }
        if self.points.len() > 5001 {
            return Err(GerberError::RangeError(
                "The maximum number of subsequent points in an outline is 5000".into(),
            ));
        }
        if self.points[0] != self.points[self.points.len() - 1] {
            return Err(GerberError::RangeError(
                "The last point must be equal to the first point".into(),
            ));
        }

        write!(writer, "4,")?;
        self.exposure.serialize_partial(writer)?;
        writeln!(writer, ",{},", self.points.len() - 1)?;

        for &(ref x, ref y) in &self.points {
            x.serialize_partial(writer)?;
            write!(writer, ",")?;
            y.serialize_partial(writer)?;
            writeln!(writer, ",")?;
        }
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A polygon primitive is a regular polygon defined by the number of vertices,
/// the center point and the diameter of the circumscribed circle.
pub struct PolygonPrimitive {
    /// Exposure off/on (0/1)
    pub exposure: MacroBoolean,

    /// Number of vertices n, 3 <= n <= 12
    pub vertices: MacroInteger,

    /// X and Y coordinates of center point, decimals
    pub center: (MacroDecimal, MacroDecimal),

    /// Diameter of the circumscribed circle, a decimal >= 0
    pub diameter: MacroDecimal,

    /// Rotation angle of the polygon primitive
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates. The first vertex is on the positive X-axis
    /// through the center point when the rotation angle is zero.
    ///
    /// Note: Rotation is only allowed if the primitive center point coincides
    /// with the origin of the macro definition.
    pub angle: MacroDecimal,
}

impl PolygonPrimitive {
    pub fn new(vertices: MacroInteger) -> Self {
        PolygonPrimitive {
            exposure: MacroBoolean::Value(true),
            vertices,
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            diameter: MacroDecimal::Value(0.0),
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn with_exposure(mut self, exposure: MacroBoolean) -> Self {
        self.exposure = exposure;
        self
    }

    #[deprecated(since = "0.4.0", note = "Use `with_exposure` instead")]
    pub fn exposure_on(mut self, exposure: bool) -> Self {
        self.with_exposure(MacroBoolean::Value(exposure))
    }

    pub fn centered_at(mut self, center: (MacroDecimal, MacroDecimal)) -> Self {
        self.center = center;
        self
    }

    pub fn with_diameter(mut self, diameter: MacroDecimal) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for PolygonPrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        // Vertice count invariants
        if matches!(self.vertices, MacroInteger::Value(value) if value < 3) {
            return Err(GerberError::MissingDataError(
                "There must be at least 3 vertices in a polygon".into(),
            ));
        }
        if matches!(self.vertices, MacroInteger::Value(value) if value > 12) {
            return Err(GerberError::RangeError(
                "The maximum number of vertices in a polygon is 12".into(),
            ));
        }
        if self.diameter.is_negative() {
            return Err(GerberError::RangeError(
                "The diameter must not be negative".into(),
            ));
        }
        write!(writer, "5,")?;
        self.exposure.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.vertices.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.diameter.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

/// The moiré primitive is a cross hair centered on concentric rings (annuli).
/// Exposure is always on.
#[derive(Debug, Clone, PartialEq)]
pub struct MoirePrimitive {
    /// X and Y coordinates of center point, decimals
    pub center: (MacroDecimal, MacroDecimal),

    /// Outer diameter of outer concentric ring, a decimal >= 0
    pub diameter: MacroDecimal,

    /// Ring thickness, a decimal >= 0
    pub ring_thickness: MacroDecimal,

    /// Gap between rings, a decimal >= 0
    pub gap: MacroDecimal,

    /// Maximum number of rings
    pub max_rings: u32,

    /// Cross hair thickness, a decimal >= 0
    pub cross_hair_thickness: MacroDecimal,

    /// Cross hair length, a decimal >= 0
    pub cross_hair_length: MacroDecimal,

    /// Rotation angle of the moiré primitive
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates.
    ///
    /// Note: Rotation is only allowed if the primitive center point coincides
    /// with the origin of the macro definition.
    pub angle: MacroDecimal,
}

impl MoirePrimitive {
    pub fn new() -> Self {
        MoirePrimitive {
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            diameter: MacroDecimal::Value(0.0),
            ring_thickness: MacroDecimal::Value(0.0),
            gap: MacroDecimal::Value(0.0),
            max_rings: 1,
            cross_hair_thickness: MacroDecimal::Value(0.0),
            cross_hair_length: MacroDecimal::Value(0.0),
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn centered_at(mut self, center: (MacroDecimal, MacroDecimal)) -> Self {
        self.center = center;
        self
    }

    pub fn with_diameter(mut self, diameter: MacroDecimal) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn with_rings_max(mut self, max_rings: u32) -> Self {
        self.max_rings = max_rings;
        self
    }

    pub fn with_ring_thickness(mut self, thickness: MacroDecimal) -> Self {
        self.ring_thickness = thickness;
        self
    }

    pub fn with_gap(mut self, gap: MacroDecimal) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_cross_thickness(mut self, thickness: MacroDecimal) -> Self {
        self.cross_hair_thickness = thickness;
        self
    }

    pub fn with_cross_length(mut self, length: MacroDecimal) -> Self {
        self.cross_hair_length = length;
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for MoirePrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        // Decimal invariants
        if self.diameter.is_negative() {
            return Err(GerberError::RangeError(
                "Outer diameter of a moiré may not be negative".into(),
            ));
        }
        if self.ring_thickness.is_negative() {
            return Err(GerberError::RangeError(
                "Ring thickness of a moiré may not be negative".into(),
            ));
        }
        if self.gap.is_negative() {
            return Err(GerberError::RangeError(
                "Gap of a moiré may not be negative".into(),
            ));
        }
        if self.cross_hair_thickness.is_negative() {
            return Err(GerberError::RangeError(
                "Cross hair thickness of a moiré may not be negative".into(),
            ));
        }
        if self.cross_hair_length.is_negative() {
            return Err(GerberError::RangeError(
                "Cross hair length of a moiré may not be negative".into(),
            ));
        }
        write!(writer, "6,")?;
        self.center.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.diameter.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.ring_thickness.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.gap.serialize_partial(writer)?;
        write!(writer, ",{},", self.max_rings)?;
        self.cross_hair_thickness.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.cross_hair_length.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

/// The thermal primitive is a ring (annulus) interrupted by four gaps.
/// Exposure is always on.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalPrimitive {
    /// X and Y coordinates of center point, decimals
    pub center: (MacroDecimal, MacroDecimal),

    /// Outer diameter, a decimal > inner diameter
    pub outer_diameter: MacroDecimal,

    /// Inner diameter, a decimal >= 0
    pub inner_diameter: MacroDecimal,

    /// Gap thickness, a decimal < (outer diameter) / sqrt(2)
    pub gap: MacroDecimal,

    /// Rotation angle of the thermal primitive
    ///
    /// The rotation angle is specified by a decimal, in degrees. The primitive
    /// is rotated around the origin of the macro definition, i.e. the (0, 0)
    /// point of macro coordinates. The gaps are on the X and Y axes through
    /// the center when the rotation angle is zero
    ///
    /// Note: Rotation is only allowed if the primitive center point coincides
    /// with the origin of the macro definition.
    pub angle: MacroDecimal,
}

impl ThermalPrimitive {
    pub fn new(inner: MacroDecimal, outer: MacroDecimal, gap: MacroDecimal) -> Self {
        ThermalPrimitive {
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            outer_diameter: outer,
            inner_diameter: inner,
            gap,
            angle: MacroDecimal::Value(0.0),
        }
    }

    pub fn centered_at(mut self, center: (MacroDecimal, MacroDecimal)) -> Self {
        self.center = center;
        self
    }

    pub fn with_angle(mut self, angle: MacroDecimal) -> Self {
        self.angle = angle;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for ThermalPrimitive {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        // Decimal invariants
        if self.inner_diameter.is_negative() {
            return Err(GerberError::RangeError(
                "Inner diameter of a thermal may not be negative".into(),
            ));
        }
        write!(writer, "7,")?;
        self.center.0.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.center.1.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.outer_diameter.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.inner_diameter.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.gap.serialize_partial(writer)?;
        write!(writer, ",")?;
        self.angle.serialize_partial(writer)?;
        write!(writer, "*")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDefinition {
    number: u32,
    expression: String,
}

impl VariableDefinition {
    pub fn new(number: u32, expr: &str) -> Self {
        VariableDefinition {
            number,
            expression: expr.into(),
        }
    }
}

impl<W: Write> PartialGerberCode<W> for VariableDefinition {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "${}={}*", self.number, self.expression)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;

    use crate::traits::PartialGerberCode;

    use super::MacroDecimal::{Value, Variable, Expression};
    use super::*;

    macro_rules! assert_partial_code {
        ($obj:expr, $expected:expr) => {
            let mut buf = BufWriter::new(Vec::new());
            $obj.serialize_partial(&mut buf)
                .expect("Could not generate Gerber code");
            let bytes = buf.into_inner().unwrap();
            let code = String::from_utf8(bytes).unwrap();
            assert_eq!(&code, $expected);
        };
    }

    #[test]
    fn test_circle_primitive_codegen() {
        let with_angle = CirclePrimitive {
            exposure: MacroBoolean::Value(true),
            diameter: Value(1.5),
            center: (Value(0.), Value(0.)),
            angle: Some(Value(0.)),
        };
        assert_partial_code!(with_angle, "1,1,1.5,0,0,0*");
        let no_angle = CirclePrimitive {
            exposure: MacroBoolean::Value(false),
            diameter: Value(99.9),
            center: (Value(1.1), Value(2.2)),
            angle: None,
        };
        assert_partial_code!(no_angle, "1,0,99.9,1.1,2.2*");
    }

    #[test]
    fn test_vector_line_primitive_codegen() {
        let line = VectorLinePrimitive {
            exposure: MacroBoolean::Value(true),
            width: Value(0.9),
            start: (Value(0.), Value(0.45)),
            end: (Value(12.), Value(0.45)),
            angle: Value(0.),
        };
        assert_partial_code!(line, "20,1,0.9,0,0.45,12,0.45,0*");
    }

    #[test]
    fn test_center_line_primitive_codegen() {
        let line = CenterLinePrimitive {
            exposure: MacroBoolean::Value(true),
            dimensions: (Value(6.8), Value(1.2)),
            center: (Value(3.4), Value(0.6)),
            angle: Value(30.0),
        };
        assert_partial_code!(line, "21,1,6.8,1.2,3.4,0.6,30*");
    }

    #[test]
    fn test_outline_primitive_codegen() {
        let line = OutlinePrimitive {
            exposure: MacroBoolean::Value(true),
            points: vec![
                (Value(0.1), Value(0.1)),
                (Value(0.5), Value(0.1)),
                (Value(0.5), Value(0.5)),
                (Value(0.1), Value(0.5)),
                (Value(0.1), Value(0.1)),
            ],
            angle: Value(0.0),
        };
        assert_partial_code!(
            line,
            "4,1,4,\n0.1,0.1,\n0.5,0.1,\n0.5,0.5,\n0.1,0.5,\n0.1,0.1,\n0*"
        );
    }

    #[test]
    fn test_polygon_primitive_codegen() {
        let line = PolygonPrimitive {
            exposure: MacroBoolean::Value(true),
            vertices: MacroInteger::Value(8),
            center: (Value(1.5), Value(2.0)),
            diameter: Value(8.0),
            angle: Value(0.0),
        };
        assert_partial_code!(line, "5,1,8,1.5,2,8,0*");
    }

    #[test]
    fn test_moire_primitive_codegen() {
        let line = MoirePrimitive {
            center: (Value(0.0), Value(0.0)),
            diameter: Value(5.0),
            ring_thickness: Value(0.5),
            gap: Value(0.5),
            max_rings: 2,
            cross_hair_thickness: Value(0.1),
            cross_hair_length: Value(6.0),
            angle: Value(0.0),
        };
        assert_partial_code!(line, "6,0,0,5,0.5,0.5,2,0.1,6,0*");
    }

    #[test]
    fn test_thermal_primitive_codegen() {
        let line = ThermalPrimitive {
            center: (Value(0.0), Value(0.0)),
            outer_diameter: Value(8.0),
            inner_diameter: Value(6.5),
            gap: Value(1.0),
            angle: Value(45.0),
        };
        assert_partial_code!(line, "7,0,0,8,6.5,1,45*");
    }

    #[test]
    fn test_aperture_macro_codegen() {
        let am = ApertureMacro::new("CRAZY")
            .add_content(MacroContent::Thermal(ThermalPrimitive {
                center: (Value(0.0), Value(0.0)),
                outer_diameter: Value(0.08),
                inner_diameter: Value(0.055),
                gap: Value(0.0125),
                angle: Value(45.0),
            }))
            .add_content(MacroContent::Moire(MoirePrimitive {
                center: (Value(0.0), Value(0.0)),
                diameter: Value(0.125),
                ring_thickness: Value(0.01),
                gap: Value(0.01),
                max_rings: 3,
                cross_hair_thickness: Value(0.003),
                cross_hair_length: Value(0.150),
                angle: Value(0.0),
            }));
        assert_partial_code!(
            am,
            "AMCRAZY*\n7,0,0,0.08,0.055,0.0125,45*\n6,0,0,0.125,0.01,0.01,3,0.003,0.15,0*"
        );
    }

    /// This test should use at least one each of the enum variants in [`MacroDecimal`]
    #[test]
    fn test_codegen_with_variable() {
        let line = VectorLinePrimitive {
            exposure: MacroBoolean::Value(true),
            width: Variable(0),
            start: (Variable(1), 0.45.into()),
            end: (Value(12.), Expression("$2x4".to_string())),
            angle: Variable(3),
        };
        assert_partial_code!(line, "20,1,$0,$1,0.45,12,$2x4,$3*");
    }

    #[test]
    fn test_macro_decimal_into() {
        let a = Value(1.0);
        let b: MacroDecimal = 1.0.into();
        assert_eq!(a, b);
        let c = Variable(1);
        let d = Variable(1);
        assert_eq!(c, d);
    }

    #[test]
    fn test_comment_codegen() {
        let comment = MacroContent::Comment("hello world".to_string());
        assert_partial_code!(comment, "0 hello world*");
    }

    #[test]
    fn test_variable_definition_codegen() {
        let var = VariableDefinition {
            number: 17,
            expression: "$40+2".to_string(),
        };
        assert_partial_code!(var, "$17=$40+2*");
    }

    #[test]
    fn test_macrocontent_from_into() {
        let a = MacroContent::Comment("hello".into());
        let b: MacroContent = "hello".to_string().into();
        let c: MacroContent = "hello".into();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn test_circle_primitive_new() {
        let c1 = CirclePrimitive::new(Value(3.0)).centered_at((Value(5.0), Value(0.0)));
        let c2 = CirclePrimitive {
            exposure: MacroBoolean::Value(true),
            diameter: Value(3.0),
            center: (Value(5.0), Value(0.0)),
            angle: None,
        };
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_vectorline_primitive_new() {
        let vl1 = VectorLinePrimitive::new((Value(0.0), Value(5.3)), (Value(3.9), Value(8.5)))
            .with_angle(Value(38.0));
        let vl2 = VectorLinePrimitive {
            exposure: MacroBoolean::Value(true),
            width: Value(0.0),
            start: (Value(0.0), Value(5.3)),
            end: (Value(3.9), Value(8.5)),
            angle: Value(38.0),
        };
        assert_eq!(vl1, vl2);
    }

    #[test]
    fn test_centerline_primitive_new() {
        let cl1 = CenterLinePrimitive::new((Value(3.0), Value(4.5))).exposure_on(false);
        let cl2 = CenterLinePrimitive {
            exposure: MacroBoolean::Value(false),
            dimensions: (Value(3.0), Value(4.5)),
            center: (Value(0.0), Value(0.0)),
            angle: Value(0.0),
        };
        assert_eq!(cl1, cl2);
    }

    #[test]
    fn test_outline_primitive_new() {
        let op1 = OutlinePrimitive::new()
            .add_point((Value(0.0), Value(0.0)))
            .add_point((Value(2.0), Value(2.0)))
            .add_point((Value(-2.0), Value(-2.0)))
            .add_point((Value(0.0), Value(0.0)));

        let pts = vec![
            (Value(0.0), Value(0.0)),
            (Value(2.0), Value(2.0)),
            (Value(-2.0), Value(-2.0)),
            (Value(0.0), Value(0.0)),
        ];

        let op2 = OutlinePrimitive {
            exposure: MacroBoolean::Value(true),
            points: pts,
            angle: Value(0.0),
        };
        assert_eq!(op1, op2);
    }

    #[test]
    fn test_polygon_primitive_new() {
        let pp1 = PolygonPrimitive::new(MacroInteger::Value(5))
            .with_angle(Value(98.0))
            .with_diameter(Value(5.3))
            .centered_at((Value(1.0), Value(1.0)));
        let pp2 = PolygonPrimitive {
            exposure: MacroBoolean::Value(true),
            vertices: MacroInteger::Value(5),
            angle: Value(98.0),
            diameter: Value(5.3),
            center: (Value(1.0), Value(1.0)),
        };
        assert_eq!(pp1, pp2);
    }

    #[test]
    fn test_moire_primitive_new() {
        let mp1 = MoirePrimitive::new()
            .with_diameter(Value(3.0))
            .with_ring_thickness(Value(0.05))
            .with_cross_thickness(Value(0.01))
            .with_cross_length(Value(0.5))
            .with_rings_max(3);
        let mp2 = MoirePrimitive {
            center: (MacroDecimal::Value(0.0), MacroDecimal::Value(0.0)),
            diameter: MacroDecimal::Value(3.0),
            ring_thickness: MacroDecimal::Value(0.05),
            gap: MacroDecimal::Value(0.0),
            max_rings: 3,
            cross_hair_thickness: MacroDecimal::Value(0.01),
            cross_hair_length: MacroDecimal::Value(0.5),
            angle: MacroDecimal::Value(0.0),
        };
        assert_eq!(mp1, mp2);
    }

    #[test]
    fn test_thermal_primitive_new() {
        let tp1 = ThermalPrimitive::new(Value(1.0), Value(2.0), Value(1.5)).with_angle(Value(87.3));
        let tp2 = ThermalPrimitive {
            inner_diameter: Value(1.0),
            outer_diameter: Value(2.0),
            gap: Value(1.5),
            angle: Value(87.3),
            center: (Value(0.0), Value(0.0)),
        };
        assert_eq!(tp1, tp2);
    }

    #[test]
    fn test_variabledefinition_new() {
        let vd1 = VariableDefinition::new(3, "Test!");
        let vd2 = VariableDefinition {
            number: 3,
            expression: "Test!".into(),
        };
        assert_eq!(vd1, vd2);
    }
}
