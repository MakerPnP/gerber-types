//! Extended code types.

use std::io::Write;

use crate::errors::GerberResult;
use crate::traits::PartialGerberCode;
use crate::MacroDecimal;
use strum_macros;
use strum_macros::{IntoStaticStr, VariantArray, VariantNames};

// Unit

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Unit {
    #[strum(serialize = "IN")]
    Inches,
    #[strum(serialize = "MM")]
    Millimeters,
}

impl_partial_gerber_code_via_strum!(Unit);

// ApertureDefinition

#[derive(Debug, Clone, PartialEq)]
pub struct ApertureDefinition {
    pub code: i32,
    pub aperture: Aperture,
}

impl ApertureDefinition {
    pub fn new(code: i32, aperture: Aperture) -> Self {
        ApertureDefinition { code, aperture }
    }
}

impl<W: Write> PartialGerberCode<W> for ApertureDefinition {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "{}", self.code)?;
        self.aperture.serialize_partial(writer)?;
        Ok(())
    }
}

// Aperture

#[derive(Debug, Clone, PartialEq)]
pub enum Aperture {
    Circle(Circle),
    Rectangle(Rectangular),
    Obround(Rectangular),
    Polygon(Polygon),

    /// gerber spec (2024.05) 4.3.1 "AD Command" - "Parameters are decimals."
    ///
    /// Note: this definition conflicts with:
    /// a) the [`MacroBoolean`] which is used for the exposure parameter for macro primitives.
    /// b) the [`MacroInteger`] which is used for the '# vertices' parameter for macro primitives.
    ///
    /// Conversion functions from MacroDecimal to [`MacroBoolean`] & [`MacroInteger`] are required.  
    Macro(String, Option<Vec<MacroDecimal>>),
}

impl<W: Write> PartialGerberCode<W> for Aperture {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Aperture::Circle(ref circle) => {
                write!(writer, "C,")?;
                circle.serialize_partial(writer)?;
            }
            Aperture::Rectangle(ref rectangular) => {
                write!(writer, "R,")?;
                rectangular.serialize_partial(writer)?;
            }
            Aperture::Obround(ref rectangular) => {
                write!(writer, "O,")?;
                rectangular.serialize_partial(writer)?;
            }
            Aperture::Polygon(ref polygon) => {
                write!(writer, "P,")?;
                polygon.serialize_partial(writer)?;
            }
            Aperture::Macro(ref string, ref args) => {
                write!(writer, "{}", string)?;
                if let Some(ref args) = *args {
                    write!(writer, ",")?;
                    for (index, arg) in args.iter().enumerate() {
                        if index > 0 {
                            write!(writer, "X")?;
                        }
                        arg.serialize_partial(writer)?;
                    }
                }
            }
        };
        Ok(())
    }
}

// Circle

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub diameter: f64,
    pub hole_diameter: Option<f64>,
}

impl Circle {
    pub fn new(diameter: f64) -> Self {
        Circle {
            diameter,
            hole_diameter: None,
        }
    }

    pub fn with_hole(diameter: f64, hole_diameter: f64) -> Self {
        Circle {
            diameter,
            hole_diameter: Some(hole_diameter),
        }
    }
}

impl<W: Write> PartialGerberCode<W> for Circle {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self.hole_diameter {
            Some(hole_diameter) => {
                write!(writer, "{}X{}", self.diameter, hole_diameter)?;
            }
            None => write!(writer, "{}", self.diameter)?,
        };
        Ok(())
    }
}

// Rectangular

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangular {
    pub x: f64,
    pub y: f64,
    pub hole_diameter: Option<f64>,
}

impl Rectangular {
    pub fn new(x: f64, y: f64) -> Self {
        Rectangular {
            x,
            y,
            hole_diameter: None,
        }
    }

    pub fn with_hole(x: f64, y: f64, hole_diameter: f64) -> Self {
        Rectangular {
            x,
            y,
            hole_diameter: Some(hole_diameter),
        }
    }
}

