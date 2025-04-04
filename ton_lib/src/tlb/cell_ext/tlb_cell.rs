use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::cell_type::CellType;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use crate::tlb::tlb_type::TLBType;

impl TLBType for TonCell {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let bits_left = parser.data_bits_left()?;

        if parser.cell.data_bits_len == bits_left as usize && parser.next_ref_pos == 0 {
            let _data = parser.read_bits(bits_left)?; // drain data from parser

            let mut refs = Vec::with_capacity(parser.cell.refs.len());
            for _ in 0..parser.cell.refs.len() {
                refs.push(parser.read_next_ref()?.clone());
            }

            Ok(Self {
                meta: parser.cell.meta.clone(),
                data: parser.cell.data.to_vec(),
                data_bits_len: parser.cell.data_bits_len,
                refs,
            })
        } else {
            let data = parser.read_bits(bits_left)?;

            let mut refs = vec![];
            while let Ok(ref_cell) = parser.read_next_ref() {
                refs.push(ref_cell.clone());
            }
            let meta = CellMeta::new(CellType::Ordinary, &data, bits_left as usize, &refs)?;
            Ok(Self {
                meta,
                data,
                data_bits_len: bits_left as usize,
                refs,
            })
        }
    }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_bits(&self.data, self.data_bits_len as u32)?;
        self.refs.iter().cloned().try_for_each(|r| builder.write_ref(r))
    }

    fn to_cell(&self) -> Result<TonCell, TonLibError> { Ok(self.clone()) }
}

impl TLBType for TonCellRef {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> { Ok(parser.read_next_ref()?.clone()) }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> { builder.write_ref(self.clone()) }
}

impl TLBType for TonHash {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        TonHash::from_vec(parser.read_bytes(TonHash::BYTES_LEN as u32)?)
    }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> { builder.write_bytes(self.as_slice()) }
}
