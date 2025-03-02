use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TonCellError {
    // cell_parser
    #[error("Requested {requested} bits, but only {left} left")]
    ParserDataUnderflow { requested: u32, left: u32 },
    #[error("New position is {new_position}, but data_bits_len is {data_bits_len}")]
    ParserBadPosition { new_position: i32, data_bits_len: u32 },
    #[error("No ref with index={requested}")]
    ParserRefsUnderflow { requested: u8 },
    #[error("Cell is not empty: {bits_left} bits left")]
    ParserCellNotEmpty { bits_left: u32 },
    #[error("Container is too small: {requested} bits requested, but only {available} available")]
    ParserSmallContainer { requested: u32, available: u32 },

    // cell_builder
    #[error("Can't write {requested} bits: only {left} free bits available")]
    BuilderDataOverflow { requested: u32, left: u32 },
    #[error("Can't write ref - 4 refs are written already")]
    BuilderRefsOverflow,
    // meta, kinda cell_builder
    #[error("Cell validation error: {0}")]
    BuilderMeta(String),

    // boc
    #[error("Expected single root, but {0} roots presented")]
    BocSingleRoot(usize),
    #[error("Fail to decode {src}: {err}")]
    BocDecode { src: String, err: String },

    // ton_hash
    #[error("Expecting {expected} bytes, got {given}")]
    TonHashWrongLen { expected: usize, given: usize },

    // handling external errors
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    FromHex(#[from] FromHexError),
}

pub type TonCellResult<T> = Result<T, TonCellError>;
