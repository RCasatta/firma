use crate::*;
use log::info;
use qr_code::structured::SplittedQr;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct QrOptions {
    pub qr_content: StringEncoding,
    pub version: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrMergeOptions {
    pub qrs_content: Vec<StringEncoding>,
}

/// From a QR content return bytes arrays of encoded bitmaps conaining QRs
/// If `qr_content` does not fit in one QR given the `version` it creates multiple Qrs
pub fn qrs(qr_content: Vec<u8>, version: i16) -> Result<Vec<Vec<u8>>> {
    match version {
        5..=20 => info!("save_qrs data len:{} version:{}", qr_content.len(), version),
        _ => return Err(format!("invalid qr version {}", version).into()),
    }
    let qrs = SplittedQr::new(qr_content, version)?.split()?;
    let mut result = vec![];
    for qr in qrs {
        let bmp = qr.to_bmp().mul(4)?.add_white_border(12)?;
        let mut cursor = Cursor::new(vec![]);
        bmp.write(&mut cursor)?;
        result.push(cursor.into_inner());
    }
    Ok(result)
}

/// like `fn qr(...)` but callable using json's
pub fn qrs_string_encoding(opt: QrOptions) -> Result<EncodedQrs> {
    Ok(EncodedQrs {
        qrs: qrs(opt.qr_content.as_bytes()?, opt.version)?
            .iter()
            .map(|v| StringEncoding::new_hex(v))
            .collect(),
    })
}

pub fn merge_qrs(opt: QrMergeOptions) -> Result<StringEncoding> {
    let mut values = vec![];
    for el in opt.qrs_content {
        values.push(el.as_bytes()?);
    }
    let result = qr_code::structured::merge_qrs(values)?;
    Ok(StringEncoding::new_hex(&result))
}
