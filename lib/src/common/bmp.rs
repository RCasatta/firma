use crate::Result;
use bitcoin::consensus::encode::Encodable;
use bitcoin::util::bip158::BitStreamWriter;
use qrcode::types::Color::{Dark, Light};
use qrcode::{Color, QrCode};

const B: u8 = 66;
const M: u8 = 77;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QrData {
    data: Vec<Color>,
    width: usize,
}

impl QrData {
    fn height(&self) -> usize {
        self.data.len() / self.width
    }

    fn get(&self, i: usize, j: usize) -> Option<&Color> {
        self.data.get(i * self.width + j)
    }

    /// multiply by `mul` every pixel
    fn mul(&self, mul: usize) -> QrData {
        let mut data = vec![];

        for i in 0..self.height() {
            let mut row = vec![];
            for j in 0..self.width {
                for _ in 0..mul {
                    row.push(self.get(i, j).unwrap());
                }
            }
            for _ in 0..mul {
                data.extend(row.clone());
            }
        }

        let width = self.width * mul;
        QrData { data, width }
    }

    /// add `white_space` pixels around
    fn add_whitespace(&self, white_space: usize) -> QrData {
        let width = self.width + white_space * 2;
        let mut data = vec![];
        for _ in 0..white_space {
            data.extend(vec![Light; width]);
        }
        for vec in self.data.chunks(self.width) {
            for _ in 0..white_space {
                data.push(Light);
            }
            data.extend(vec);
            for _ in 0..white_space {
                data.push(Light);
            }
        }
        for _ in 0..white_space {
            data.extend(vec![Light; width]);
        }

        QrData { data, width }
    }

    /// Returns a monocromatic bitmap
    pub fn bmp(&self) -> Result<Vec<u8>> {
        let qr = self.mul(3).add_whitespace(12);
        let width = qr.width;
        let height = qr.height();

        let color_pallet_size = 2 * 4; // 2 colors each 4 bytes
        let header_size = 2 + 12 + 40 + color_pallet_size;
        let bytes_per_row = bytes_per_row(width as u32);
        let padding = padding(bytes_per_row);
        let data_size = (bytes_per_row + padding) * (height as u32);
        let total_size = header_size + data_size;
        let mut bmp_data = vec![];

        B.consensus_encode(&mut bmp_data).unwrap();
        M.consensus_encode(&mut bmp_data).unwrap();
        total_size.consensus_encode(&mut bmp_data).unwrap(); // size of the bmp
        0u16.consensus_encode(&mut bmp_data).unwrap(); // creator1
        0u16.consensus_encode(&mut bmp_data).unwrap(); // creator2
        header_size.consensus_encode(&mut bmp_data).unwrap(); // pixel offset
        40u32.consensus_encode(&mut bmp_data).unwrap(); // dib header size
        (width as u32).consensus_encode(&mut bmp_data).unwrap(); // width
        (height as u32).consensus_encode(&mut bmp_data).unwrap(); // height
        1u16.consensus_encode(&mut bmp_data).unwrap(); // planes
        1u16.consensus_encode(&mut bmp_data).unwrap(); // bitsperpixel
        0u32.consensus_encode(&mut bmp_data).unwrap(); // no compression
        data_size.consensus_encode(&mut bmp_data).unwrap(); // size of the raw bitmap data with padding
        2835u32.consensus_encode(&mut bmp_data).unwrap(); // hres
        2835u32.consensus_encode(&mut bmp_data).unwrap(); // vres
        2u32.consensus_encode(&mut bmp_data).unwrap(); // num_colors
        2u32.consensus_encode(&mut bmp_data).unwrap(); // num_imp_colors

        // color_pallet
        0x00_FF_FF_FFu32.consensus_encode(&mut bmp_data).unwrap();
        0x00_00_00_00u32.consensus_encode(&mut bmp_data).unwrap();

        let mut data = Vec::new();
        let mut writer = BitStreamWriter::new(&mut data);

        for i in 0..height {
            for j in 0..width {
                let color = qr.get(i, j).unwrap();
                match color {
                    &Light => writer.write(0, 1)?,
                    &Dark => writer.write(1, 1)?,
                };
            }
            writer.write(0, 8 - (width % 8) as u8)?;
            writer.write(0, padding as u8 * 8)?;
        }
        writer.flush().unwrap();
        bmp_data.extend(data);

        Ok(bmp_data)
    }
}

impl From<QrCode> for QrData {
    fn from(qr: QrCode) -> Self {
        let width = qr.width();
        let data = qr.into_colors();
        QrData { width, data }
    }
}

