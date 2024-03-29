use bitcoin::{Address, Amount, Txid};
use bitcoind::bitcoincore_rpc::{Client, RpcApi};
use bitcoind::downloaded_exe_path;
use firma::bitcoin::Network;
use firma::*;
use rand::distributions::Alphanumeric;
use rand::{self, thread_rng, Rng};
use serde_json::{from_value, to_string_pretty, Value};
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

pub fn rnd_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(20).collect()
}

#[test]
fn integration_test() {
    init_logger();
    let mut rng = rand::thread_rng();
    let firma_exe_dir = env::var("FIRMA_EXE_DIR").unwrap_or("../target/debug/".to_string());
    let bitcoind_exe = env::var("BITCOIND_EXE")
        .ok()
        .or_else(downloaded_exe_path)
        .expect("version feature or env BITCOIND_EXE is required for tests");

    let mut bitcoind = bitcoind::BitcoinD::new(bitcoind_exe).unwrap();

    // fund the bitcoind default wallet
    let address = bitcoind.client.get_new_address(None, None).unwrap();
    bitcoind.client.generate_to_address(101, &address).unwrap();
    let balance = bitcoind.client.get_balance(None, None).unwrap();
    assert!(balance.as_btc() > 49.9);

    // create firma 2of2 wallet
    let name_2of2 = "n2of2".to_string();
    let firma_2of2 = FirmaCommand::new(&firma_exe_dir).unwrap();
    let cookie_file_str = format!("{}", bitcoind.params.cookie_file.display());
    firma_2of2
        .online_connect(&bitcoind.rpc_url(), &cookie_file_str)
        .unwrap();
    let r1 = firma_2of2.offline_random("r1", None).unwrap();
    let r2 = firma_2of2.offline_dice("r2", vec![2u32; 59], 20).unwrap();
    let r_err = firma_2of2.offline_dice("r3", vec![0u32; 59], 20);
    assert_eq!(
        r_err.unwrap_err().to_string(),
        Error::DiceValueErr(0, 20).to_string()
    );
    let key_names = vec![r1.id.name.to_string(), r2.id.name.to_string()];

    let created_2of2_wallet = firma_2of2
        .online_create_wallet(2, &key_names, &name_2of2, true)
        .unwrap();
    assert_eq!(&created_2of2_wallet.id.name, &name_2of2);
    assert_eq!(
        firma_2of2
            .online_create_wallet(2, &key_names, &name_2of2, false)
            .unwrap_err()
            .to_string(),
        "Wallet n2of2 already exists in the bitcoin node"
    );
    let _result = firma_2of2.offline_sign_wallet(&name_2of2).unwrap();

    let r3 = firma_2of2
        .offline_restore("r3", "xprv", &r1.key.to_string())
        .unwrap();
    assert_eq!(r3.key, r1.key);

    // create firma 2of3 wallet
    let name_2of3 = "n2of3".to_string();
    let firma_2of3 = FirmaCommand::new(&firma_exe_dir).unwrap();
    firma_2of3
        .online_connect(&bitcoind.rpc_url(), &cookie_file_str)
        .unwrap();
    let mut vec = vec![];
    for i in 0..3 {
        vec.push(firma_2of3.offline_random(&format!("p{}", i), None).unwrap());
    }

    let key_names_2of3: Vec<String> = vec.iter().map(|e| e.id.name.to_string()).collect();
    let created_2of3_wallet = firma_2of3
        .online_create_wallet(2, &key_names_2of3, &name_2of3, true)
        .unwrap();
    assert_eq!(&created_2of3_wallet.id.name, &name_2of3);

    assert!(firma_2of3
        .online_create_wallet(2, &key_names_2of3, &name_2of3, true)
        .unwrap_err()
        .to_string()
        .contains("Cannot overwrite"));

    // create address for firma 2of2
    let address_2of2 = firma_2of2.online_get_address(&name_2of2).unwrap().address;
    let fund_2of2 = 100_000_000;
    client_send_to_address(&bitcoind.client, &address_2of2, fund_2of2).unwrap();

    // create address for firma 2of3
    let address_2of3 = firma_2of3.online_get_address(&name_2of3).unwrap().address;
    let fund_2of3 = 100_000_000;
    client_send_to_address(&bitcoind.client, &address_2of3, fund_2of3).unwrap();

    // generate 1 block so funds are confirmed
    bitcoind.client.generate_to_address(1, &address).unwrap();

    // check balances 2of2
    let balance_2of2 = firma_2of2.online_balance(&name_2of2).unwrap();
    assert_eq!(fund_2of2, balance_2of2.confirmed.satoshi);

    // check balances 2of3
    let balance_2of3 = firma_2of3.online_balance(&name_2of3).unwrap();
    assert_eq!(fund_2of3, balance_2of3.confirmed.satoshi);

    // create a tx from firma 2of2 wallet and send back to myself (detecting script reuse)
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address_2of2.clone(), value_sent)];
    let psbt_name = rnd_string();
    let create_tx = firma_2of2
        .online_create_tx(recipients, &psbt_name, &name_2of2)
        .unwrap();
    let psbt_file_str = firma_2of2.path_str(Kind::Psbt, &create_tx.psbt_name);

    //let sign_a_wrong = firma_2of2.offline_sign(psbt_file_str, psbt_file_str);
    //assert_eq!(sign_a_wrong.unwrap_err().to_string(),Error::WrongKeyFileName.to_string());

    let print_a = firma_2of2.offline_print(&psbt_file_str).unwrap();
    let print_a_by_name = firma_2of2.offline_print_by_name(&psbt_name).unwrap();
    assert_eq!(print_a, print_a_by_name);
    let sign_a = firma_2of2
        .offline_sign(&psbt_name, &r1.id.name, &name_2of2)
        .unwrap(); //TODO test passing public key
    assert_ne!(print_a, sign_a);
    assert_ne!(print_a.inputs, sign_a.inputs);
    assert_eq!(print_a.outputs, sign_a.outputs);
    assert_ne!(print_a.size, sign_a.size);
    assert_eq!(print_a.fee, sign_a.fee);
    assert_ne!(print_a.info, sign_a.info);
    assert!(sign_a.info.iter().any(|msg| msg.contains("#Address_reuse")));
    let sign_b = firma_2of2
        .offline_sign(&psbt_name, &r2.id.name, &name_2of2)
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);

    let sent_tx = firma_2of2
        .online_send_tx(vec![&psbt_name], &name_2of2)
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of2 = firma_2of2.online_balance(&name_2of2).unwrap();
    let expected = fund_2of2 - sign_a.fee.absolute.unwrap(); // since sending to myself deduct just the fee
    assert_eq!(expected, balance_2of2.confirmed.satoshi);

    // create a tx from firma 2of2 with rounded amount but same script types, check privacy analysis
    let value_sent = 1_000_000;
    let recipients = vec![(address_2of2.clone(), value_sent)];
    let psbt_name = rnd_string();
    let create_tx = firma_2of2
        .online_create_tx(recipients, &psbt_name, &name_2of2)
        .unwrap();
    assert!(create_tx.address_reused.contains(&address_2of2));

    let sign_a = firma_2of2
        .offline_sign(&psbt_name, &r1.id.name, &name_2of2)
        .unwrap();
    assert!(sign_a.info.iter().any(|msg| msg.contains("#Round_numbers")));
    assert!(!sign_a
        .info
        .iter()
        .any(|msg| msg.contains("#Sending_to_a_different_script_type")));

    //TODO create a tx from firma 2of2 sending all

    // create a tx from firma 2of3 wallet and send back to bitcoind with keys 0 and 1
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address.clone(), value_sent)];
    let psbt_name = rnd_string();
    let _create_tx = firma_2of3
        .online_create_tx(recipients, &psbt_name, &name_2of3)
        .unwrap();

    let sign_a = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[0], &name_2of3)
        .unwrap(); //TODO passing xpub file gives misleading error
    assert!(sign_a
        .info
        .iter()
        .any(|msg| msg.contains("#Sending_to_a_different_script_type"))); // core generates a different address type
    let sign_b = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[1], &name_2of3)
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![&psbt_name], &name_2of3)
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3 = firma_2of3.online_balance(&name_2of3).unwrap();
    let expected = fund_2of3 - value_sent - sign_a.fee.absolute.unwrap();
    assert_eq!(expected, balance_2of3.confirmed.satoshi);

    // create a tx from firma 2of3 wallet and send back to bitcoind with keys 1 and 2
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address.clone(), value_sent)];
    let psbt_name = rnd_string();
    let _create_tx = firma_2of3
        .online_create_tx(recipients, &psbt_name, &name_2of3)
        .unwrap();
    let sign_a = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[1], &name_2of3)
        .unwrap();
    let sign_b = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[2], &name_2of3)
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![&psbt_name], &name_2of3)
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3_2 = firma_2of3.online_balance(&name_2of3).unwrap();
    let expected = balance_2of3.confirmed.satoshi - value_sent - sign_a.fee.absolute.unwrap();
    assert_eq!(expected, balance_2of3_2.confirmed.satoshi);

    // create a tx from firma 2of3 wallet and send back to bitcoind with keys 0 and 2
    // sending in serial psbt->signer_a->signer_b->broadcast
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address.clone(), value_sent)];
    let psbt_name = rnd_string();
    let _create_tx = firma_2of3
        .online_create_tx(recipients, &psbt_name, &name_2of3)
        .unwrap();

    let sign_a = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[0], &name_2of3)
        .unwrap();
    let sign_b = firma_2of3
        .offline_sign(&psbt_name, &key_names_2of3[2], &name_2of3)
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![&psbt_name], &name_2of3)
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3_3 = firma_2of3.online_balance(&name_2of3).unwrap();
    let expected = balance_2of3_2.confirmed.satoshi - value_sent - sign_a.fee.absolute.unwrap();
    assert_eq!(expected, balance_2of3_3.confirmed.satoshi);

    let coins_output = firma_2of3.online_list_coins(&name_2of3).unwrap();
    assert!(!coins_output.coins.is_empty());

    let list_keys = firma_2of2.offline_list(Kind::MasterSecret, None).unwrap();
    let keys = &list_keys.master_secrets;
    assert!(keys.iter().any(|k| k.id.name == r1.id.name));
    assert!(keys.iter().any(|k| k.id.name == r2.id.name));
    let list_wallets = firma_2of2.offline_list(Kind::Wallet, None).unwrap();
    assert!(list_wallets.wallets.iter().any(|w| w.id.name == name_2of2));
    let list_psbt = firma_2of2.offline_list(Kind::Psbt, None).unwrap();
    assert_eq!(list_psbt.psbts.len(), 2);
    let result = firma_2of3.online_rescan(&name_2of3); // TODO test restore a wallet, find funds with rescan
    assert!(result.is_ok());

    // test key encryption
    let encryption_key = Some(&[0u8; 32][..]);
    let e1 = firma_2of2
        .offline_random("key_encrypted", encryption_key)
        .unwrap();
    let list_keys = firma_2of2.offline_list(Kind::MasterSecret, None).unwrap();
    let keys = &list_keys.master_secrets;
    assert!(
        !keys.iter().any(|k| k.id.name == e1.id.name),
        "can see private key without encryption_key"
    );
    let keys = firma_2of2
        .offline_list(Kind::MasterSecret, encryption_key)
        .unwrap()
        .master_secrets;
    assert!(
        keys.iter().any(|k| k.id.name == e1.id.name),
        "can't see private key with encryption_key"
    );
    assert!(firma_2of2
        .offline_export("MasterSecret", &e1.id.name, None)
        .is_err());
    let d1 = firma_2of2
        .offline_export("MasterSecret", &e1.id.name, encryption_key)
        .unwrap();
    assert_eq!(d1.get("key").unwrap().as_str().unwrap(), e1.key.to_string());

    // stop bitcoind
    let ecode = bitcoind.stop().unwrap();
    assert!(ecode.success());
}

