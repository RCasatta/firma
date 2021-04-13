//#![warn(missing_docs)]

pub mod common;
pub mod entities;
pub mod offline;
pub mod online;

#[cfg(target_os = "android")]
mod android;

pub use common::context::*;
pub use common::error::*;
pub use common::*;
pub use entities::*;

// Re-exports
pub use bitcoin;
pub use bitcoincore_rpc;
pub use log;
pub use serde;
pub use serde_json;
pub use structopt;

pub type Result<R> = std::result::Result<R, Error>;
pub type BitcoinPsbt = bitcoin::util::psbt::PartiallySignedTransaction;
