use crate::cell::cell_owned::CellOwned;
use crate::cell::ton_cell::{TonCellRef, TonCell};
use crate::errors::{TonCellError, TonCellResult};
use crate::number::TonNumber;
use bitstream_io::{BigEndian, BitRead, BitReader};
use std::io::{Cursor, SeekFrom};

pub struct TonCellParser<'a> {
    pub cell: &'a dyn TonCell,
    pub next_ref_pos: usize,
    data_reader: BitReader<Cursor<&'a [u8]>, BigEndian>,
}

impl<'a> TonCellParser<'a> {
    pub fn new(cell: &'a dyn TonCell) -> Self {
        let cursor = Cursor::new(cell.get_data());
        let data_reader = BitReader::endian(cursor, BigEndian);

        Self {
            cell,
            data_reader,
            next_ref_pos: 0,
        }
    }

    pub fn lookup_bits(&mut self, bits_len: u8) -> TonCellResult<u128> {
        let value = self.read_num(bits_len as u32)?;
        self.seek_bits(-(bits_len as i32))?;
        Ok(value)
    }

    pub fn read_bit(&mut self) -> TonCellResult<bool> {
        self.ensure_enough_bits(1)?;
        Ok(self.data_reader.read_bit()?)
    }

    pub fn read_bits(&mut self, bits_len: u32, dst: &mut [u8]) -> TonCellResult<()> {
        if dst.len() * 8 < bits_len as usize {
            return Err(TonCellError::ParserSmallContainer {
                requested: bits_len,
                available: dst.len() as u32,
            });
        }
        self.ensure_enough_bits(bits_len)?;
        let full_bytes = bits_len as usize / 8;
        let remaining_bits = bits_len % 8;

        self.data_reader.read_bytes(&mut dst[..full_bytes])?;

        if remaining_bits != 0 {
            let last_byte = self.data_reader.read::<u8>(remaining_bits)?;
            dst[full_bytes] = last_byte << (8 - remaining_bits);
        }
        Ok(())
    }

    pub fn read_byte(&mut self) -> TonCellResult<u8> {
        self.ensure_enough_bits(8)?;
        Ok(self.data_reader.read::<u8>(8)?)
    }

    pub fn read_bytes(&mut self, dst: &mut [u8]) -> TonCellResult<()> {
        self.read_bits((dst.len() * 8) as u32, dst)?;
        Ok(())
    }

    pub fn read_num<N: TonNumber>(&mut self, bit_len: u32) -> TonCellResult<N> {
        self.ensure_enough_bits(bit_len)?;
        Ok(self.data_reader.read::<N>(bit_len)?)
    }

    pub fn read_next_ref(&mut self) -> TonCellResult<&TonCellRef> {
        let refs = self.cell.get_refs();
        if self.next_ref_pos == refs.len() {
            return Err(TonCellError::ParserRefsUnderflow {
                requested: self.next_ref_pos,
            });
        }
        let cell_ref = &refs[self.next_ref_pos];
        self.next_ref_pos += 1;
        Ok(cell_ref)
    }

    pub fn read_rest_data(&mut self) -> TonCellResult<(Vec<u8>, u32)> {
        let bits_left = self.ensure_enough_bits(0)?;

        let mut data = vec![0u8; (bits_left as usize + 7) / 8];
        self.read_bits(bits_left, &mut data)?;
        Ok((data, bits_left))
    }

    pub fn read_rest_cell(&mut self) -> TonCellResult<CellOwned> { todo!() }

    pub fn ensure_empty(&mut self) -> TonCellResult<()> {
        let bits_left = self.data_bits_left()?;
        if bits_left == 0 {
            return Ok(());
        }

        Err(TonCellError::ParserCellNotEmpty { bits_left })
    }

    pub fn data_bits_left(&mut self) -> TonCellResult<u32> {
        let reader_pos = self.data_reader.position_in_bits()? as u32;
        Ok(self.cell.get_data_bits_len() as u32 - reader_pos)
    }

    // returns remaining bits
    fn ensure_enough_bits(&mut self, bit_len: u32) -> TonCellResult<u32> {
        let bits_left = self.data_bits_left()?;

        if bit_len <= bits_left {
            return Ok(bits_left);
        }
        Err(TonCellError::ParserDataUnderflow {
            requested: bit_len,
            left: bits_left,
        })
    }

    fn seek_bits(&mut self, offset: i32) -> TonCellResult<()> {
        let new_pos = self.data_reader.position_in_bits()? as i32 + offset;
        let data_bits_len = self.cell.get_data_bits_len() as i32;
        if new_pos < 0 || new_pos > (data_bits_len - 1) {
            return Err(TonCellError::ParserBadPosition {
                new_position: new_pos,
                data_bits_len: data_bits_len as u32,
            });
        }
        self.data_reader.seek_bits(SeekFrom::Current(offset as i64))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::cell::cell_slice::CellSlice;
    use crate::cell::meta::cell_meta::CellMeta;
    use tokio_test::{assert_err, assert_ok};
    use crate::cell::ton_cell::TonCellRefsStore;

    #[test]
    fn test_parser_seek_bits() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101001, 0b01010100],
            data_bits_len: 10,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
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
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
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
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
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
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 10,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
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
        let cell_ref = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b11110000],
            data_bits_len: 0,
            refs: TonCellRefsStore::new(),
        }.into_ref();
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![],
            data_bits_len: 0,
            refs: TonCellRefsStore::from(vec![cell_ref.clone(), cell_ref.clone()]),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        assert_eq!(parser.read_next_ref()?.get_data(), cell_ref.get_data());
        assert_eq!(parser.read_next_ref()?.get_data(), cell_ref.get_data());
        assert!(parser.read_next_ref().is_err());
        Ok(())
    }

    #[test]
    fn test_parser_read_bits() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        let mut dst = [0u8; 2];
        parser.read_bits(3, &mut dst)?;
        assert_eq!(dst, [0b10100000, 0]);
        parser.read_bits(6, &mut dst)?;
        assert_eq!(dst, [0b01010000, 0]);
        Ok(())
    }

    #[test]
    fn test_parser_read_byte() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        assert_eq!(parser.read_byte()?, 0b10101010);
        assert_eq!(parser.data_reader.position_in_bits()?, 8);
        assert_eq!(parser.read_byte()?, 0b01010101);
        assert_eq!(parser.data_reader.position_in_bits()?, 16);
        Ok(())
    }

    #[test]
    fn test_parser_read_bytes() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        let mut dst = [0u8; 2];
        parser.read_bytes(&mut dst)?;
        assert_eq!(dst, [0b10101010, 0b01010101]);
        Ok(())
    }

    #[test]
    fn test_parser_read_num() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
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
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b0001_0001, 0b0000_0000, 0b1010_0000],
            data_bits_len: 19,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<u8>(4)?, 1);
        assert_eq!(parser.data_reader.position_in_bits()?, 4);
        assert_eq!(parser.read_num::<u16>(5)?, 2);
        assert_eq!(parser.data_reader.position_in_bits()?, 9);
        assert_eq!(parser.read_num::<u32>(10)?, 5);
        assert_eq!(parser.data_reader.position_in_bits()?, 19);
        Ok(())
    }

    #[test]
    fn test_parser_read_rest() -> anyhow::Result<()> {
        let cell_slice = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b10101010, 0b01010101],
            data_bits_len: 16,
            refs: TonCellRefsStore::new(),
        };
        let mut parser = TonCellParser::new(&cell_slice);
        assert!(parser.read_bit()?);
        assert_eq!(parser.read_rest_data()?, (vec![0b01010100, 0b10101010], 15));
        Ok(())
    }
}