struct FirmaCommand {
    pub exe_dir: String,
    pub work_dir: TempDir,
}

impl FirmaCommand {
    pub fn new(exe_dir: &str) -> Result<Self> {
        let work_dir = TempDir::new().unwrap();
        Ok(FirmaCommand {
            exe_dir: exe_dir.to_string(),
            work_dir,
        })
    }

    pub fn path_str(&self, kind: Kind, name: &str) -> String {
        let buf = Identifier::new(Network::Regtest, kind, name)
            .as_path_buf(&self.work_dir, false)
            .unwrap();
        buf.to_str().unwrap().to_string()
    }

    pub fn online(&self, subcmd: &str, args: Vec<&str>) -> Result<Value> {
        let output = Command::new(format!("{}/firma-online", self.exe_dir))
            .arg("--datadir")
            .arg(format!("{}", self.work_dir.path().display()))
            .arg("--network")
            .arg("regtest")
            .arg(subcmd)
            .args(&args)
            .output()
            .unwrap();
        if !output.status.success() {
            println!("{}", std::str::from_utf8(&output.stderr)?);
        }
        assert!(
            output.status.success(),
            "online subcmd:{} args:{:?}",
            subcmd,
            args
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        println!("{}", to_string_pretty(&value).unwrap());
        Ok(value)
    }

    pub fn online_connect(&self, node_url: &str, cookie_file: &str) -> Result<DaemonOpts> {
        let args = vec!["--url", node_url, "--cookie-file", cookie_file];
        let result = self.online("connect", args);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn online_create_wallet(
        &self,
        required_sig: u8,
        names: &[String],
        wallet_name: &str,
        allow_wallet_already_exists: bool,
    ) -> Result<Wallet> {
        let required_sig = format!("{}", required_sig);
        let mut args = vec!["-r", &required_sig, "--wallet-name", wallet_name];
        if allow_wallet_already_exists {
            args.push("--allow-wallet-already-exists");
        }
        for name in names {
            args.push("--key-name");
            args.push(name);
        }
        let result = self.online("create-wallet", args);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    fn online_get_address(&self, wallet_name: &str) -> Result<GetAddressOutput> {
        Ok(from_value(
            self.online("get-address", vec!["--wallet-name", wallet_name])
                .unwrap(),
        )?)
    }

    fn online_balance(&self, wallet_name: &str) -> Result<BalanceOutput> {
        Ok(from_value(
            self.online("balance", vec!["--wallet-name", wallet_name])
                .unwrap(),
        )?)
    }

    fn online_list_coins(&self, wallet_name: &str) -> Result<ListCoinsOutput> {
        Ok(from_value(
            self.online("list-coins", vec!["--wallet-name", wallet_name])
                .unwrap(),
        )?)
    }

    pub fn online_rescan(&self, wallet_name: &str) -> Result<usize> {
        Ok(from_value(
            self.online(
                "rescan",
                vec!["--start-from", "0", "--wallet-name", wallet_name],
            )
            .unwrap(),
        )?)
    }

    fn online_create_tx(
        &self,
        recipients: Vec<(Address, u64)>,
        psbt_name: &str,
        wallet_name: &str,
    ) -> Result<CreateTxOutput> {
        let mut args = vec!["--wallet-name".to_string(), wallet_name.to_string()];
        for recipient in recipients {
            args.push("--recipient".to_string());
            args.push(format!("{}:{}", recipient.0, recipient.1));
        }
        args.push("--psbt-name".to_string());
        args.push(psbt_name.to_string());
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        let output = from_value(self.online("create-tx", args).unwrap())?;
        Ok(output)
    }

    fn online_send_tx(&self, names: Vec<&str>, wallet_name: &str) -> Result<SendTxOutput> {
        let mut args = vec!["--broadcast", "--wallet-name", wallet_name];
        for name in names {
            args.push("--psbt-name");
            args.push(name);
        }
        let value = self.online("send-tx", args).unwrap();
        Ok(from_value(value)?)
    }

    pub fn offline(
        &self,
        subcmd: &str,
        args: Vec<&str>,
        encryption_key: Option<&[u8]>,
    ) -> Result<Value> {
        let (stdin, read_stdin_arg) = match encryption_key {
            Some(_) => (Stdio::piped(), vec!["--encrypt"]),
            None => (Stdio::null(), vec![]),
        };
        let mut process = Command::new(format!("{}/firma-offline", self.exe_dir))
            .stdin(stdin)
            .stdout(Stdio::piped())
            .arg("--datadir")
            .arg(format!("{}", self.work_dir.path().display()))
            .arg("--network")
            .arg("regtest")
            .args(&read_stdin_arg)
            .arg(subcmd)
            .args(&args)
            .spawn()
            .unwrap();
        if let Some(encryption_key) = encryption_key {
            let child_stdin = process.stdin.as_mut().unwrap();
            child_stdin.write_all(encryption_key).unwrap();
        }
        let output = process.wait_with_output().unwrap();
        if !output.status.success() {
            println!("{}", std::str::from_utf8(&output.stderr)?);
        }

        assert!(
            output.status.success(),
            "offline subcmd:{} args:{:?} encryption_key:{:?}",
            subcmd,
            args,
            encryption_key
        );

        let value: Value = serde_json::from_slice(&output.stdout)?;
        println!("{:?}", to_string_pretty(&value));

        Ok(value)
    }

    pub fn offline_random(
        &self,
        key_name: &str,
        encryption_key: Option<&[u8]>,
    ) -> Result<MasterSecret> {
        let result = self.offline("random", vec!["--key-name", key_name], encryption_key);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_export(
        &self,
        kind: &str,
        name: &str,
        encryption_key: Option<&[u8]>,
    ) -> Result<Value> {
        let result = self.offline(
            "export",
            vec!["--kind", kind, "--name", name],
            encryption_key,
        );
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_dice(
        &self,
        key_name: &str,
        launches: Vec<u32>,
        faces: u32,
    ) -> Result<MasterSecret> {
        let faces = format!("{}", faces);
        let mut args = vec!["--key-name", key_name, "--faces", &faces];
        let launches: Vec<String> = launches.iter().map(|e| format!("{}", e)).collect();
        for launch in launches.iter() {
            args.push("-l");
            args.push(launch);
        }
        let result = self.offline("dice", args, None);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_restore(
        &self,
        key_name: &str,
        nature: &str,
        value: &str,
    ) -> Result<MasterSecret> {
        let result = self.offline(
            "restore",
            vec!["--key-name", key_name, "--nature", nature, value],
            None,
        );
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_sign(
        &self,
        psbt_name: &str,
        key_name: &str,
        wallet_name: &str,
    ) -> Result<PsbtPrettyPrint> {
        let result = self.offline(
            "sign",
            vec![
                "--psbt-name",
                psbt_name,
                "--key-name",
                key_name,
                "--total-derivations",
                "20",
                "--wallet-name",
                wallet_name,
            ],
            None,
        );
        let value = map_json_error(result)?;
        let output = from_value(value)?;
        Ok(output)
    }

    pub fn offline_sign_wallet(&self, wallet_name: &str) -> Result<WalletSignature> {
        let result = self.offline("sign-wallet", vec!["--wallet-name", wallet_name], None);
        let value = map_json_error(result)?;
        let output = from_value(value)?;
        Ok(output)
    }

    pub fn offline_list(&self, kind: Kind, encryption_key: Option<&[u8]>) -> Result<ListOutput> {
        Ok(from_value(
            self.offline("list", vec!["--kind", &kind.to_string()], encryption_key)
                .unwrap(),
        )?)
    }

    pub fn offline_print(&self, psbt_file: &str) -> Result<PsbtPrettyPrint> {
        let result = self.offline("print", vec!["--psbt-file", psbt_file], None);
        let value = map_json_error(result)?;
        let output = from_value(value)?;
        Ok(output)
    }

    pub fn offline_print_by_name(&self, psbt_name: &str) -> Result<PsbtPrettyPrint> {
        let result = self.offline("print", vec!["--psbt-name", psbt_name], None);
        let value = map_json_error(result)?;
        let output = from_value(value)?;
        Ok(output)
    }
}

fn client_send_to_address(client: &Client, address: &Address, satoshi: u64) -> Result<Txid> {
    Ok(client
        .send_to_address(
            address,
            Amount::from_sat(satoshi),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap())
}
