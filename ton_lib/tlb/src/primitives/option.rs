use crate::errors::TLBResult;
use crate::tlb_type::TLBType;
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;

// Maybe X
impl<T: TLBType> TLBType for Option<T> {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        match parser.read_bit()? {
            false => Ok(None),
            true => Ok(Some(T::read(parser)?)),
        }
    }

    fn write_def(&self, dst: &mut TonCellBuilder) -> TLBResult<()> {
        match self {
            None => {
                dst.write_bit(false)?;
            },
            Some(value) => {
                dst.write_bit(true)?;
                value.write(dst)?;
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::_test_types::TestType1;
    use crate::tlb_type::TLBType;
    use tokio_test::assert_ok;
    use ton_lib_cell::build_parse::builder::TonCellBuilder;
    use ton_lib_cell::build_parse::parser::TonCellParser;

    #[test]
    fn test_option() -> anyhow::Result<()> {
        let obj1 = Some(TestType1 { value: 1 });
        let obj2: Option<TestType1> = None;
        let mut builder = TonCellBuilder::new();
        obj1.write(&mut builder)?;
        obj2.write(&mut builder)?;

        let cell = builder.build()?;
        let mut parser = TonCellParser::new(&cell);
        let parsed_obj1: Option<TestType1> = TLBType::read(&mut parser)?;
        let parsed_obj2: Option<TestType1> = TLBType::read(&mut parser)?;
        assert_eq!(obj1, parsed_obj1);
        assert_eq!(None, parsed_obj2);

        // check layout
        let mut parser = TonCellParser::new(&cell);
        assert!(parser.read_bit()?); // Some
        assert_ok!(parser.read_bits(32, &mut [0; 32])); // skipping
        assert!(!parser.read_bit()?); // None
        Ok(())
    }
}
