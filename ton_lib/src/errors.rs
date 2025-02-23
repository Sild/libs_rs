use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TonLibError {
    // cell_parser
    #[error("Requested {requested} bits, but only {left} left")]
    CellParserDataUnderflow { requested: u32, left: u32 },
    #[error("New position is {new_position}, but data_bits_len is {data_bits_len}")]
    CellParserBadPosition { new_position: i32, data_bits_len: u32 },
    #[error("No ref with index={requested}")]
    CellParserRefsUnderflow { requested: u8 },
    #[error("Cell is not empty: {bits_left} bits left")]
    CellParserCellNotEmpty { bits_left: u32 },
    #[error("Container is too small: {requested} bits requested, but only {available} available")]
    CellParserSmallContainer { requested: u32, available: u32 },

    // cell_builder
    #[error("Can't write {requested} bits: only {left} free bits available")]
    CellBuilderDataOverflow { requested: u32, left: u32 },
    #[error("Can't write ref - 4 refs are written already")]
    CellBuilderRefsOverflow,
    // meta, kinda cell_builder
    #[error("Cell validation error: {0}")]
    CellBuilderMeta(String),

    // boc
    #[error("Expected single root, but {0} roots presented")]
    BocSingleRoot(usize),
    #[error("Fail to decode {src}: {err}")]
    BocDecode { src: String, err: String },

    // tlb
    #[error("Expecting prefix {expected:.X}, but got {given:.X}")]
    TLBWrongPrefix { expected: u128, given: u128 },

    // ton_hash
    #[error("Expecting {expected} bytes, got {given}")]
    TonHashWrongLen { expected: usize, given: usize },

    // handling external errors
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    FromHex(#[from] FromHexError),
}

pub type TonLibResult<T> = Result<T, TonLibError>;
