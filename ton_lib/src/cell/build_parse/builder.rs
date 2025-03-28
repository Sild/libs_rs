use crate::cell::cell_owned::CellOwned;
use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::cell_type::CellType;
use crate::cell::ton_cell::{TonCell, TonCellRef, TonCellRefsStore};
use crate::cell::ton_number::traits::{TonBigNumber, TonNumber};
use crate::errors::TonLibError;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use std::cmp::min;

pub struct CellBuilder {
    cell_type: CellType,
    data_writer: BitWriter<Vec<u8>, BigEndian>,
    data_bits_len: usize,
    refs: TonCellRefsStore,
}

impl Default for CellBuilder {
    fn default() -> Self { Self::new() }
}

impl CellBuilder {
    pub fn new() -> Self { Self::new_with_type(CellType::Ordinary) }

    pub fn new_with_type(cell_type: CellType) -> Self {
        Self {
            cell_type,
            data_writer: BitWriter::endian(vec![], BigEndian),
            data_bits_len: 0,
            refs: TonCellRefsStore::new(),
        }
    }

    pub fn build(self) -> Result<CellOwned, TonLibError> {
        let (data, data_bits_len) = build_cell_data(self.data_writer)?;
        let meta = CellMeta::new(self.cell_type, &data, data_bits_len, &self.refs)?;
        Ok(CellOwned::new(meta, data, data_bits_len, self.refs))
    }

    pub fn write_bit(&mut self, data: bool) -> Result<(), TonLibError> {
        self.ensure_capacity(1)?;
        self.data_writer.write_bit(data)?;
        Ok(())
    }

    /// expecting data.len() * 8 >= (bits_offset + bits_len)
    pub fn write_bits_with_offset<T: AsRef<[u8]>>(
        &mut self,
        data: T,
        mut bits_len: u32,
        mut bits_offset: u32,
    ) -> Result<(), TonLibError> {
        self.ensure_capacity(bits_len)?;
        let mut data_ref = data.as_ref();

        if (bits_len + bits_offset + 7) / 8 > data_ref.len() as u32 {
            return Err(TonLibError::BuilderNotEnoughData {
                required_bits: bits_len + bits_offset,
                given: data_ref.len() as u32,
            });
        }

        if bits_len == 0 {
            return Ok(());
        }

        // skip bytes_offset, adjust borders
        data_ref = &data_ref[bits_offset as usize / 8..];
        bits_offset %= 8;

        let first_byte_bits_len = min(bits_len, 8 - bits_offset);
        let mut first_byte_val = data_ref[0] << bits_offset >> bits_offset;
        if first_byte_bits_len == bits_len {
            first_byte_val >>= 8 - bits_offset - bits_len
        }
        self.data_writer.write(first_byte_bits_len, first_byte_val)?;

        data_ref = &data_ref[1..];
        bits_len -= first_byte_bits_len;

        let full_bytes = bits_len as usize / 8;
        self.data_writer.write_bytes(&data_ref[0..full_bytes])?;
        let rest_bits_len = bits_len % 8;
        if rest_bits_len != 0 {
            self.data_writer.write(rest_bits_len, data_ref[full_bytes] >> (8 - rest_bits_len))?;
        }
        self.data_bits_len += bits_len as usize;
        Ok(())
    }

    pub fn write_bits<T: AsRef<[u8]>>(&mut self, data: T, bits_len: u32) -> Result<(), TonLibError> {
        self.write_bits_with_offset(data, bits_len, 0)
    }

    pub fn write_bytes<T: AsRef<[u8]>>(&mut self, data: T) -> Result<(), TonLibError> {
        let data_ref = data.as_ref();
        self.write_bits(data_ref, data_ref.len() as u32 * 8)?;
        Ok(())
    }

    pub fn write_byte(&mut self, data: u8) -> Result<(), TonLibError> { self.write_bytes([data]) }

    pub fn write_cell(&mut self, cell: &dyn TonCell) -> Result<(), TonLibError> {
        self.write_bits(cell.get_data(), cell.get_data_bits_len() as u32)?;
        for i in 0..cell.refs_count() {
            self.write_ref(cell.get_ref(i).unwrap().clone())?;
        }
        Ok(())
    }

    pub fn write_ref(&mut self, cell: TonCellRef) -> Result<(), TonLibError> {
        if self.refs.len() >= CellMeta::CELL_MAX_REFS_COUNT {
            return Err(TonLibError::BuilderRefsOverflow);
        }
        self.refs.push(cell);
        Ok(())
    }

