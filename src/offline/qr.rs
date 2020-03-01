use bech32;
use qrcode::types::Color::{Dark, Light};
use qrcode::QrCode;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

type Result<R> = std::result::Result<R, Box<dyn Error>>;

/// qr
#[derive(StructOpt, Debug)]
#[structopt(name = "qr")]
pub struct QrOptions {
    /// json index
    #[structopt(short, long)]
    index: Option<String>,

    /// Invert qr color
    #[structopt(short, long)]
    reverse: bool,

    /// json input file
    file: PathBuf,
}

pub fn show(opt: &QrOptions) -> Result<()> {
    let json = fs::read_to_string(&opt.file)?;
    let initial_json: Value = serde_json::from_str(&json)?;

    let value = match opt.index.clone() {
        Some(val) => initial_json
            .get(val.clone())
            .unwrap_or_else(|| panic!("Can't find key `{}` in the json", val)),
        None => &initial_json,
    };
    let string = if let Value::String(value) = value {
        value.to_string()
    } else {
        format!("{}", value)
    };

    print_qr(&string, opt.reverse);

    Ok(())
}

fn print_qr(value: &str, inverted: bool) {
    let value = if bech32::decode(value).is_ok() {
        value.to_uppercase()
    } else {
        value.to_string()
    };
    let qr_code = QrCode::new(value.as_bytes()).unwrap();
    let width = qr_code.width();
    let qr_code = qr_code.into_colors();
    let height = qr_code.len() / width;
    let mut vec = Vec::new();
    vec.extend(vec![Light; width * 4]);
    vec.extend(qr_code);
    vec.extend(vec![Light; width * 4]);

    let inverted = if inverted { 0 } else { 4 };
    let blocks = ["█", "▀", "▄", " ", " ", "▄", "▀", "█"];

    for i in (0..height + 8).step_by(2) {
        print!(
            "{}{}{}",
            blocks[inverted], blocks[inverted], blocks[inverted]
        );
        for j in 0..width {
            let start = i * width + j;
            let val = match (vec[start], vec.get(start + width).unwrap_or(&Light)) {
                (Light, Light) => 0,
                (Light, Dark) => 1,
                (Dark, Light) => 2,
                (Dark, Dark) => 3,
            };
            print!("{}", blocks[val + inverted]);
        }
        println!(
            "{}{}{}",
            blocks[inverted], blocks[inverted], blocks[inverted]
        );
    }
    println!("{}", value);
}
