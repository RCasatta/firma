use crate::Result;
use bitcoin::consensus::encode::Encodable;
use bitcoin::util::bip158::BitStreamWriter;
use qrcode::types::Color::{Dark, Light};
use qrcode::{Color, QrCode};