    pub fn write_num<N: TonNumber>(&mut self, data: N, bits_len: u32) -> Result<(), TonLibError> {
        self.ensure_capacity(bits_len)?;
        let unsigned_data = data.to_unsigned();
        self.data_writer.write(bits_len, unsigned_data)?;
        Ok(())
    }

    pub fn write_big_num<N: TonBigNumber>(&mut self, data: &N, bits_len: u32) -> Result<(), TonLibError> {
        self.ensure_capacity(bits_len)?;

        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
        if bits_len == 0 {
            if data.is_zero() {
                return Ok(());
            }
            return Err(TonLibError::BuilderNumberBitsMismatch {
                number: format!("{data}"),
                bits: bits_len,
            });
        }

        let min_bits_len = data.min_bits_len();
        if min_bits_len > bits_len {
            return Err(TonLibError::BuilderNumberBitsMismatch {
                number: format!("{data}"),
                bits: bits_len,
            });
        }

        if N::SIGNED {
            self.write_bit(data.is_negative())?;
            if bits_len == 1 {
                return Ok(());
            }
        }

        let mag_bits_to_write = bits_len - N::SIGNED as u32;
        let min_bits_len_unsigned = min_bits_len - N::SIGNED as u32;

        let padding_bits_len = mag_bits_to_write.saturating_sub(min_bits_len_unsigned);
        let padding_to_write = vec![0; (padding_bits_len as usize + 7) / 8];
        self.write_bits(padding_to_write, padding_bits_len)?;

        let data_bytes = data.to_unsigned_bytes_be();
        let bits_offset = (data_bytes.len() as u32 * 8).saturating_sub(min_bits_len_unsigned);
        self.write_bits_with_offset(data_bytes, mag_bits_to_write - padding_bits_len, bits_offset)
    }

    fn ensure_capacity(&mut self, bits_len: u32) -> Result<(), TonLibError> {
        let new_bits_len = self.data_bits_len as u32 + bits_len;
        if new_bits_len <= CellMeta::CELL_MAX_DATA_BITS_LEN {
            self.data_bits_len = new_bits_len as usize;
            return Ok(());
        }
        Err(TonLibError::BuilderDataOverflow {
            req: bits_len,
            left: CellMeta::CELL_MAX_DATA_BITS_LEN - bits_len,
        })
    }
}