impl<W: Write> PartialGerberCode<W> for Rectangular {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match self.hole_diameter {
            Some(hole_diameter) => write!(writer, "{}X{}X{}", self.x, self.y, hole_diameter)?,
            None => write!(writer, "{}X{}", self.x, self.y)?,
        };
        Ok(())
    }
}

// Polygon

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub diameter: f64,
    pub vertices: u8, // 3--12
    pub rotation: Option<f64>,
    pub hole_diameter: Option<f64>,
}

impl Polygon {
    pub fn new(diameter: f64, vertices: u8) -> Self {
        Polygon {
            diameter,
            vertices,
            rotation: None,
            hole_diameter: None,
        }
    }

    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.rotation = Some(angle);
        self
    }

    pub fn with_diameter(mut self, diameter: f64) -> Self {
        self.diameter = diameter;
        self
    }
}

impl<W: Write> PartialGerberCode<W> for Polygon {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match (self.rotation, self.hole_diameter) {
            (Some(rot), Some(hd)) => {
                write!(writer, "{}X{}X{}X{}", self.diameter, self.vertices, rot, hd)?
            }
            (Some(rot), None) => write!(writer, "{}X{}X{}", self.diameter, self.vertices, rot)?,
            (None, Some(hd)) => write!(writer, "{}X{}X0X{}", self.diameter, self.vertices, hd)?,
            (None, None) => write!(writer, "{}X{}", self.diameter, self.vertices)?,
        };
        Ok(())
    }
}

// Polarity

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Polarity {
    #[strum(serialize = "C")]
    Clear,
    #[strum(serialize = "D")]
    Dark,
}

impl_partial_gerber_code_via_strum!(Polarity);

// Mirroring

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Mirroring {
    #[strum(serialize = "N")]
    None,
    X,
    Y,
    XY,
}

impl_partial_gerber_code_via_strum!(Mirroring);

// Scaling

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scaling {
    pub scale: f64,
}

impl<W: Write> PartialGerberCode<W> for Scaling {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "{}", self.scale)?;
        Ok(())
    }
}

// Rotation

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    /// in degrees, counter-clockwise
    pub rotation: f64,
}

impl<W: Write> PartialGerberCode<W> for Rotation {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "{}", self.rotation)?;
        Ok(())
    }
}

// StepAndRepeat

#[derive(Debug, Clone, PartialEq)]
pub enum StepAndRepeat {
    Open {
        repeat_x: u32,
        repeat_y: u32,
        distance_x: f64,
        distance_y: f64,
    },
    Close,
}

impl<W: Write> PartialGerberCode<W> for StepAndRepeat {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            StepAndRepeat::Open {
                repeat_x: rx,
                repeat_y: ry,
                distance_x: dx,
                distance_y: dy,
            } => write!(writer, "X{}Y{}I{}J{}", rx, ry, dx, dy)?,
            StepAndRepeat::Close => {}
        };
        Ok(())
    }
}

// ApertureBlock

#[derive(Debug, Clone, PartialEq)]
pub enum ApertureBlock {
    Open { code: i32 },
    Close,
}

