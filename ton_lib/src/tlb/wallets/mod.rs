mod utils;
mod wallet_v1_v2;
mod wallet_v3;

pub use wallet_v1_v2::*;
pub use wallet_v3::*;

// todo move to another place?
pub const DEFAULT_WALLET_ID: i32 = 0x29a9a317;
pub const DEFAULT_WALLET_ID_V5R1: i32 = 0x7FFFFF11;
pub const DEFAULT_WALLET_ID_V5R1_TESTNET: i32 = 0x7FFFFFFD;
