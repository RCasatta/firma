//#![warn(missing_docs)]

pub mod common;
pub mod offline;
pub mod online;

#[cfg(target_os = "android")]
mod android;

pub use common::context::*;
pub use common::error::*;
pub use common::json::*;
pub use common::*;

// Re-exports
pub use bitcoin;
pub use bitcoincore_rpc;
pub use log;
pub use serde;
pub use serde_json;
pub use structopt;

pub type Result<R> = std::result::Result<R, Error>;
pub type PSBT = bitcoin::util::psbt::PartiallySignedTransaction;