/// return bytes needed for `width` bits
fn bytes_per_row(width: u32) -> u32 {
    (width + 7) / 8
}

/// return the padding needed for n
fn padding(n: u32) -> u32 {
    (4 - n % 4) % 4
}

#[cfg(test)]
mod test {
    use crate::bmp::QrData;
    use crate::common::bmp::{bytes_per_row, padding};
    use qrcode::types::Color::{Dark, Light};
    use qrcode::QrCode;

    #[test]
    fn test_padding() {
        assert_eq!(padding(0), 0);
        assert_eq!(padding(1), 3);
        assert_eq!(padding(2), 2);
        assert_eq!(padding(3), 1);
        assert_eq!(padding(4), 0);
    }

    #[test]
    fn test_bytes_per_row() {
        assert_eq!(bytes_per_row(0), 0);
        assert_eq!(bytes_per_row(1), 1);
        assert_eq!(bytes_per_row(3), 1);
        assert_eq!(bytes_per_row(8), 1);
        assert_eq!(bytes_per_row(9), 2);
        assert_eq!(bytes_per_row(64), 8);
        assert_eq!(bytes_per_row(65), 9);
    }

    #[test]
    fn test_mul() {
        let qr_data = QrData {
            data: vec![Light, Dark, Light, Dark],
            width: 2,
        };

        let qr_data_bigger = QrData {
            data: vec![
                Light, Light, Dark, Dark, Light, Light, Dark, Dark, Light, Light, Dark, Dark,
                Light, Light, Dark, Dark,
            ],
            width: 4,
        };

        assert_eq!(qr_data.mul(2), qr_data_bigger);
    }

    #[test]
    fn test_add() {
        let qr_data = QrData {
            data: vec![Light],
            width: 1,
        };

        let qr_data_bigger = QrData {
            data: vec![Light; 25],
            width: 5,
        };

        assert_eq!(qr_data.add_whitespace(2), qr_data_bigger);
    }

    #[test]
    fn test_bmp() {
        let qr = QrCode::new(b"01234567").unwrap();
        let qr: QrData = qr.into();
        let bmp_data = qr.bmp().unwrap();
        assert_eq!(hex::encode(&bmp_data),"424d52040000000000003e000000280000005700000057000000010001000000000014040000130b0000130b00000200000002000000ffffff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000fffff81c7e3ffffe00000000fffff81c7e3ffffe00000000fffff81c7e3ffffe00000000e000381ffe38000e00000000e000381ffe38000e00000000e000381ffe38000e00000000e3fe38e00038ff8e00000000e3fe38e00038ff8e00000000e3fe38e00038ff8e00000000e3fe38fc0038ff8e00000000e3fe38fc0038ff8e00000000e3fe38fc0038ff8e00000000e3fe38e3fe38ff8e00000000e3fe38e3fe38ff8e00000000e3fe38e3fe38ff8e00000000e00038e00e38000e00000000e00038e00e38000e00000000e00038e00e38000e00000000fffff8e38e3ffffe00000000fffff8e38e3ffffe00000000fffff8e38e3ffffe00000000000000e07e00000000000000000000e07e00000000000000000000e07e00000000000000e3fff81c0e3fff8000000000e3fff81c0e3fff8000000000e3fff81c0e3fff80000000000071c7e38e071f80000000000071c7e38e071f80000000000071c7e38e071f800000000003803f1c71c0fffe0000000003803f1c71c0fffe0000000003803f1c71c0fffe00000000000e001c0007ff8000000000000e001c0007ff8000000000000e001c0007ff8000000000007fffe071c0e00000000000007fffe071c0e00000000000007fffe071c0e00000000000000000e3fff81f8000000000000000e3fff81f8000000000000000e3fff81f8000000000fffff81f8e3f000000000000fffff81f8e3f000000000000fffff81f8e3f000000000000e00038e3fff8038e00000000e00038e3fff8038e00000000e00038e3fff8038e00000000e3fe38e00e071f8000000000e3fe38e00e071f8000000000e3fe38e00e071f8000000000e3fe38fc0e07000000000000e3fe38fc0e07000000000000e3fe38fc0e07000000000000e3fe38e3f1c0e38000000000e3fe38e3f1c0e38000000000e3fe38e3f1c0e38000000000e00038000007e3f000000000e00038000007e3f000000000e00038000007e3f000000000fffff8fff1c0e38000000000fffff8fff1c0e38000000000fffff8fff1c0e38000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    }
}
