//! Types for Gerber code generation related to coordinates.

use std::convert::TryFrom;
use std::convert::{From, Into};
use std::io::Write;
use std::num::FpCategory;

use num_rational::Ratio;

use crate::errors::{GerberError, GerberResult};
use crate::traits::PartialGerberCode;

// Helper macros

/// Automatically implement `PartialGerberCode` trait for struct types
/// that are based on `x` and `y` attributes.
macro_rules! impl_xy_partial_gerbercode {
    ($class:ty, $x:expr, $y: expr) => {
        impl<W: Write> PartialGerberCode<W> for $class {
            fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
                if let Some(x) = self.x {
                    write!(writer, "{}{}", $x, x.gerber(&self.format)?)?;
                }
                if let Some(y) = self.y {
                    write!(writer, "{}{}", $y, y.gerber(&self.format)?)?;
                }
                Ok(())
            }
        }
    };
}

// Types

/// The coordinate format specifies the number of integer and decimal places in
/// a coordinate number. For example, the `24` format specifies 2 integer and 4
/// decimal places. The number of decimal places must be 4, 5 or 6. The number
/// of integer places must be not more than 6. Thus the longest representable
/// coordinate number is `nnnnnn.nnnnnn`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CoordinateFormat {
    pub integer: u8,
    pub decimal: u8,
}

impl CoordinateFormat {
    pub fn new(integer: u8, decimal: u8) -> Self {
        CoordinateFormat { integer, decimal }
    }
}

/// Coordinate numbers are integers conforming to the rules set by the FS
/// command.
///
/// Coordinate numbers are integers. Explicit decimal points are not allowed.
///
/// A coordinate number must have at least one character. Zero therefore must
/// be encoded as `0`.
///
/// The value is stored as a 64 bit integer with 6 decimal places.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CoordinateNumber {
    nano: i64,
}

impl CoordinateNumber {
    pub fn new(nano: i64) -> Self {
        CoordinateNumber { nano }
    }
}

const DECIMAL_PLACES_CHARS: u8 = 6;
const DECIMAL_PLACES_FACTOR: i64 = 1_000_000;

impl TryFrom<f64> for CoordinateNumber {
    type Error = GerberError;
    fn try_from(val: f64) -> Result<Self, Self::Error> {
        match val.classify() {
            FpCategory::Nan => Err(GerberError::ConversionError("Value is NaN".into())),
            FpCategory::Infinite => Err(GerberError::ConversionError("Value is infinite".into())),
            FpCategory::Zero | FpCategory::Subnormal => Ok(CoordinateNumber { nano: 0 }),
            FpCategory::Normal => {
                let multiplied = val * DECIMAL_PLACES_FACTOR as f64;
                if (multiplied > i64::MAX as f64) || (multiplied < i64::MIN as f64) {
                    Err(GerberError::ConversionError(
                        "Value is out of bounds".into(),
                    ))
                } else {
                    Ok(CoordinateNumber {
                        nano: multiplied as i64,
                    })
                }
            }
        }
    }
}

impl From<CoordinateNumber> for f64 {
    fn from(value: CoordinateNumber) -> Self {
        (value.nano as f64) / DECIMAL_PLACES_FACTOR as f64
    }
}

macro_rules! impl_from_integer {
    ($class:ty) => {
        impl From<$class> for CoordinateNumber {
            fn from(val: $class) -> Self {
                CoordinateNumber {
                    nano: val as i64 * DECIMAL_PLACES_FACTOR,
                }
            }
        }
    };
}

// These are the types we can safely multiply with DECIMAL_PLACES_FACTOR
// without the risk of an overflow.
impl_from_integer!(i8);
impl_from_integer!(i16);
impl_from_integer!(i32);
impl_from_integer!(u8);
impl_from_integer!(u16);

// Define a new trait for types that can be converted into an Option<CoordinateNumber>
pub trait IntoOptionalCoordinate {
    fn into_optional_coordinate(self) -> Option<CoordinateNumber>;
}

// Implement for CoordinateNumber
impl IntoOptionalCoordinate for CoordinateNumber {
    fn into_optional_coordinate(self) -> Option<CoordinateNumber> {
        Some(self)
    }
}

impl IntoOptionalCoordinate for Option<CoordinateNumber> {
    fn into_optional_coordinate(self) -> Option<CoordinateNumber> {
        self
    }
}

macro_rules! impl_from_optional_integer {
    ($class:ty) => {
        // Implement for integer types
        impl IntoOptionalCoordinate for $class {
            fn into_optional_coordinate(self) -> Option<CoordinateNumber> {
                Some(CoordinateNumber::from(self))
            }
        }
    };
}

