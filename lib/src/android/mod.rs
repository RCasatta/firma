use crate::common::list::ListOptions;
use crate::common::qr::{QrMergeOptions, QrOptions};
use crate::offline::descriptor::DeriveAddressOptions;
use crate::offline::dice::DiceOptions;
use crate::offline::print::PrintOptions;
use crate::offline::random::RandomOptions;
use crate::offline::restore::RestoreOptions;
use crate::offline::sign::SignOptions;
use crate::online::WalletNameOptions;
use crate::*;
use android_logger::Config;
use common::error::ToJson;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::{debug, info, Level};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

fn rust_call(c_str: &CStr) -> Result<CString> {
    let str = c_str.to_str()?;
    let value: Value = serde_json::from_str(str)?;

    let method = value
        .get("method")
        .ok_or_else(|| Error::MissingMethod)?
        .as_str()
        .ok_or_else(|| Error::MissingMethod)?
        .to_string();
    let context_value = value
        .get("context")
        .ok_or_else(|| Error::MissingContext)?
        .clone();
    let context: Context = serde_json::from_value(context_value)?;
    let args = value.get("args").unwrap_or(&Value::Null).clone();

    info!("method:{:?} context:{:?} args:{:?}", method, context, args);

    let value = match method.as_str() {
        "random" => {
            let random_opts: RandomOptions = serde_json::from_value(args)?;
            let result = context.create_key(&random_opts)?;
            serde_json::to_value(result)?
        }
        "dice" => {
            let dice_opts: DiceOptions = serde_json::from_value(args)?;
            let result = context.roll(&dice_opts)?;
            serde_json::to_value(result)?
        }
        "list" => {
            let list_opts: ListOptions = serde_json::from_value(args)?;
            let result = context.list(&list_opts)?;
            serde_json::to_value(result)?
        }
        "merge_qrs" => {
            let opts: QrMergeOptions = serde_json::from_value(args)?;
            let result = qr::merge_qrs(opts)?;
            serde_json::to_value(result)?
        }
        "qrs" => {
            let opts: QrOptions = serde_json::from_value(args)?;
            let result = qr::qrs_string_encoding(opts)?;
            serde_json::to_value(result)?
        }
        "sign" => {
            let opts: SignOptions = serde_json::from_value(args)?;
            let result = context.sign(&opts)?;
            serde_json::to_value(result)?
        }
        "restore" => {
            let opts: RestoreOptions = serde_json::from_value(args)?;
            let result = context.restore(&opts)?;
            serde_json::to_value(result)?
        }
        "print" => {
            let opts: PrintOptions = serde_json::from_value(args)?;
            let result = context.print(&opts)?;
            serde_json::to_value(result)?
        }
        "save_psbt" => {
            let opts: SavePSBTOptions = serde_json::from_value(args)?;
            let result = context.save_psbt_options(&opts)?;
            serde_json::to_value(result)?
        }
        "derive_address" => {
            let opts: DeriveAddressOptions = serde_json::from_value(args)?;
            let result = crate::offline::descriptor::derive_address(context.network, &opts)?;
            serde_json::to_value(result)?
        }
        "import" => {
            let result = context.import_json(args)?;
            serde_json::to_value(result)?
        }
        "sign_wallet" => {
            let opts: WalletNameOptions = serde_json::from_value(args)?;
            let result = context.sign_wallet(&opts)?;
            serde_json::to_value(result)?
        }
        "verify_wallet" => {
            let opts: WalletNameOptions = serde_json::from_value(args)?;
            let result = context.verify_wallet(&opts)?;
            serde_json::to_value(result)?
        }
        a @ _ => Error::MethodNotExist(a.to_string()).to_json(),
    };
    let result = serde_json::to_string(&value)?;
    debug!("result: ({})", result);
    Ok(CString::new(result)?)
}

static START: Once = Once::new();

#[no_mangle]
pub extern "C" fn c_call(to: *const c_char) -> *mut c_char {
    if cfg!(debug_assertions) {
        START.call_once(|| {
            android_logger::init_once(Config::default().with_min_level(Level::Debug));
        });
    }

    let input = unsafe { CStr::from_ptr(to) };
    info!("<-- ({:?})", input.to_str());

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
