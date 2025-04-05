use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::num::traits::TonCellNum;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::errors::TonLibError;
use bitstream_io::{BigEndian, BitRead, BitReader};
use num_traits::Zero;
use std::io::{Cursor, SeekFrom};

pub struct CellParser<'a> {
    pub cell: &'a TonCell,
    pub next_ref_pos: usize,
    data_reader: BitReader<Cursor<&'a [u8]>, BigEndian>,
}

impl<'a> CellParser<'a> {
    pub fn new(cell: &'a TonCell) -> Self {
        let cursor = Cursor::new(cell.data.as_slice());
        let data_reader = BitReader::endian(cursor, BigEndian);

        Self {
            cell,
            data_reader,
            next_ref_pos: 0,
        }
    }

    pub fn lookup_bits(&mut self, bits_len: u8) -> Result<u128, TonLibError> {
        let value = self.read_num(bits_len as u32)?;
        self.seek_bits(-(bits_len as i32))?;
        Ok(value)
    }

    pub fn read_bit(&mut self) -> Result<bool, TonLibError> {
        self.ensure_enough_bits(1)?;
        Ok(self.data_reader.read_bit()?)
    }

    pub fn read_bits(&mut self, bits_len: u32) -> Result<Vec<u8>, TonLibError> {
        self.ensure_enough_bits(bits_len)?;
        let mut dst = vec![0; (bits_len as usize).div_ceil(8)];
        let full_bytes = bits_len as usize / 8;
        let remaining_bits = bits_len % 8;

        self.data_reader.read_bytes(&mut dst[..full_bytes])?;

        if remaining_bits != 0 {
            let last_byte = self.data_reader.read::<u8>(remaining_bits)?;
            dst[full_bytes] = last_byte << (8 - remaining_bits);
        }
        Ok(dst)
    }

    pub fn read_num<N: TonCellNum>(&mut self, bits_len: u32) -> Result<N, TonLibError> {
        if bits_len == 0 {
            return Ok(N::from_primitive(N::Primitive::zero()));
        }
        self.ensure_enough_bits(bits_len)?;
        if N::IS_PRIMITIVE {
            let primitive = self.data_reader.read::<N::Primitive>(bits_len)?;
            return Ok(N::from_primitive(primitive));
        }
        let bytes = self.read_bits(bits_len)?;
        let res = N::from_bytes(&bytes);
        if bits_len % 8 != 0 {
            return Ok(res.shr(8 - bits_len % 8));
        }
        Ok(res)
    }

    pub fn read_cell(&mut self) -> Result<TonCell, TonLibError> {
        let bits_left = self.data_bits_left()?;
        let data = self.read_bits(bits_left)?;

        let mut builder = CellBuilder::new();
        builder.write_bits(data, bits_left)?;

        while let Ok(cell_ref) = self.read_next_ref() {
            builder.write_ref(cell_ref.clone())?;
        }
        builder.build()
    }

    pub fn read_next_ref(&mut self) -> Result<&TonCellRef, TonLibError> {
        if self.next_ref_pos == self.cell.refs.len() {
            return Err(TonLibError::ParserRefsUnderflow { req: self.next_ref_pos });
        }
        let cell_ref = &self.cell.refs[self.next_ref_pos];
        self.next_ref_pos += 1;
        Ok(cell_ref)
    }

    pub fn data_bits_left(&mut self) -> Result<u32, TonLibError> {
        let reader_pos = self.data_reader.position_in_bits()? as u32;
        Ok(self.cell.data_bits_len as u32 - reader_pos)
    }

    pub fn seek_bits(&mut self, offset: i32) -> Result<(), TonLibError> {
        let new_pos = self.data_reader.position_in_bits()? as i32 + offset;
        let data_bits_len = self.cell.data_bits_len as i32;
        if new_pos < 0 || new_pos > (data_bits_len - 1) {
            return Err(TonLibError::ParserBadPosition {
                new_pos,
                bits_len: data_bits_len as u32,
            });
        }
        self.data_reader.seek_bits(SeekFrom::Current(offset as i64))?;
        Ok(())
    }

    pub fn ensure_empty(&mut self) -> Result<(), TonLibError> {
        let bits_left = self.data_bits_left()?;
        if bits_left == 0 {
            return Ok(());
        }

        Err(TonLibError::ParserCellNotEmpty { bits_left })
    }

    // returns remaining bits
    fn ensure_enough_bits(&mut self, bit_len: u32) -> Result<u32, TonLibError> {
        let bits_left = self.data_bits_left()?;

        if bit_len <= bits_left {
            return Ok(bits_left);
        }
        Err(TonLibError::ParserDataUnderflow {
            req: bit_len,
            left: bits_left,
        })
    }
}

