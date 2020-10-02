use crate::*;
use bmp_monochrome::Bmp;
use log::info;
use qrcode::bits::{Bits, ExtendedMode};
use qrcode::types::Color::{Dark, Light};
use qrcode::{bits, EcLevel, QrCode, Version};
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub enum QrError {}

pub fn print_qr(qr_code: &QrCode, inverted: bool) -> Result<String> {
    let mut result = String::new();
    let width = qr_code.width();
    let mut qr_code = qr_code.clone().into_colors();
    let height = qr_code.len() / width;
    qr_code.extend(vec![Light; width]);

    let inverted = if inverted { 0 } else { 4 };
    let blocks = ["█", "▀", "▄", " ", " ", "▄", "▀", "█"];
    result.push_str("\n\n\n\n");
    for i in (0..height).step_by(2) {
        result.push_str(&format!(
            "{}{}{}",
            blocks[inverted], blocks[inverted], blocks[inverted]
        ));
        for j in 0..width {
            let start = i * width + j;
            let val = match (qr_code[start], qr_code.get(start + width).unwrap_or(&Light)) {
                (Light, Light) => 0,
                (Light, Dark) => 1,
                (Dark, Light) => 2,
                (Dark, Dark) => 3,
            };
            result.push_str(&blocks[val + inverted].to_string());
        }
        result.push_str("\n");
    }
    result.push_str("\n\n\n\n");
    Ok(result)
}

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
        let qr_data = to_matrix(qr);
        info!("Saving qr in {:?}", &qr_file);
        qr_data.write(File::create(&qr_file)?)?;

        wallet_qr_files.push(qr_file.clone());

        for b in &[true, false] {
            let qr_txt = print_qr(&qr, *b)?;
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

pub fn merge_qrs(mut bytes: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    use std::collections::HashSet;
    use std::convert::TryInto;

    let mut vec_structured = vec![];

    bytes.sort();
    bytes.dedup();

    if bytes.len() < 2 {
        return Err(Error::QrAtLeast2Pieces);
    }

    for vec in bytes {
        let current: StructuredQr = vec.try_into()?;
        vec_structured.push(current);
    }

    let total = (vec_structured.len() - 1) as u8;
    let totals_same = vec_structured.iter().map(|q| q.total).all(|t| t == total);
    if !totals_same {
        return Err(crate::Error::QrTotalMismatch(vec_structured.len()));
    }

    let sequences: HashSet<u8> = vec_structured.iter().map(|q| q.seq).collect();
    let all_sequence = sequences.len() == vec_structured.len();
    if !all_sequence {
        return Err(crate::Error::QrMissingParts);
    }

    vec_structured.sort_by(|a, b| a.seq.cmp(&b.seq)); // allows to merge out of order by reordering here
    let result: Vec<u8> = vec_structured
        .iter()
        .map(|q| q.content.clone())
        .flatten()
        .collect();

    let final_parity = result.iter().fold(0u8, |acc, &x| acc ^ x);
    if vec_structured
        .iter()
        .map(|q| q.parity)
        .all(|p| p == final_parity)
    {
        Ok(result)
    } else {
        Err(crate::Error::QrParity)
    }
}

struct StructuredQr {
    pub seq: u8,   // u4
    pub total: u8, // u4
    pub parity: u8,
    pub content: Vec<u8>,
}

impl TryFrom<Vec<u8>> for StructuredQr {
    type Error = crate::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        if value.len() < 5 {
            return Err(Error::QrTooShort);
        }
        let qr_mode = value[0] >> 4;
        if qr_mode != 3 {
            return Err(Error::QrStructuredWrongMode);
        }
        let seq = value[0] & 0x0f;
        let total = value[1] >> 4;
        if seq > total {
            return Err(Error::QrSeqGreaterThanTotal(seq, total));
        }
        let parity = ((value[1] & 0x0f) << 4) + (value[2] >> 4);
        let enc_mode = value[2] & 0x0f;
        if enc_mode != 4 {
            return Err(Error::QrStructuredWrongEnc);
        }

        let (length, from) = if value.len() < u8::max_value() as usize + 4 {
            // 4 is header size, TODO recheck boundary
            (value[3] as u16, 4usize)
        } else {
            (((value[3] as u16) << 8) + (value[4] as u16), 5usize)
        };
        let end = from + length as usize;
        if value.len() < end {
            return Err(crate::Error::QrLengthMismatch(end, value.len()));
        }
        let content = (&value[from..end]).to_vec();
        //TODO check padding

        Ok(StructuredQr {
            seq,
            total,
            parity,
            content,
        })
    }
}

pub struct SplittedQr {
    pub version: i16,
    pub parity: u8,
    pub total_qr: usize,
    pub bytes: Vec<u8>,
}

