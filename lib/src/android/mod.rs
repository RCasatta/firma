use crate::common::list::ListOptions;
use crate::offline::descriptor::DeriveAddressOpts;
use crate::offline::dice::DiceOptions;
use crate::offline::print::PrintOptions;
use crate::offline::random::RandomOptions;
use crate::offline::restore::RestoreOptions;
use crate::offline::sign::SignOptions;
use crate::*;
use android_logger::Config;
use bitcoin::Network;
use common::error::ToJson;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::{debug, info, Level};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str::FromStr;
use std::sync::Once;

fn rust_call(c_str: &CStr) -> Result<CString> {
    let str = c_str.to_str()?;
    let value: Value = serde_json::from_str(str)?;
    let datadir = value
        .get("datadir")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::MissingDatadir)?;
    let network = value
        .get("network")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::MissingNetwork)?;
    let network = Network::from_str(network)?;
    let method = value.get("method").and_then(|s| s.as_str());
    let args = value.get("args").unwrap_or(&Value::Null);
    if !str.contains("encryption_key") {
        info!(
            "method:{:?} datadir:{} network:{} args:{:?}",
            method, datadir, network, args
        );
    } else {
        info!(
            "method:{:?} datadir:{} network:{} args:REDACTED",
            method, datadir, network
        );
    }

    let value = match method {
        Some("random") => {
            let random_opts: RandomOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::random::create_key(datadir, network, &random_opts)?;
            serde_json::to_value(result)?
        }
        Some("dice") => {
            let dice_opts: DiceOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::dice::roll(datadir, network, &dice_opts)?;
            serde_json::to_value(result)?
        }
        Some("list") => {
            let list_opts: ListOptions = serde_json::from_value(args.clone())?;
            let result = crate::common::list::list(datadir, network, &list_opts)?;
            serde_json::to_value(result)?
        }
        Some("merge_qrs") => {
            let string_values: Vec<String> = serde_json::from_value(args.clone())?;
            let mut values = vec![];
            for string in string_values {
                values.push(hex::decode(&string)?);
            }
            match qr_code::structured::merge_qrs(values) {
                Ok(merged) => hex::encode(merged).into(),
                Err(e) => e.to_json(),
            }
        }
        Some("sign") => {
            let opts: SignOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::sign::start(&opts, network)?;
            serde_json::to_value(result)?
        }
        Some("restore") => {
            let opts: RestoreOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::restore::start(datadir, network, &opts)?;
            serde_json::to_value(result)?
        }
        Some("print") => {
            let opts: PrintOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::print::start(datadir, network, &opts)?;
            serde_json::to_value(result)?
        }
        Some("save_psbt") => {
            let opts: SavePSBTOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::sign::save_psbt_options(datadir, network, &opts)?;
            serde_json::to_value(result)?
        }
        Some("derive_address") => {
            let opts: DeriveAddressOpts = serde_json::from_value(args.clone())?;
            let result = crate::offline::descriptor::derive_address(network, &opts, 0)?;
            serde_json::to_value(result)?
        }
        Some("import_wallet") => {
            let wallet: WalletJson = serde_json::from_value(args.clone())?;
            let result = crate::online::create_wallet::import_wallet(datadir, network, &wallet)?;
            serde_json::to_value(result)?
        }
        _ => {
            let error: Error = "invalid method".into();
            error.to_json()
        }
    };
    let result = serde_json::to_string(&value)?;
    debug!("result: ({})", result);
    Ok(CString::new(result)?)
}

static START: Once = Once::new();

#[no_mangle]
pub extern "C" fn c_call(to: *const c_char) -> *mut c_char {
    START.call_once(|| {
        android_logger::init_once(Config::default().with_min_level(Level::Debug));
    });

    let input = unsafe { CStr::from_ptr(to) };
    if !input.to_str().unwrap().contains("encryption_key") {
        info!("<-- ({:?})", input.to_str());
    } else {
        info!("<-- (REDACTED)");
    }

    let output = rust_call(input)
        .unwrap_or_else(|e| CString::new(serde_json::to_vec(&e.to_json()).unwrap()).unwrap());
    info!("--> ({:?})", output);
    output.into_raw()
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn Java_it_casatta_Rust_call(
    env: JNIEnv,
    _: JClass,
    java_pattern: JString,
) -> jstring {
    let call_result = c_call(
        env.get_string(java_pattern)
            .expect("invalid pattern string")
            .as_ptr(),
    );
    let call_ptr = CString::from_raw(call_result);
    let output = env
        .new_string(call_ptr.to_str().unwrap())
        .expect("Couldn't create java string!");

    output.into_inner()
}
