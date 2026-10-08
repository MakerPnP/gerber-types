//! Generic code generation, e.g. implementations of `PartialGerberCode` for
//! bool or Vec<G: GerberCode>.

use std::io::Write;

use crate::errors::GerberResult;
use crate::traits::{GerberCode, PartialGerberCode};
use crate::types::*;
use crate::{CoordinateMode, ZeroOmission};

/// Format a string as printable ASCII with Unicode escapes.
/// Commas are preserved; non-ASCII characters, control characters, `%`, `*`, and
/// backslashes are escaped. Names and macro expressions must not use this helper.
pub(crate) struct EscapedString<'a>(pub(crate) &'a str);

/// Format an attribute field as printable ASCII with Unicode escapes.
/// Uses the same escaping as `EscapedString`, but also escapes commas as
/// `\u002C` because commas separate attribute fields.
pub(crate) struct EscapedField<'a>(pub(crate) &'a str);

impl std::fmt::Display for EscapedString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_escaped(self.0, f, false)
    }
}

impl std::fmt::Display for EscapedField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_escaped(self.0, f, true)
    }
}

fn fmt_escaped(
    text: &str,
    f: &mut std::fmt::Formatter<'_>,
    escape_commas: bool,
) -> std::fmt::Result {
    for ch in text.chars() {
        if !ch.is_ascii()
            || ch.is_ascii_control()
            || matches!(ch, '%' | '*' | '\\')
            || (escape_commas && ch == ',')
        {
            let code = ch as u32;
            if code <= 0xffff {
                write!(f, r"\u{:04X}", code)?;
            } else {
                write!(f, r"\U{:08X}", code)?;
            }
        } else {
            write!(f, "{}", ch)?;
        }
    }
    Ok(())
}

/// Implement `PartialGerberCode` for booleans
impl<W: Write> PartialGerberCode<W> for bool {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if *self {
            write!(writer, "1")?;
        } else {
            write!(writer, "0")?;
        };
        Ok(())
    }
}

/// Implement `GerberCode` for Vectors of types that are `GerberCode`.
impl<W: Write, G: GerberCode<W>> GerberCode<W> for Vec<G> {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        for item in self.iter() {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

/// Implement `PartialGerberCode` for `Option<T: PartialGerberCode>`
impl<T: PartialGerberCode<W>, W: Write> PartialGerberCode<W> for Option<T> {
    fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
        if let Some(ref val) = *self {
            val.serialize_partial(writer)?;
        }
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for Command {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            Command::FunctionCode(ref code) => code.serialize(writer)?,
            Command::ExtendedCode(ref code) => code.serialize(writer)?,
        };
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for FunctionCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            FunctionCode::DCode(ref code) => code.serialize(writer)?,
            FunctionCode::GCode(ref code) => code.serialize(writer)?,
            FunctionCode::MCode(ref code) => code.serialize(writer)?,
        };
        Ok(())
    }
}