fn build_cell_data(mut bit_writer: BitWriter<Vec<u8>, BigEndian>) -> Result<(Vec<u8>, usize), TonLibError> {
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
#[cfg(feature = "num-bigint")]
#[cfg(feature = "fastnum")]
mod tests {
    use super::*;
    use crate::cell::meta::level_mask::LevelMask;
    use crate::cell::ton_cell::{TonCell, TonCellRefsStore};
    use crate::cell::ton_hash::TonHash;
    use hex::FromHex;
    use std::str::FromStr;

    use std::sync::Arc;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_builder_write_bit() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1010_0000]);
        assert_eq!(cell.get_data_bits_len(), 4);
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_with_offset() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bits_with_offset([0b1010_1010], 8, 0)?;
        cell_builder.write_bits_with_offset([0b0000_1111], 4, 4)?;
        cell_builder.write_bits_with_offset([0b1111_0011], 3, 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1010_1010, 0b1111_0010]);
        assert_eq!(cell.get_data_bits_len(), 15);

        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bits_with_offset([0b1010_1010, 0b0000_1111], 3, 10)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b0010_0000]);
        assert_eq!(cell.get_data_bits_len(), 3);
        Ok(())
    }

    #[test]
    fn test_builder_write_bits() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bit(true)?;
        cell_builder.write_bits([0b1010_1010], 8)?;
        cell_builder.write_bits([0b0101_0101], 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1101_0101, 0b0010_1000]);
        assert_eq!(cell.get_data_bits_len(), 13);
        Ok(())
    }

    #[test]
    fn test_builder_write_byte() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_byte(0b1010_1010)?;
        cell_builder.write_byte(0b0101_0101)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1010_1010, 0b0101_0101]);
        assert_eq!(cell.get_data_bits_len(), 16);
        Ok(())
    }

    #[test]
    fn test_builder_write_bytes() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bytes([0b1010_1010, 0b0101_0101])?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1010_1010, 0b0101_0101]);
        assert_eq!(cell.get_data_bits_len(), 16);
        Ok(())
    }

    #[test]
    fn test_builder_write_data_overflow() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bit(true)?;
        assert!(cell_builder.write_bits([0b1010_1010], CellMeta::CELL_MAX_DATA_BITS_LEN).is_err());
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_num(0b1010_1010, 8)?;
        cell_builder.write_num(0b0000_0101, 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1010_1010, 0b0101_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_in_many_bits() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        assert_err!(cell_builder.write_num(0b1010_1010u8, 16));
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_num(1u8, 4)?;
        cell_builder.write_num(2u16, 5)?;
        cell_builder.write_num(5u32, 10)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b0001_0001, 0b0000_0000, 0b1010_0000]);
        assert_eq!(cell.get_data_bits_len(), 19);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        assert!(cell_builder.write_num(-3i32, 3).is_err());
        assert!(cell_builder.write_num(-3i32, 31).is_err());
        cell_builder.write_num(-3i16, 16)?;
        cell_builder.write_num(-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b1111_1111, 0b1111_1101, 0b1111_1101]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = CellBuilder::new();
        cell_builder.write_bit(false)?;
        cell_builder.write_num(-3i16, 16)?;
        cell_builder.write_num(-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.get_data(), vec![0b0111_1111, 0b1111_1110, 0b1111_1110, 0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_cell() -> anyhow::Result<()> {
        let mut ref_builder = CellBuilder::new();
        ref_builder.write_bit(true)?;
        ref_builder.write_bytes([1, 2, 3])?;
        let ref_cell = ref_builder.build()?.into_ref();

        let mut cell_with_ref_builder = CellBuilder::new();
        cell_with_ref_builder.write_bit(true)?;
        cell_with_ref_builder.write_ref(ref_cell.clone())?;
        let cell_with_ref = cell_with_ref_builder.build()?;

        let mut cell_builder = CellBuilder::new();
        cell_builder.write_cell(&cell_with_ref)?;
        let cell = cell_builder.build()?;

        assert_eq!(cell, cell_with_ref);
        Ok(())
    }

    #[test]
    fn test_builder_write_refs() -> anyhow::Result<()> {
        let cell_ref =
            Arc::new(CellOwned::new(CellMeta::EMPTY_CELL_META, vec![0b1111_0000], 4, TonCellRefsStore::new()));

        let mut cell_builder = CellBuilder::new();
        cell_builder.write_ref(cell_ref.clone())?;
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.refs_count(), 2);
        assert_eq!(cell.get_ref(0).unwrap().get_data(), cell_ref.get_data());
        assert_eq!(cell.get_ref(0).unwrap().get_data(), cell_ref.get_data());
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_ordinary_empty() -> anyhow::Result<()> {
        let cell_builder = CellBuilder::new();
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
        let mut builder5 = CellBuilder::new();
        builder5.write_byte(0x05)?;
        let cell5 = builder5.build()?;

        let mut builder3 = CellBuilder::new();
        builder3.write_byte(0x03)?;
        builder3.write_ref(cell5.clone().into())?;
        let cell3 = builder3.build()?;

        let mut builder4 = CellBuilder::new();
        builder4.write_byte(0x04)?;
        let cell4 = builder4.build()?;

        let mut builder2 = CellBuilder::new();
        builder2.write_byte(0x02)?;
        let cell2 = builder2.build()?;

        let mut builder1 = CellBuilder::new();
        builder1.write_byte(0x01)?;
        builder1.write_ref(cell3.clone().into())?;
        builder1.write_ref(cell4.clone().into())?;
        let cell1 = builder1.build()?;

        let mut builder0 = CellBuilder::new();
        builder0.write_bit(true)?;
        builder0.write_byte(0b0000_0001)?;
        builder0.write_byte(0b0000_0011)?;
        builder0.write_ref(cell1.clone().into())?;
        builder0.write_ref(cell2.clone().into())?;
        let cell0 = builder0.build()?;

        assert_eq!(cell0.refs_count(), 2);
        assert_eq!(cell0.get_data_bits_len(), 17);
        assert_eq!(cell0.get_data(), vec![0b1000_0000, 0b1000_0001, 0b1000_0000]);

        let exp_hash = TonHash::from_hex("5d64a52c76eb32a63a393345a69533f095f945f2d30f371a1f323ac10102c395")?;
        for level in 0..4 {
            assert_eq!(cell0.hash_for_level(LevelMask::new(level)), &exp_hash);
            assert_eq!(cell0.get_meta().depths[level as usize], 3);
        }
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_library() -> anyhow::Result<()> {
        let mut builder = CellBuilder::new_with_type(CellType::Library);
        builder.write_bytes(TonHash::ZERO)?;
        assert_err!(builder.build()); // no ton_lib prefix

        let mut builder = CellBuilder::new_with_type(CellType::Library);
        builder.write_byte(2)?; // ton_lib prefix https://docs.ton.org/v3/documentation/data-formats/tlb/exotic-cells#library-reference
        builder.write_bytes(TonHash::ZERO)?;
        let lib_cell = assert_ok!(builder.build());

        let expected_hash = TonHash::from_hex("6f3fd5de541ec62d350d30785ada554a2b13b887a3e4e51896799d0b0c46c552")?;
        for level in 0..4 {
            assert_eq!(lib_cell.hash_for_level(LevelMask::new(level)), &expected_hash);
            assert_eq!(lib_cell.get_meta().depths[level as usize], 0);
        }
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_not_enough() -> anyhow::Result<()> {
        let mut builder = CellBuilder::new();
        let data = vec![1u8; 2];
        assert_err!(builder.write_bits(&data, 32));
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_not_enough_unaligned() -> anyhow::Result<()> {
        let mut builder = CellBuilder::new();
        let data = vec![1u8; 2];
        assert_err!(builder.write_bits(&data, 33));
        Ok(())
    }

    #[test]
    fn test_builder_write_num_bigint() -> anyhow::Result<()> {
        let prepare_cell = |num_str: &str, bits_len: u32| {
            let number = num_bigint::BigInt::from_str(num_str)?;
            let mut builder = CellBuilder::new();
            builder.write_bits([0], 7)?; // for pretty printing
            builder.write_big_num(&number, bits_len)?;
            let cell = builder.build()?;
            Ok::<_, anyhow::Error>(cell)
        };

        let cell = prepare_cell("3", 33)?;
        assert_eq!(cell.get_data(), [0, 0, 0, 0, 3]);

        // 256 bits (+ sign)
        let cell = prepare_cell("97887266651548624282413032824435501549503168134499591480902563623927645013201", 257)?;
        assert_eq!(
            cell.get_data(),
            [
                0, 216, 106, 58, 195, 97, 8, 173, 64, 195, 26, 52, 186, 72, 230, 253, 248, 12, 245, 147, 137, 170, 38,
                117, 66, 220, 74, 104, 103, 119, 137, 4, 209
            ]
        );

        let cell = prepare_cell("-5", 257)?;
        assert_eq!(
            cell.get_data(),
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5]
        );

        let cell = prepare_cell("-5", 33)?;
        assert_eq!(cell.get_data(), [1, 0, 0, 0, 5]);

        let cell = prepare_cell("-5", 4)?;
        assert_eq!(cell.get_data(), [1, 160]);

        let cell = prepare_cell("-5", 5)?;
        assert_eq!(cell.get_data(), [1, 80]);
        Ok(())
    }

    fn prepare_cell_big_uint(num_str: &str, bits_len: u32) -> anyhow::Result<CellOwned> {
        let number = num_bigint::BigUint::from_str(num_str)?;
        let mut builder = CellBuilder::new();
        builder.write_bits([0], 7)?; // for pretty printing
        builder.write_big_num(&number, bits_len)?;
        let cell = builder.build()?;
        Ok(cell)
    }

    #[test]
    fn test_builder_write_num_biguint() -> anyhow::Result<()> {
        let prepare_cell = |num_str: &str, bits_len: u32| {
            let number = num_bigint::BigUint::from_str(num_str)?;
            let mut builder = CellBuilder::new();
            builder.write_bits([0], 7)?; // for pretty printing
            builder.write_big_num(&number, bits_len)?;
            let cell = builder.build()?;
            Ok::<_, anyhow::Error>(cell)
        };

        let cell = prepare_cell("3", 33)?;
        assert_eq!(cell.get_data(), [0, 0, 0, 0, 3]);

        // 256 bits (+ sign)
        let cell = prepare_cell_big_uint(
            "97887266651548624282413032824435501549503168134499591480902563623927645013201",
            257,
        )?;
        assert_eq!(
            cell.get_data(),
            [
                0, 216, 106, 58, 195, 97, 8, 173, 64, 195, 26, 52, 186, 72, 230, 253, 248, 12, 245, 147, 137, 170, 38,
                117, 66, 220, 74, 104, 103, 119, 137, 4, 209
            ]
        );
        Ok(())
    }

    #[test]
    fn test_builder_write_bignum_zero() -> anyhow::Result<()> {
        let number = num_bigint::BigInt::from_str("0")?;
        let mut builder = CellBuilder::new();
        assert_ok!(builder.write_big_num(&number, 0));
        assert_ok!(builder.write_big_num(&number, 1));
        assert_ok!(builder.write_big_num(&number, 2));

        let number = num_bigint::BigUint::from_str("0")?;
        let mut builder = CellBuilder::new();
        assert_ok!(builder.write_big_num(&number, 0));
        assert_ok!(builder.write_big_num(&number, 1));
        assert_ok!(builder.write_big_num(&number, 2));

        Ok(())
    }
}
