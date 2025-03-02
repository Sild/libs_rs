use crate::cell::cell_owned::{CellOwned};
use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::cell_type::CellType;
use crate::errors::{TonCellError, TonCellResult};
use crate::number::TonNumber;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use crate::cell::ton_cell::{TonCellRef, TonCellRefsStore};

pub struct TonCellBuilder {
    cell_type: CellType,
    data_writer: BitWriter<Vec<u8>, BigEndian>,
    data_bits_len: usize,
    refs: TonCellRefsStore,
}

impl Default for TonCellBuilder {
    fn default() -> Self { Self::new() }
}

impl TonCellBuilder {
    pub fn new() -> Self { Self::new_with_type(CellType::Ordinary) }

    pub fn new_with_type(cell_type: CellType) -> Self {
        let buffer = vec![];
        let bit_writer = BitWriter::endian(buffer, BigEndian);
        Self {
            cell_type,
            data_writer: bit_writer,
            data_bits_len: 0,
            refs: TonCellRefsStore::new(),
        }
    }

    pub fn build(self) -> TonCellResult<CellOwned> {
        let (mut data, data_bits_len) = build_cell_data(self.data_writer)?;
        let meta = CellMeta::new(self.cell_type, &data, data_bits_len, &self.refs)?;
        data.shrink_to_fit();
        Ok(CellOwned {
            meta,
            data,
            data_bits_len,
            refs: self.refs,
        })
    }

    pub fn write_bit(&mut self, data: bool) -> TonCellResult<&mut Self> {
        self.ensure_capacity(1)?;
        self.data_writer.write_bit(data)?;
        self.data_bits_len += 1;
        Ok(self)
    }

    pub fn write_bits<T: AsRef<[u8]>>(&mut self, data: T, bits_len: u32) -> TonCellResult<&mut Self> {
        self.ensure_capacity(bits_len)?;
        let data_ref = data.as_ref();
        let full_bytes = bits_len as usize / 8;
        self.data_writer.write_bytes(&data_ref[0..full_bytes])?;
        let rest_bits_len = bits_len % 8;
        if rest_bits_len != 0 {
            let mut last_byte = 0;
            if full_bytes < data_ref.len() {
                last_byte = data_ref[full_bytes] >> (8 - rest_bits_len);
            }
            self.data_writer.write(rest_bits_len, last_byte)?;
        }
        Ok(self)
    }

    pub fn write_byte(&mut self, data: u8) -> TonCellResult<&mut Self> { self.write_bits([data], 8) }

    pub fn write_bytes<T: AsRef<[u8]>>(&mut self, data: T) -> TonCellResult<&mut Self> {
        let data_ref = data.as_ref();
        self.write_bits(data_ref, data_ref.len() as u32 * 8)?;
        Ok(self)
    }

    pub fn write_num<N: TonNumber>(&mut self, data: N, bits_len: u32) -> TonCellResult<&mut Self> {
        self.ensure_capacity(bits_len)?;
        let unsigned_data = data.to_unsigned();
        self.data_writer.write(bits_len, unsigned_data)?;
        Ok(self)
    }

    pub fn write_ref(&mut self, cell: TonCellRef) -> TonCellResult<&mut Self> {
        if self.refs.len() >= CellMeta::CELL_MAX_REFS_COUNT {
            return Err(TonCellError::BuilderRefsOverflow);
        }
        self.refs.push(cell);
        Ok(self)
    }

    fn ensure_capacity(&mut self, bits_len: u32) -> TonCellResult<()> {
        let bits_left = CellMeta::CELL_MAX_DATA_BITS_LEN - self.data_bits_len as u32;
        if bits_len <= bits_left {
            return Ok(());
        }
        Err(TonCellError::BuilderDataOverflow {
            requested: bits_len,
            left: bits_left,
        })
    }
}