impl<W: Write> GerberCode<W> for ExtendedCode {
    fn serialize(&self, writer: &mut W) -> GerberResult<()> {
        match *self {
            ExtendedCode::CoordinateFormat(ref cf) => {
                let zero_omission = match &cf.zero_omission {
                    ZeroOmission::Leading => 'L',
                    ZeroOmission::Trailing => 'T',
                };
                let mode = match &cf.coordinate_mode {
                    CoordinateMode::Absolute => 'A',
                    CoordinateMode::Incremental => 'I',
                };
                writeln!(
                    writer,
                    "%FS{2}{3}X{0}{1}Y{0}{1}*%",
                    cf.integer, cf.decimal, zero_omission, mode
                )?;
            }
            ExtendedCode::Unit(ref unit) => {
                write!(writer, "%MO")?;
                unit.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureDefinition(ref def) => {
                write!(writer, "%ADD")?;
                def.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureMacro(ref am) => {
                write!(writer, "%")?;
                am.serialize_partial(writer)?;
                writeln!(writer, "%")?;
            }
            ExtendedCode::LoadPolarity(ref polarity) => {
                write!(writer, "%LP")?;
                polarity.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadMirroring(ref mirroring) => {
                write!(writer, "%LM")?;
                mirroring.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadRotation(ref rotation) => {
                write!(writer, "%LR")?;
                rotation.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::LoadScaling(ref scaling) => {
                write!(writer, "%LS")?;
                scaling.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::StepAndRepeat(ref sar) => {
                write!(writer, "%SR")?;
                sar.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::FileAttribute(ref attr) => {
                write!(writer, "%TF")?;
                attr.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::DeleteAttribute(ref attr) => {
                write!(writer, "%TD")?;
                attr.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureBlock(ref ab) => {
                write!(writer, "%AB")?;
                ab.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ApertureAttribute(ref aa) => {
                write!(writer, "%TA")?;
                aa.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ObjectAttribute(ref oa) => {
                write!(writer, "%TO")?;
                oa.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::MirrorImage(ref mi) => {
                write!(writer, "%MI")?;
                mi.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::OffsetImage(ref of) => {
                write!(writer, "%OF")?;
                of.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ScaleImage(ref sf) => {
                write!(writer, "%SF")?;
                sf.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::RotateImage(ref ir) => {
                write!(writer, "%IR")?;
                ir.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ImagePolarity(ref ip) => {
                write!(writer, "%IP")?;
                ip.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::AxisSelect(ref r#as) => {
                write!(writer, "%AS")?;
                r#as.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
            ExtendedCode::ImageName(ref r#in) => {
                write!(writer, "%IN")?;
                r#in.serialize_partial(writer)?;
                writeln!(writer, "*%")?;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod unicode_tests {
    use super::*;
    use crate::{ApertureAttribute, FileAttribute, Net, ObjectAttribute};

    #[test]
    fn unicode_and_reserved_characters() {
        assert_eq!(
            EscapedString("ASCII ©中😀\\u0041%*\r\n\t").to_string(),
            r"ASCII \u00A9\u4E2D\U0001F600\u005Cu0041\u0025\u002A\u000D\u000A\u0009"
        );
        assert_eq!(EscapedString(",").to_string(), ",");
        assert_eq!(EscapedField(",").to_string(), r"\u002C");
        assert_eq!(
            EscapedField("\u{ffff}\u{10000}\u{10ffff}").to_string(),
            r"\uFFFF\U00010000\U0010FFFF"
        );
    }

    #[test]
    fn attribute_fields_preserve_separators() {
        let commands = vec![
            Command::ExtendedCode(ExtendedCode::FileAttribute(FileAttribute::UserDefined {
                name: "custom".into(),
                values: vec!["a,b".into(), "©*%".into()],
            })),
            Command::ExtendedCode(ExtendedCode::ApertureAttribute(
                ApertureAttribute::UserDefined {
                    name: "custom".into(),
                    values: vec!["😀".into()],
                },
            )),
            Command::ExtendedCode(ExtendedCode::ObjectAttribute(ObjectAttribute::Net(
                Net::Connected(vec!["a,b".into(), "中".into()]),
            ))),
        ];
        let mut output = Vec::new();
        commands.serialize(&mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(),
            "%TFcustom,a\\u002Cb,\\u00A9\\u002A\\u0025*%\n%TAcustom,\\U0001F600*%\n%TO.N,a\\u002Cb,\\u4E2D*%\n");
    }

    #[test]
    fn comments_preserve_commas_and_escape_delimiters() {
        let mut output = Vec::new();
        crate::CommentContent::String("©, *%\\".into())
            .serialize_partial(&mut output)
            .unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            r"\u00A9, \u002A\u0025\u005C"
        );
        let mut output = Vec::new();
        crate::MacroContent::Comment("😀,*".into())
            .serialize_partial(&mut output)
            .unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), r"0 \U0001F600,\u002A*");
    }

    #[test]
    fn writer_errors_are_propagated() {
        let mut output = std::io::Cursor::new([0u8; 1]);
        let result = crate::CommentContent::String("©".into()).serialize_partial(&mut output);
        assert!(matches!(result, Err(crate::GerberError::IoError(_))));
    }
}
