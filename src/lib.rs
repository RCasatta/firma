pub mod common;

pub use common::cmd::*;
pub use common::error::*;
pub use common::json::*;
// pub use common::*;

pub type Result<R> = std::result::Result<R, Error>;
pub type PSBT = bitcoin::util::psbt::PartiallySignedTransaction;