fn build_cell_data(mut bit_writer: BitWriter<Vec<u8>, BigEndian>) -> TonCellResult<(Vec<u8>, usize)> {
    let mut trailing_zeros = 0;
    while !bit_writer.byte_aligned() {
        bit_writer.write_bit(false)?;
        trailing_zeros += 1;
    }
    let data = bit_writer.into_writer();
    let bits_len = data.len() * 8 - trailing_zeros;
    Ok((data, bits_len))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::cell::meta::level_mask::LevelMask;
    use crate::cell::ton_cell::{TonCell, TonCellRefsStore};
    use crate::cell::ton_hash::TonHash;
    use hex::FromHex;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_builder_write_bit() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_0000]);
        assert_eq!(cell.data_bits_len, 4);
        Ok(())
    }

    #[test]
    fn test_builder_write_bits() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_bit(true)?;
        cell_builder.write_bits([0b1010_1010], 8)?;
        cell_builder.write_bits([0b0101_0101], 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1101_0101, 0b0010_1000]);
        assert_eq!(cell.data_bits_len, 13);
        Ok(())
    }

    #[test]
    fn test_builder_write_byte() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_byte(0b1010_1010)?;
        cell_builder.write_byte(0b0101_0101)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010, 0b0101_0101]);
        assert_eq!(cell.data_bits_len, 16);
        Ok(())
    }

    #[test]
    fn test_builder_write_bytes() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_bytes([0b1010_1010, 0b0101_0101])?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010, 0b0101_0101]);
        assert_eq!(cell.data_bits_len, 16);
        Ok(())
    }

    #[test]
    fn test_builder_write_data_overflow() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_bit(true)?;
        assert!(cell_builder.write_bits([0b1010_1010], CellMeta::CELL_MAX_DATA_BITS_LEN).is_err());
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_num(0b1010_1010, 8)?;
        cell_builder.write_num(0b0000_0101, 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010, 0b0101_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_num(1u8, 4)?;
        cell_builder.write_num(2u16, 5)?;
        cell_builder.write_num(5u32, 10)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b0001_0001, 0b0000_0000, 0b1010_0000]);
        assert_eq!(cell.data_bits_len, 19);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        assert!(cell_builder.write_num(-3i32, 3).is_err());
        assert!(cell_builder.write_num(-3i32, 31).is_err());
        cell_builder.write_num(-3i16, 16)?;
        cell_builder.write_num(-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1111_1111, 0b1111_1101, 0b1111_1101]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_bit(false)?;
        cell_builder.write_num(-3i16, 16)?;
        cell_builder.write_num(-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b0111_1111, 0b1111_1110, 0b1111_1110, 0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_refs() -> anyhow::Result<()> {
        let cell_ref = Arc::new(CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0b1111_0000],
            data_bits_len: 4,
            refs: TonCellRefsStore::new(),
        });

        let mut cell_builder = TonCellBuilder::new();
        cell_builder.write_ref(cell_ref.clone())?;
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.refs.len(), 2);
        assert_eq!(cell.refs[0].get_data(), cell_ref.data);
        assert_eq!(cell.refs[1].get_data(), cell_ref.data);
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_ordinary_empty() -> anyhow::Result<()> {
        let cell_builder = TonCellBuilder::new();
        let cell = cell_builder.build()?;
        assert_eq!(cell, CellOwned::EMPTY);
        for level in 0..4 {
            assert_eq!(cell.hash_for_level(LevelMask::new(level)), &TonHash::EMPTY_CELL_HASH);
        }
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_ordinary_non_empty() -> anyhow::Result<()> {
        //          0
        //        /   \
        //       1     2
        //      /
        //    3 4
        //   /
        //  5
        let mut builder5 = TonCellBuilder::new();
        builder5.write_byte(0x05)?;
        let cell5 = builder5.build()?;

        let mut builder3 = TonCellBuilder::new();
        builder3.write_byte(0x03)?;
        builder3.write_ref(cell5.clone().into())?;
        let cell3 = builder3.build()?;

        let mut builder4 = TonCellBuilder::new();
        builder4.write_byte(0x04)?;
        let cell4 = builder4.build()?;

        let mut builder2 = TonCellBuilder::new();
        builder2.write_byte(0x02)?;
        let cell2 = builder2.build()?;

        let mut builder1 = TonCellBuilder::new();
        builder1.write_byte(0x01)?;
        builder1.write_ref(cell3.clone().into())?;
        builder1.write_ref(cell4.clone().into())?;
        let cell1 = builder1.build()?;

        let mut builder0 = TonCellBuilder::new();
        builder0.write_bit(true)?;
        builder0.write_byte(0b0000_0001)?;
        builder0.write_byte(0b0000_0011)?;
        builder0.write_ref(cell1.clone().into())?;
        builder0.write_ref(cell2.clone().into())?;
        let cell0 = builder0.build()?;

        assert_eq!(cell0.get_refs().len(), 2);
        assert_eq!(cell0.data_bits_len, 17);
        assert_eq!(cell0.data, vec![0b1000_0000, 0b1000_0001, 0b1000_0000]);

        let exp_hash = TonHash::from_hex("5d64a52c76eb32a63a393345a69533f095f945f2d30f371a1f323ac10102c395")?;
        for level in 0..4 {
            assert_eq!(cell0.hash_for_level(LevelMask::new(level)), &exp_hash);
            assert_eq!(cell0.get_meta().depths[level as usize], 3);
        }
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_library() -> anyhow::Result<()> {
        let mut builder = TonCellBuilder::new_with_type(CellType::Library);
        builder.write_bytes(TonHash::ZERO)?;
        assert_err!(builder.build()); // no lib prefix

        let mut builder = TonCellBuilder::new_with_type(CellType::Library);
        builder.write_byte(2)?; // lib prefix https://docs.ton.org/v3/documentation/data-formats/tlb/exotic-cells#library-reference
        builder.write_bytes(TonHash::ZERO)?;
        let lib_cell = assert_ok!(builder.build());

        let expected_hash = TonHash::from_hex("6f3fd5de541ec62d350d30785ada554a2b13b887a3e4e51896799d0b0c46c552")?;
        for level in 0..4 {
            assert_eq!(lib_cell.hash_for_level(LevelMask::new(level)), &expected_hash);
            assert_eq!(lib_cell.get_meta().depths[level as usize], 0);
        }
        Ok(())
    }
}
