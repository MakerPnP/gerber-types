//! Extended code types.

use std::io::Write;

use crate::errors::GerberResult;
use crate::traits::PartialGerberCode;

// Unit

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Inches,
    Millimeters,
}

impl<W: Write> PartialGerberCode<W> for Unit {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Unit::Millimeters => write!(writer, "MM")?,
            Unit::Inches => write!(writer, "IN")?,
        };
        Ok(())
    }
}

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
    Macro(String, Option<String>),
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
                    write!(writer, ",{}", args)?;
                }
            },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Clear,
    Dark,
}

impl<W: Write> PartialGerberCode<W> for Polarity {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Polarity::Clear => write!(writer, "C")?,
            Polarity::Dark => write!(writer, "D")?,
        };
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
}
