use thiserror::Error;
use ton_lib_cell::errors::TonCellError;
use ton_lib_tlb::errors::TLBError;

#[derive(Error, Debug)]
pub enum TonTypesError {
    #[error("{0}")]
    TonCellError(#[from] TonCellError),
    #[error("{0}")]
    TLBError(#[from] TLBError),
}

pub type TonTypesResult<T> = Result<T, TonTypesError>;
