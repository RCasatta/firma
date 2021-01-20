use crate::*;
use log::info;
use qr_code::structured::SplittedQr;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};

#[derive(Debug)]
pub enum QrError {}

//TODO instead of saving files, qr are returned as StringEncoding? less code around, everything is
// encrypted without qr-codes leaking things to be rendered it's possibile via qr_text or
// trough pipes | jq -r data | base64 -d >file.bmp
// but what about multiple qr-codes?

/// path contains up to the filename (use dummy value) that will be replaced by qr file name
pub fn save_qrs(bytes: Vec<u8>, qr_dir: PathBuf, version: i16) -> Result<Vec<PathBuf>> {
    match version {
        0 => return Ok(vec![]),
        5..=20 => info!("save_qrs data len:{} version:{}", bytes.len(), version),
        _ => return Err(format!("invalid qr version {}", version).into()),
    }

    let mut wallet_qr_files = vec![];

    let qrs = SplittedQr::new(bytes, version)?.split()?;
    info!("splitted qr in {} pieces", qrs.len());

    let mut text_qr = vec![String::new(); 2];
    let single = qrs.len() == 1;

    if qr_dir.exists() {
        // delete existing QR if any
        if qr_dir.is_dir() {
            info!("listing {:?}", qr_dir);
            for entry in std::fs::read_dir(&qr_dir)? {
                let entry = entry?;
                let path = entry.path();
                info!("deleting {:?}", path);
                fs::remove_file(path)?;
            }
        } else {
            return Err("save_qrs qr_dir is not a dir".into());
        }
    } else {
        fs::create_dir(&qr_dir)?;
    }

    let mut qr_file = qr_dir;
    qr_file.push("dummy");
    for (i, qr) in qrs.iter().enumerate() {
        if single {
            qr_file.set_file_name("qr.bmp");
        } else {
            qr_file.set_file_name(&format!("qr-{}.bmp", i));
        }
        let qr_data = qr.to_bmp().mul(4)?.add_white_border(12)?;
        info!("Saving qr in {:?}", &qr_file);
        qr_data.write(File::create(&qr_file)?)?;

        wallet_qr_files.push(qr_file.clone());

        for b in &[true, false] {
            let qr_txt = qr.to_string(*b, 3);
            text_qr[*b as usize].push_str(&qr_txt);
        }
    }
    qr_file.set_file_name("qrs.txt");
    info!("Saving qr in {:?}", &qr_file);
    let mut qr_txt_file = File::create(&qr_file)?;
    qr_txt_file.write_all(text_qr[0].as_bytes())?;
    qr_txt_file.write_all(text_qr[1].as_bytes())?;
    Ok(wallet_qr_files)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QrMode {
    Text { inverted: bool },
    Image,
    None,
}

impl Default for QrMode {
    fn default() -> Self {
        QrMode::None
    }
}

impl FromStr for QrMode {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "none" => Ok(QrMode::None),
            "image" => Ok(QrMode::Image),
            "text" => Ok(QrMode::Text { inverted: true }),
            "inverted" => Ok(QrMode::Text { inverted: false }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("({}) valid values are: none, image, text, inverted", s),
            )),
        }
    }
}