impl SplittedQr {
    pub fn new(bytes: Vec<u8>, version: i16) -> Result<Self> {
        let parity = bytes.iter().fold(0u8, |acc, &x| acc ^ x);
        let max_bytes = *MAX_BYTES
            .get(version as usize)
            .ok_or_else(|| Error::QrUnsupportedVersion(version))?;
        let extra = if bytes.len() % max_bytes == 0 { 0 } else { 1 };
        let total_qr = bytes.len() / max_bytes + extra;
        if total_qr > 16 {
            return Err(Error::QrSplitMax16(total_qr));
        }

        Ok(SplittedQr {
            bytes,
            version,
            parity,
            total_qr,
        })
    }

    fn split_to_bits(&self) -> Result<Vec<Bits>> {
        let max_bytes = MAX_BYTES[self.version as usize];
        if self.bytes.len() < max_bytes {
            let bits = bits::encode_auto(&self.bytes, LEVEL)?;
            Ok(vec![bits])
        } else {
            let mut result = vec![];
            for (i, chunk) in self.bytes.chunks(max_bytes).enumerate() {
                let bits = self.make_chunk(i, chunk)?;
                result.push(bits);
            }
            Ok(result)
        }
    }

    pub fn split(&self) -> Result<Vec<QrCode>> {
        self.split_to_bits()?
            .into_iter()
            .map(|bits| Ok(QrCode::with_bits(bits, LEVEL)?))
            .collect()
    }

    fn make_chunk(&self, i: usize, chunk: &[u8]) -> Result<Bits> {
        //println!("chunk len : {}", chunk.len());
        //println!("chunk : {}", hex::encode(chunk) );
        let mut bits = Bits::new(Version::Normal(self.version));
        bits.push_mode_indicator(ExtendedMode::StructuredAppend)?;
        bits.push_number_checked(4, i)?;
        bits.push_number_checked(4, self.total_qr - 1)?;
        bits.push_number_checked(8, self.parity as usize)?;
        bits.push_byte_data(chunk)?;
        bits.push_terminator(LEVEL)?;

        //println!("bits: {}\n", hex::encode(bits.clone().into_bytes()));

        Ok(bits)
    }
}

pub fn to_matrix(qr: &QrCode) -> Bmp {
    let width = qr.width();
    let data = qr
        .to_colors()
        .iter()
        .map(|e| match e {
            Light => false,
            Dark => true,
        })
        .collect();
    Bmp::new(data, width).unwrap()
}

const LEVEL: qrcode::types::EcLevel = EcLevel::L;

/// Max bytes encodable in a structured append qr code, given Qr code version as array index
const MAX_BYTES: [usize; 33] = [
    0, 15, 30, 51, 76, 104, 132, 152, 190, 228, 269, 319, 365, 423, 456, 518, 584, 642, 716, 790,
    856, 927, 1001, 1089, 1169, 1271, 1365, 1463, 1526, 1626, 1730, 1838, 1950,
];

#[cfg(test)]
mod tests {
    use crate::common::qr::{merge_qrs, print_qr, SplittedQr, StructuredQr, LEVEL};
    use crate::Error;
    use qrcode::bits::{Bits, ExtendedMode};
    use qrcode::{QrCode, Version};
    use rand::Rng;
    use std::convert::TryInto;

    // from example https://segno.readthedocs.io/en/stable/structured-append.html#structured-append
    /*
    I read the news today oh boy
    4 1c 49207265616420746865206e6577 7320746f646179206f6820626f79 000ec11ec

    I read the new
    3 0 1 39 4 0e 49207265616420746865206e6577 00

    s today oh boy
    3 1 1 39 4 0e 7320746f646179206f6820626f79 00

    MODE SEQ TOTAL PARITY MODE LENGTH
    */

    const _FULL: &str = "41c49207265616420746865206e65777320746f646179206f6820626f79000ec11ec";
    const FULL_CONTENT: &str = "49207265616420746865206e65777320746f646179206f6820626f79";
    const FIRST: &str = "3013940e49207265616420746865206e657700";
    const FIRST_CONTENT: &str = "49207265616420746865206e6577";
    const SECOND: &str = "3113940e7320746f646179206f6820626f7900";
    const SECOND_CONTENT: &str = "7320746f646179206f6820626f79";

    #[test]
    fn test_try_into_structured() {
        let bytes = hex::decode(FIRST).unwrap();
        let content = hex::decode(FIRST_CONTENT).unwrap();
        let structured: StructuredQr = bytes.try_into().unwrap();
        assert_eq!(structured.seq, 0);
        assert_eq!(structured.total, 1);
        assert_eq!(structured.parity, 57);
        assert_eq!(structured.content, content);

        let bytes = hex::decode(SECOND).unwrap();
        let content = hex::decode(SECOND_CONTENT).unwrap();
        let structured_2: StructuredQr = bytes.try_into().unwrap();
        assert_eq!(structured_2.seq, 1);
        assert_eq!(structured_2.total, 1);
        assert_eq!(structured_2.parity, 57);
        assert_eq!(structured_2.content, content);
    }