impl<W: Write> PartialGerberCode<W> for ApertureBlock {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            ApertureBlock::Open { code } => write!(writer, "{}", code)?,
            ApertureBlock::Close => {}
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aperture_definition_new() {
        let ad1 = ApertureDefinition::new(10, Aperture::Circle(Circle::new(3.0)));
        let ad2 = ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle::new(3.0)),
        };
        assert_eq!(ad1, ad2);
    }

    #[test]
    fn test_rectangular_new() {
        let r1 = Rectangular::new(2.0, 3.0);
        let r2 = Rectangular {
            x: 2.0,
            y: 3.0,
            hole_diameter: None,
        };
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_rectangular_with_hole() {
        let r1 = Rectangular::with_hole(3.0, 2.0, 1.0);
        let r2 = Rectangular {
            x: 3.0,
            y: 2.0,
            hole_diameter: Some(1.0),
        };
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_circle_new() {
        let c1 = Circle::new(3.0);
        let c2 = Circle {
            diameter: 3.0,
            hole_diameter: None,
        };
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_circle_with_hole() {
        let c1 = Circle::with_hole(3.0, 1.0);
        let c2 = Circle {
            diameter: 3.0,
            hole_diameter: Some(1.0),
        };
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_polygon_new() {
        let p1 = Polygon::new(3.0, 4).with_rotation(45.0);
        let p2 = Polygon {
            diameter: 3.0,
            vertices: 4,
            rotation: Some(45.0),
            hole_diameter: None,
        };
        assert_eq!(p1, p2);
    }

    /// This test is to ensure that the `Unit` enum is hashable.
    #[test]
    fn unit_in_hashmap() {
        let mut map = std::collections::HashMap::new();
        map.insert(Unit::Inches, ());
        map.insert(Unit::Millimeters, ());

        assert_eq!(map.len(), 2);
    }
}

// Image Mirroring

/// Gerber spec 2024.05 8.1.7 "Mirror Image (MI)"
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
pub enum ImageMirroring {
    #[strum(serialize = "")]
    None,
    #[strum(serialize = "A1")]
    A,
    #[strum(serialize = "B1")]
    B,
    #[strum(serialize = "A1B1")]
    AB,
}

impl_partial_gerber_code_via_strum!(ImageMirroring);

impl Default for ImageMirroring {
    fn default() -> Self {
        ImageMirroring::None
    }
}

// Image Rotation

/// Gerber spec 2024.05 8.1.5 "Image Rotation (IR)"
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
#[allow(non_camel_case_types)]
pub enum ImageRotation {
    #[strum(serialize = "0")]
    None,
    #[strum(serialize = "90")]
    CCW_90,
    #[strum(serialize = "180")]
    CCW_180,
    #[strum(serialize = "270")]
    CCW_270,
}

impl_partial_gerber_code_via_strum!(ImageRotation);

impl Default for ImageRotation {
    fn default() -> Self {
        ImageRotation::None
    }
}
// Image Scaling

/// Gerber spec 2024.05 8.1.9 "Scale Factor (SF)"
/// By default, A=X, B=Y, but this changes depending on the axis select command (AS)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageScaling {
    /// scale factor for A axis
    pub a: f64,
    /// scale factor for B axis
    pub b: f64,
}

impl<W: Write> PartialGerberCode<W> for ImageScaling {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if self.a != 0.0 {
            write!(writer, "A{}", self.a)?;
        }
        if self.b != 0.0 {
            write!(writer, "B{}", self.b)?;
        }
        Ok(())
    }
}

impl Default for ImageScaling {
    fn default() -> Self {
        ImageScaling { a: 1.0, b: 1.0 }
    }
}

// Image Offset

/// Gerber spec 2024.05 8.1.8 "Offset (OF)"
/// By default, A=X, B=Y, but this changes depending on the axis select command (AS)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImageOffset {
    /// offset for A axis
    pub a: f64,
    /// offset for B axis
    pub b: f64,
}

impl<W: Write> PartialGerberCode<W> for ImageOffset {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if self.a != 0.0 {
            write!(writer, "A{}", self.a)?;
        }
        if self.b != 0.0 {
            write!(writer, "B{}", self.b)?;
        }
        Ok(())
    }
}

// Axis Select

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
#[strum(serialize_all = "UPPERCASE")]
pub enum AxisSelect {
    AXBY,
    AYBX,
}

impl_partial_gerber_code_via_strum!(AxisSelect);

impl Default for AxisSelect {
    fn default() -> Self {
        AxisSelect::AXBY
    }
}

// Image Polarity

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames, VariantArray)]
pub enum ImagePolarity {
    #[strum(serialize = "POS")]
    Positive,
    #[strum(serialize = "NEG")]
    Negative,
}

impl_partial_gerber_code_via_strum!(ImagePolarity);

impl Default for ImagePolarity {
    fn default() -> Self {
        ImagePolarity::Positive
    }
}

/// Gerber spec 2024.05 8.1.9 "Scale Factor (SF)"
/// By default, A=X, B=Y, but this changes depending on the axis select command (AS)
#[derive(Debug, Clone, PartialEq)]
pub struct ImageName {
    pub name: String,
}

impl<W: Write> PartialGerberCode<W> for ImageName {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        write!(writer, "{}", self.name)?;
        Ok(())
    }
}