impl_from_optional_integer!(i8);
impl_from_optional_integer!(i16);
impl_from_optional_integer!(i32);
impl_from_optional_integer!(u8);
impl_from_optional_integer!(u16);

// Add implementations for Option<integer> types
macro_rules! impl_from_option_integer {
    ($class:ty) => {
        impl IntoOptionalCoordinate for Option<$class> {
            fn into_optional_coordinate(self) -> Option<CoordinateNumber> {
                self.map(CoordinateNumber::from)
            }
        }
    };
}

impl_from_option_integer!(i8);
impl_from_option_integer!(i16);
impl_from_option_integer!(i32);
impl_from_option_integer!(u8);
impl_from_option_integer!(u16);

impl CoordinateNumber {
    pub fn gerber(&self, format: &CoordinateFormat) -> Result<String, GerberError> {
        self.validate(format)?;

        let divisor: i64 = 10_i64.pow((DECIMAL_PLACES_CHARS - format.decimal) as u32);
        let number: i64 = Ratio::new(self.nano, divisor).round().to_integer();
        Ok(number.to_string())
    }

    pub fn validate(&self, format: &CoordinateFormat) -> Result<(), GerberError> {
        if format.decimal > DECIMAL_PLACES_CHARS {
            return Err(GerberError::CoordinateFormatError(
                "Invalid precision: Too high!".into(),
            ));
        }
        if self.nano.abs() >= 10_i64.pow((format.integer + DECIMAL_PLACES_CHARS) as u32) {
            return Err(GerberError::CoordinateFormatError(
                "Number is too large for the chosen format!".into(),
            ));
        }
        Ok(())
    }
}

/// Coordinates are part of an operation.
///
/// Coordinates are modal. If an X is omitted, the X coordinate of the
/// current point is used. Similar for Y.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coordinates {
    pub x: Option<CoordinateNumber>,
    pub y: Option<CoordinateNumber>,
    pub format: CoordinateFormat,
}

