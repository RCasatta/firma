//#![warn(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub extern crate core_rpc as bitcoincore_rpc;
use bitcoincore_rpc::core_rpc_json as bitcoincore_rpc_json;

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
pub use log;
pub use serde;
pub use serde_json;
pub use structopt;

pub type Result<R> = std::result::Result<R, Error>;
pub type BitcoinPsbt = bitcoin::util::psbt::PartiallySignedTransaction;
