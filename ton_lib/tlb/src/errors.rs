use thiserror::Error;
use ton_lib_cell::errors::TonCellError;

#[derive(Error, Debug)]
pub enum TLBError {
    #[error("{0}")]
    TonCellError(#[from] TonCellError),

    #[error("Wrong prefix: expected={}, actual={}", expected, actual)]
    WrongPrefix { expected: u128, actual: u128 },
}

pub type TLBResult<T> = Result<T, TLBError>;