#[cfg(feature = "fastnum")]
#[cfg(feature = "num-bigint")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::build_parse::builder::CellBuilder;
    use num_bigint::{BigInt, BigUint};
    use tokio_test::{assert_err, assert_ok};

    fn make_test_cell(data: &[u8], bits_len: u32) -> anyhow::Result<TonCell> {
        let mut builder = CellBuilder::new();
        builder.write_bits(data, bits_len)?;
        Ok(builder.build()?)
    }

    #[test]
    fn test_parser_seek_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101001, 0b01010100], 10)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_ok!(parser.seek_bits(3));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 3);
        assert_ok!(parser.seek_bits(-2));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
        assert_ok!(parser.seek_bits(0));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
        assert_ok!(parser.seek_bits(-1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
        assert_err!(parser.seek_bits(-1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
        assert_ok!(parser.seek_bits(cell_slice.data_bits_len as i32 - 1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, cell_slice.data_bits_len - 1);
        assert_err!(parser.seek_bits(1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, cell_slice.data_bits_len - 1);

        assert_err!(parser.seek_bits(20));
        Ok(())
    }

    #[test]
    fn test_parser_lookup_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.lookup_bits(3)?, 0b101);
        assert_eq!(parser.data_reader.position_in_bits()?, 0);
        assert!(assert_ok!(parser.read_bit()));
        assert_eq!(parser.data_reader.position_in_bits()?, 1);
        assert_eq!(parser.lookup_bits(3)?, 0b010);
        assert_eq!(parser.data_reader.position_in_bits()?, 1);
        Ok(())
    }

    #[test]
    fn test_parser_read_bit() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        for i in 0..8 {
            assert_eq!(assert_ok!(parser.read_bit()), i % 2 == 0);
        }
        for i in 0..8 {
            assert_eq!(assert_ok!(parser.read_bit()), i % 2 != 0);
        }
        Ok(())
    }

    #[test]
    fn test_parser_ensure_enough_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 10)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.data_reader.position_in_bits()?, 0);
        assert_ok!(parser.ensure_enough_bits(0));
        assert_ok!(parser.ensure_enough_bits(1));
        assert_ok!(parser.ensure_enough_bits(6));
        assert_ok!(parser.ensure_enough_bits(10));
        assert_err!(parser.ensure_enough_bits(11));
        Ok(())
    }

    #[test]
    fn test_parser_read_ref() -> anyhow::Result<()> {
        let mut ref_builder = CellBuilder::new();
        ref_builder.write_num(&0b11110000, 8)?;
        let cell_ref = ref_builder.build()?.into_ref();

        let mut cell_builder = CellBuilder::new();
        cell_builder.write_ref(cell_ref.clone())?;
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?.into_ref();

        let mut parser = CellParser::new(cell.as_ref());
        assert_eq!(parser.read_next_ref()?.data, cell_ref.data);
        assert_eq!(parser.read_next_ref()?.data, cell_ref.data);
        assert!(parser.read_next_ref().is_err());
        Ok(())
    }

    #[test]
    fn test_parser_read_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        let dst = parser.read_bits(3)?;
        assert_eq!(dst, [0b10100000]);
        let dst = parser.read_bits(6)?;
        assert_eq!(dst, [0b01010000]);
        Ok(())
    }

    #[test]
    fn test_parser_read_num() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<u8>(3)?, 0b101);
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<u32>(3)?, 0b010);
        assert_eq!(parser.data_reader.position_in_bits()?, 6);
        assert_eq!(parser.read_num::<u64>(3)?, 0b100);
        assert_eq!(parser.data_reader.position_in_bits()?, 9);
        Ok(())
    }

    #[test]
    fn test_parser_read_num_unaligned() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b0001_0001, 0b0000_0000, 0b1010_0000], 19)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<u8>(4)?, 1);
        assert_eq!(parser.data_reader.position_in_bits()?, 4);
        assert_eq!(parser.read_num::<u16>(5)?, 2);
        assert_eq!(parser.data_reader.position_in_bits()?, 9);
        assert_eq!(parser.read_num::<u32>(10)?, 5);
        assert_eq!(parser.data_reader.position_in_bits()?, 19);
        Ok(())
    }

    #[test]
    fn test_parser_read_cell() -> anyhow::Result<()> {
        let data = [0b0001_0001, 0b0000_0000, 0b1010_0000];
        let cell = make_test_cell(&data, 19)?;
        assert_eq!(CellParser::new(&cell).read_cell()?, cell);
        Ok(())
    }

    #[test]
    fn test_parser_read_bigint() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b111_01010, 0b01101011, 0b10000000, 0b00000001], 32)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<BigInt>(3)?, (-1).into());
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<BigInt>(5)?, 10.into()); // finish with first byte
        assert_eq!(parser.data_reader.position_in_bits()?, 8);
        parser.read_bit()?; // skip 1 bit
        assert_eq!(parser.read_num::<BigInt>(7)?, (-21).into()); // finish with second byte
        assert_eq!(parser.data_reader.position_in_bits()?, 16);
        assert_eq!(parser.read_num::<BigInt>(16)?, (-32767).into());
        Ok(())
    }

    #[test]
    fn test_parser_read_bigint_unaligned() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b00011110, 0b11111111], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        parser.seek_bits(3)?;
        assert_eq!(parser.read_num::<BigInt>(9)?, (-17).into());
        Ok(())
    }

    #[test]
    fn test_parser_read_biguint() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101, 0b11111111, 0b11111111], 32)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<BigUint>(3)?, 5u32.into());
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<BigUint>(5)?, 10u32.into()); // finish with first byte
        assert_eq!(parser.data_reader.position_in_bits()?, 8);
        parser.read_bit()?; // skip 1 bit
        assert_eq!(parser.read_num::<BigUint>(7)?, 85u32.into()); // finish with second byte
        assert_eq!(parser.data_reader.position_in_bits()?, 16);
        assert_eq!(parser.read_num::<BigUint>(16)?, 65535u32.into());
        Ok(())
    }
}
