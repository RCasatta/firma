use crate::*;
use bitcoin::Network;
use log::info;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::{fs, io};

pub struct PathBuilder {
    datadir: String,
    network: Network,
    kind: Kind,
    name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum KindAndName {
    Wallet(String),
    Key(String),
    PSBT(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Kind {
    #[serde(rename = "wallets")]
    Wallet,
    #[serde(rename = "keys")]
    Key,
    #[serde(rename = "psbts")]
    PSBT,
}

impl ToString for Kind {
    fn to_string(&self) -> String {
        match self {
            Kind::Wallet => "wallets",
            Kind::Key => "keys",
            Kind::PSBT => "psbts",
        }
        .to_string()
    }
}

impl FromStr for Kind {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "wallets" => Ok(Kind::Wallet),
            "keys" => Ok(Kind::Key),
            "psbts" => Ok(Kind::PSBT),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("({}) valid values are: wallets, keys, psbts", s),
            )),
        }
    }
}

impl PathBuilder {
    pub fn new(datadir: &str, network: Network, kind: Kind, name: Option<String>) -> Self {
        PathBuilder {
            datadir: datadir.to_string(),
            network,
            kind,
            name,
        }
    }

    pub fn file(&self, filename: &str) -> Result<PathBuf> {
        let content = self
            .name
            .as_ref()
            .ok_or_else(|| Error::Generic("missing name".into()))?;
        let kind = self.kind.to_string();
        let network_string = format!("{}", self.network);
        let paths: Vec<&str> = vec![&self.datadir, &network_string, &kind, &content];

        let mut path = path_for(paths)?;
        path.push(filename);

        Ok(path)
    }

    pub fn type_path(&self) -> Result<PathBuf> {
        path_for(vec![
            &self.datadir,
            &format!("{}", self.network),
            &self.kind.to_string(),
        ])
    }
}

fn path_for(dirs: Vec<&str>) -> Result<PathBuf> {
    let mut path = PathBuf::from(
        dirs.get(0)
            .ok_or_else(|| Error::Generic("Need at least 1".into()))?,
    );
    path = expand_tilde(path)?;
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    for dir in dirs.iter().skip(1) {
        path.push(&format!("{}/", dir));
        if !path.exists() {
            fs::create_dir(&path)?;
        }
    }
    Ok(path)
}

// from https://stackoverflow.com/questions/54267608/expand-tilde-in-rust-path-idiomatically
pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Result<PathBuf> {
    let p = path_user_input.as_ref();
    if p.starts_with("~") {
        let mut home_dir =
            dirs::home_dir().ok_or_else(|| Error::Generic("cannot retrieve home dir".into()))?;
        if p == Path::new("~") {
            Ok(home_dir)
        } else if home_dir == Path::new("/").to_path_buf() {
            // Corner case: `home_dir` root directory;
            // don't prepend extra `/`, just drop the tilde.
            Ok(p.strip_prefix("~")?.to_path_buf())
        } else {
            home_dir.push(p.strip_prefix("~/")?);
            Ok(home_dir)
        }
    } else {
        Ok(p.to_path_buf())
    }
}

fn save(value: String, output: &PathBuf) -> Result<()> {
    fs::write(output, value)?;
    info!("Saving {:?}", output);
    Ok(())
}

pub fn save_public(public_key: &PublicMasterKey, output: &PathBuf) -> Result<()> {
    if output.exists() {
        return Err(Error::FileExist(output.clone()));
    }
    save(serde_json::to_string_pretty(public_key)?, output)
}

pub fn save_private(private_key: &PrivateMasterKey, output: &PathBuf) -> Result<()> {
    if output.exists() {
        return Err(Error::FileExist(output.clone()));
    }
    save(serde_json::to_string_pretty(private_key)?, output)
}

pub fn save_keys(
    datadir: &str,
    network: Network,
    key_name: &str,
    key: PrivateMasterKey,
    qr_version: i16,
) -> Result<MasterKeyOutput> {
    let option_name = Some(key_name.to_string());
    let path_builder = PathBuilder::new(datadir, network, Kind::Key, option_name.clone());
    let private_key_file = path_builder.file("PRIVATE.json")?;
    let public_key_file = path_builder.file("public.json")?;
    save_private(&key, &private_key_file)?;
    let public_master_key = key.clone().into();
    save_public(&public_master_key, &public_key_file)?;

    let path_for_qr = PathBuilder::new(datadir, network, Kind::Key, option_name).file("qr")?;

    let public_qr_files = qr::save_qrs(
        public_master_key.xpub.to_string().as_bytes().to_vec(),
        path_for_qr,
        qr_version,
    )?;

    Ok(MasterKeyOutput {
        key,
        public_file: Some(public_key_file),
        private_file: private_key_file,
        public_qr_files,
    })
}

pub fn read_psbt_json(path: &Path) -> Result<PsbtJson> {
    let slice = fs::read(path)?;
    Ok(serde_json::from_slice(&slice)?)
}

pub fn read_psbt(path: &Path) -> Result<PSBT> {
    let psbt_json = read_psbt_json(&path)?;
    Ok(psbt_from_base64(&psbt_json.psbt)?.1)
}

pub fn read_wallet(path: &PathBuf) -> Result<WalletJson> {
    let wallet = fs::read(path)?;
    Ok(serde_json::from_slice(&wallet)?)
}

/*

pub fn filename_for_qr(datadir: &str,
                       network: Network,
                       what: &WKP, name: &str) -> Result<PathBuf> {
    let content = what.name_to_string().ok_or_else(|| Error::Generic("missing content".into()) ) ?;
    let path = filename_for()
    let mut path = path_for(vec![datadir, &format!("{}", network), &what.type_to_string(), &content, "qrs"])?;
    path.push(name);
    Ok(path)
}
*/