impl Coordinates {
    pub fn new<T, U>(x: T, y: U, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
        U: IntoOptionalCoordinate,
    {
        Coordinates {
            x: x.into_optional_coordinate(),
            y: y.into_optional_coordinate(),
            format,
        }
    }

    pub fn at_x<T>(x: T, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
    {
        Coordinates {
            x: x.into_optional_coordinate(),
            y: None,
            format,
        }
    }

    pub fn at_y<T>(y: T, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
    {
        Coordinates {
            x: None,
            y: y.into_optional_coordinate(),
            format,
        }
    }
}

impl_xy_partial_gerbercode!(Coordinates, "X", "Y");

/// Coordinate offsets can be used for interpolate operations in circular
/// interpolation mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateOffset {
    pub x: Option<CoordinateNumber>,
    pub y: Option<CoordinateNumber>,
    pub format: CoordinateFormat,
}

impl CoordinateOffset {
    pub fn new<T, U>(x: T, y: U, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
        U: IntoOptionalCoordinate,
    {
        CoordinateOffset {
            x: x.into_optional_coordinate(),
            y: y.into_optional_coordinate(),
            format,
        }
    }

    pub fn at_x<T>(x: T, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
    {
        CoordinateOffset {
            x: x.into_optional_coordinate(),
            y: None,
            format,
        }
    }

    pub fn at_y<T>(y: T, format: CoordinateFormat) -> Self
    where
        T: IntoOptionalCoordinate,
    {
        CoordinateOffset {
            x: None,
            y: y.into_optional_coordinate(),
            format,
        }
    }
}

impl_xy_partial_gerbercode!(CoordinateOffset, "I", "J");

#[cfg(test)]
mod test {
    use super::*;

    use std::f64;
    use std::io::BufWriter;

    use std::convert::TryFrom;

    use crate::traits::PartialGerberCode;

    #[test]
    /// Test integer to coordinate number conversion
    fn test_from_i8() {
        let a = CoordinateNumber { nano: 13000000 };
        let b = CoordinateNumber::from(13i8);
        assert_eq!(a, b);

        let c = CoordinateNumber { nano: -99000000 };
        let d = CoordinateNumber::from(-99i8);
        assert_eq!(c, d);
    }

    #[test]
    /// Test integer to coordinate number conversion
    fn test_from_i32() {
        let a = CoordinateNumber { nano: 13000000 };
        let b = CoordinateNumber::from(13);
        assert_eq!(a, b);

        let c = CoordinateNumber { nano: -998000000 };
        let d = CoordinateNumber::from(-998);
        assert_eq!(c, d);
    }

    #[test]
    /// Test float to coordinate number conversion
    fn test_try_from_f64_success() {
        let a = CoordinateNumber { nano: 1375000i64 };
        let b = CoordinateNumber::try_from(1.375f64).unwrap();
        assert_eq!(a, b);

        let c = CoordinateNumber {
            nano: 123456888888i64,
        };
        let d = CoordinateNumber::try_from(123456.888888f64).unwrap();
        assert_eq!(c, d);

        let e = CoordinateNumber { nano: 0i64 };
        let f = CoordinateNumber::try_from(0f64).unwrap();
        assert_eq!(e, f);

        let g = CoordinateNumber { nano: -12345678 };
        let h = CoordinateNumber::try_from(-12.345678).unwrap();
        assert_eq!(g, h);
    }

    #[test]
    /// Test failing float to coordinate number conversion
    fn test_try_from_f64_fail() {
        let cn1 = CoordinateNumber::try_from(f64::NAN);
        assert!(cn1.is_err());

        let cn2 = CoordinateNumber::try_from(f64::INFINITY);
        assert!(cn2.is_err());

        let cn3 = CoordinateNumber::try_from(f64::MAX - 1.0);
        assert!(cn3.is_err());

        let cn4 = CoordinateNumber::try_from(f64::MIN + 1.0);
        assert!(cn4.is_err());
    }

    #[test]
    /// Test coordinate number to float conversion
    fn test_into_f64() {
        let a: f64 = CoordinateNumber { nano: 1375000i64 }.into();
        let b = 1.375f64;
        assert_eq!(a, b);

        let c: f64 = CoordinateNumber {
            nano: 123456888888i64,
        }
        .into();
        let d = 123456.888888f64;
        assert_eq!(c, d);

        let e: f64 = CoordinateNumber { nano: 0i64 }.into();
        let f = 0f64;
        assert_eq!(e, f);
    }

    #[test]
    /// Test the coordinate number constructor creates correct
    /// coordinate numbers.
    fn test_coordinate_number_new() {
        let nano = 5;
        let cn1 = CoordinateNumber::new(nano);
        assert_eq!(cn1.nano, nano);
    }

    #[test]
    /// Test coordinate number to string conversion when it's 0
    fn test_formatted_zero() {
        let cf1 = CoordinateFormat::new(6, 6);
        let cf2 = CoordinateFormat::new(2, 4);

        let a = CoordinateNumber { nano: 0 }.gerber(&cf1).unwrap();
        let b = CoordinateNumber { nano: 0 }.gerber(&cf2).unwrap();
        assert_eq!(a, "0".to_string());
        assert_eq!(b, "0".to_string());
    }

    #[test]
    /// Test coordinate number to string conversion when the decimal part is 0
    fn test_formatted_decimal_zero() {
        let cf1 = CoordinateFormat::new(6, 6);
        let cf2 = CoordinateFormat::new(2, 4);

        let a = CoordinateNumber { nano: 10000000 }.gerber(&cf1).unwrap();
        let b = CoordinateNumber { nano: 20000000 }.gerber(&cf2).unwrap();
        assert_eq!(a, "10000000".to_string());
        assert_eq!(b, "200000".to_string());
    }

    #[test]
    /// Test coordinate number to string conversion
    fn test_formatted_65() {
        let cf = CoordinateFormat::new(6, 5);
        let d = CoordinateNumber { nano: 123456789012 }.gerber(&cf).unwrap();
        assert_eq!(d, "12345678901".to_string());
    }

    #[test]
    /// Test coordinate number to string conversion
    fn test_formatted_54() {
        let cf = CoordinateFormat::new(5, 4);
        let d = CoordinateNumber { nano: 12345678901 }.gerber(&cf).unwrap();
        assert_eq!(d, "123456789".to_string());
    }

    #[test]
    /// Test coordinate number to string conversion failure
    fn test_formatted_number_too_large() {
        let cf = CoordinateFormat::new(4, 5);
        let d = CoordinateNumber { nano: 12345000000 }.gerber(&cf);
        assert!(d.is_err());
    }

    #[test]
    /// Test coordinate number to string conversion failure
    fn test_formatted_negative_number_too_large() {
        let cf = CoordinateFormat::new(4, 5);
        let d = CoordinateNumber { nano: -12345000000 }.gerber(&cf);
        assert!(d.is_err());
    }

    #[test]
    /// Test coordinate number to string conversion (rounding of decimal part)
    fn test_formatted_44_rounding() {
        let cf = CoordinateFormat::new(4, 4);
        let d = CoordinateNumber { nano: 1234432199 }.gerber(&cf).unwrap();
        assert_eq!(d, "12344322".to_string());
    }

    #[test]
    /// Test negative coordinate number to string conversion
    fn test_formatted_negative_rounding() {
        let cf = CoordinateFormat::new(6, 4);
        let d = CoordinateNumber {
            nano: -123456789099,
        }
        .gerber(&cf)
        .unwrap();
        assert_eq!(d, "-1234567891".to_string());
    }

    #[test]
    fn test_coordinates_into() {
        let cf = CoordinateFormat::new(2, 4);
        let c1 = Coordinates::new(CoordinateNumber::from(1), CoordinateNumber::from(2), cf);
        let c2 = Coordinates::new(1, 2, cf);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinates_into_option_some() {
        let cf = CoordinateFormat::new(2, 4);
        let c1 = Coordinates::new(CoordinateNumber::from(1), CoordinateNumber::from(2), cf);
        let c2 = Coordinates::new(Some(1), Some(2), cf);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinates_into_option_none() {
        let cf = CoordinateFormat::new(2, 4);
        let c1 = Coordinates {
            x: None,
            y: None,
            format: cf,
        };
        let c2 = Coordinates::new::<Option<i8>, Option<i8>>(None, None, cf);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinates_into_option_partial() {
        let cf = CoordinateFormat::new(2, 4);
        let c1 = Coordinates {
            x: Some(CoordinateNumber::from(1_i32)),
            y: None,
            format: cf,
        };
        let c2 = Coordinates::new::<Option<i32>, Option<i32>>(Some(1_i32), None, cf);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinates_into_mixed() {
        let cf = CoordinateFormat::new(2, 4);
        let c1 = Coordinates::new(CoordinateNumber::from(1), 2, cf);
        let c2 = Coordinates::new(1, 2, cf);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinates() {
        macro_rules! assert_coords {
            ($coords:expr, $result:expr) => {{
                assert_partial_code!($coords, $result);
            }};
        }
        let cf44 = CoordinateFormat::new(4, 4);
        let cf46 = CoordinateFormat::new(4, 6);
        assert_coords!(Coordinates::new(10, 20, cf44), "X100000Y200000");
        assert_coords!(
            Coordinates {
                x: None,
                y: None,
                format: cf44
            },
            ""
        ); // TODO should we catch this?
        assert_coords!(Coordinates::at_x(10, cf44), "X100000");
        assert_coords!(Coordinates::at_x(Some(10), cf44), "X100000");
        assert_coords!(Coordinates::at_y(20, cf46), "Y20000000");
        assert_coords!(Coordinates::at_y(Some(20), cf46), "Y20000000");
        assert_coords!(Coordinates::new(0, -400, cf44), "X0Y-4000000");
        assert_coords!(Coordinates::new(Some(0), Some(-400), cf44), "X0Y-4000000");
    }

    #[test]
    fn test_offset() {
        macro_rules! assert_coords {
            ($coords:expr, $result:expr) => {{
                assert_partial_code!($coords, $result);
            }};
        }
        let cf44 = CoordinateFormat::new(4, 4);
        let cf55 = CoordinateFormat::new(5, 5);
        let cf66 = CoordinateFormat::new(6, 6);
        assert_coords!(CoordinateOffset::new(10, 20, cf44), "I100000J200000");
        assert_coords!(
            CoordinateOffset {
                x: None,
                y: None,
                format: cf44
            },
            ""
        ); // TODO should we catch this?
        assert_coords!(CoordinateOffset::at_x(Some(10), cf66), "I10000000");
        assert_coords!(CoordinateOffset::at_x(10, cf66), "I10000000");
        assert_coords!(CoordinateOffset::at_y(Some(20), cf55), "J2000000");
        assert_coords!(CoordinateOffset::at_y(20, cf55), "J2000000");
        assert_coords!(CoordinateOffset::new(0, -400, cf44), "I0J-4000000");
        assert_coords!(
            CoordinateOffset::new(Some(0), Some(-400), cf44),
            "I0J-4000000"
        );
    }

    #[test]
    fn test_validate_too_large_for_format() {
        // %FSLAX23Y23*%
        let cf23 = CoordinateFormat::new(2, 3);
        let number = CoordinateNumber::try_from(100.001).unwrap();

        println!("{:?}", number);
        // 100_001000 = 100.001 = 2,3
        // '100' is too large for 2 digits before the decimal point
        assert!(number.nano == 100_001000);

        let result = number.validate(&cf23);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(GerberError::CoordinateFormatError(str)) if str.eq("Number is too large for the chosen format!")
        ));
    }
}
