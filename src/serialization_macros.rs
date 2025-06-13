macro_rules! impl_partial_gerber_code_via_strum {
    ($name:ident) => {
        impl<W: Write> PartialGerberCode<W> for $name {
            fn serialize_partial(&self, writer: &mut W) -> GerberResult<()> {
                let value: &'static str = self.into();
                write!(writer, "{}", value)?;
                Ok(())
            }
        }
    };
}