    #[test]
    fn test_merge() {
        let first = hex::decode(FIRST).unwrap();
        let second = hex::decode(SECOND).unwrap();
        let full_content = hex::decode(FULL_CONTENT).unwrap();
        let vec = vec![first.clone(), second.clone()];
        let result = merge_qrs(vec).unwrap();
        assert_eq!(hex::encode(result), FULL_CONTENT);

        let vec = vec![second.clone(), first.clone()];
        let result = merge_qrs(vec).unwrap(); //merge out of order
        assert_eq!(hex::encode(result), FULL_CONTENT);

        let vec = vec![second.clone(), first.clone(), second.clone()];
        let result = merge_qrs(vec).unwrap(); //merge duplicates
        assert_eq!(hex::encode(result), FULL_CONTENT);

        let vec = vec![first.clone(), first.clone()];
        let result = merge_qrs(vec);
        assert_eq!(
            result.unwrap_err().to_string(),
            Error::QrAtLeast2Pieces.to_string()
        );

        let vec = vec![first.clone(), full_content.clone()];
        let result = merge_qrs(vec);
        assert_eq!(
            result.unwrap_err().to_string(),
            Error::QrStructuredWrongMode.to_string()
        );

        let mut first_mut = first.clone();
        first_mut[15] = 14u8;
        let vec = vec![first.clone(), first_mut.clone()];
        let result = merge_qrs(vec);
        assert_eq!(
            result.unwrap_err().to_string(),
            Error::QrMissingParts.to_string()
        );

        let vec = vec![first.clone(), first_mut.clone(), second.clone()];
        let result = merge_qrs(vec);
        assert_eq!(
            result.unwrap_err().to_string(),
            Error::QrTotalMismatch(3).to_string(),
        );
    }

    #[test]
    fn test_structured_append() {
        let data = "I read the news today oh boy".as_bytes();
        let data_half = "I read the new".as_bytes();
        let parity = data.iter().fold(0u8, |acc, &x| acc ^ x);
        let mut bits = Bits::new(Version::Normal(1));
        bits.push_mode_indicator(ExtendedMode::StructuredAppend)
            .unwrap();
        bits.push_number_checked(4, 0).unwrap(); // first element of the sequence
        bits.push_number_checked(4, 1).unwrap(); // total length of the sequence (means 2)
        bits.push_number_checked(8, parity as usize).unwrap(); //parity of the complete data
        bits.push_byte_data(data_half).unwrap();
        bits.push_terminator(LEVEL).unwrap();
        assert_eq!(
            hex::encode(bits.into_bytes()),
            "3013940e49207265616420746865206e657700"
        ); // raw bytes of the first qr code of the example
    }

    #[test]
    fn test_split_merge_qr() {
        // consider using https://rust-fuzz.github.io/book/introduction.html
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..4000).map(|_| rand::random::<u8>()).collect();
        for _ in 0..1_000 {
            let len = rng.gen_range(100, 4000);
            let ver = rng.gen_range(10, 20);
            let data = (&random_bytes[0..len]).to_vec();
            let split_qr = SplittedQr::new(data.clone(), ver).unwrap();
            let bits = split_qr.split_to_bits().unwrap();
            if bits.len() > 1 {
                let bytes: Vec<Vec<u8>> = bits.into_iter().map(|b| b.into_bytes()).collect();
                let result = merge_qrs(bytes).unwrap();
                assert_eq!(result, data);
            }
        }
    }

    #[test]
    fn test_print_qr() {
        let qr = QrCode::new(b"01234567").unwrap();
        let printed = print_qr(&qr, false).unwrap();
        assert_eq!(hex::encode(printed.as_bytes()),"0a0a0a0a202020e29688e29680e29680e29680e29680e29680e296882020e29688e29684e29688e2968820e29688e29680e29680e29680e29680e29680e296880a202020e2968820e29688e29688e2968820e2968820e29688e2968420202020e2968820e29688e29688e2968820e296880a202020e2968820e29680e29680e2968020e2968820e2968820e29680e29680e2968820e2968820e29680e29680e2968020e296880a202020e29680e29680e29680e29680e29680e29680e2968020e2968820e29680e29684e2968820e29680e29680e29680e29680e29680e29680e296800a202020e2968020e29680e29688e29680e29688e29680e29684e29684e29680e2968420e2968820e29680e29688e29680e29688e2968820200a2020202020e2968020e2968420e29680e2968020e2968820e2968020e2968020e29684e29688e29688e29688e29680e296800a202020202020e29680e29680e29680e29680e29680e2968820e29684e29688e29684e29688e2968420e29680e29684e2968420200a202020e29688e29680e29680e29680e29680e29680e2968820e29684e29680e29688e29684e29688e29684e29688e296802020e2968420e296840a202020e2968820e29688e29688e2968820e2968820e29688e296842020e296882020e2968820e29680e2968020200a202020e2968820e29680e29680e2968020e2968820e2968020e29680e2968020e2968020e29684e2968820e29688e29684200a202020e29680e29680e29680e29680e29680e29680e2968020e29680e29680e29680e2968020e296802020e2968020e2968020200a0a0a0a0a");
    }
}
