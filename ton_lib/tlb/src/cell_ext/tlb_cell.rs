use crate::errors::TLBResult;
use crate::tlb_type::TLBType;
use std::sync::Arc;
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;
use ton_lib_cell::cell::cell_owned::CellOwned;
use ton_lib_cell::cell::meta::cell_meta::CellMeta;
use ton_lib_cell::cell::meta::cell_type::CellType;
use ton_lib_cell::cell::ton_cell::ArcTonCell;
use ton_lib_cell::cell::ton_hash::TonHash;

impl TLBType for CellOwned {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        if parser.cell.get_data_bits_len() == parser.data_bits_left()? as usize && parser.next_ref_pos == 0 {
            Ok(Self {
                meta: parser.cell.get_meta().clone(),
                data: parser.cell.get_data().to_vec(),
                data_bits_len: parser.cell.get_data_bits_len(),
                refs: parser.cell.get_refs().to_vec(),
            })
        } else {
            let (data, data_bits_len) = parser.read_rest_data()?;
            let mut refs = vec![];
            while let Ok(ref_cell) = parser.read_next_ref() {
                refs.push(ref_cell.clone());
            }
            let meta = CellMeta::new(CellType::Ordinary, &data, data_bits_len as usize, &refs)?;
            Ok(Self {
                meta,
                data,
                data_bits_len: data_bits_len as usize,
                refs,
            })
        }
    }

    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        builder.write_bits(&self.data, self.data_bits_len as u32)?;
        for ref_cell in &self.refs {
            builder.write_ref(ref_cell.clone())?;
        }
        Ok(())
    }
}

impl TLBType for ArcTonCell {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        let cell = CellOwned::read(parser)?;
        Ok(Arc::new(cell))
    }

    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        self.write(builder)?;
        Ok(())
    }
}

impl TLBType for TonHash {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        let mut data = [0; TonHash::BYTES_LEN];
        parser.read_bytes(&mut data)?;
        Ok(TonHash::from(data))
    }

    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        builder.write_bytes(self.as_slice())?;
        Ok(())
    }
}
