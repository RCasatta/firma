use bitcoin::{Address, Amount, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use firma::*;
use rand::distributions::Alphanumeric;
use rand::{self, thread_rng, Rng};
use serde_json::{from_value, to_string_pretty, Value};
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

mod bitcoind;

fn rnd_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(20).collect()
}

#[test]
fn integration_test() {
    let mut rng = rand::thread_rng();
    let firma_exe_dir = env::var("FIRMA_EXE_DIR").unwrap_or("../target/debug/".to_string());

    let mut bitcoind = bitcoind::BitcoinD::new();

    // fund the bitcoind default wallet
    let address = bitcoind.client.get_new_address(None, None).unwrap();
    bitcoind.client.generate_to_address(101, &address).unwrap();
    let balance = bitcoind.client.get_balance(None, None).unwrap();
    assert!(balance.as_btc() > 49.9);

    // create firma 2of2 wallet
    let name_2of2 = "n2of2".to_string();
    let firma_2of2 = FirmaCommand::new(&firma_exe_dir, &name_2of2).unwrap();
    let r1 = firma_2of2.offline_random("r1", None).unwrap();
    let r2 = firma_2of2.offline_dice("r2", vec![2u32; 59], 20).unwrap();
    let r_err = firma_2of2.offline_dice("r3", vec![0u32; 59], 20);
    assert_eq!(
        r_err.unwrap_err().to_string(),
        Error::DiceValueErr(0, 20).to_string()
    );
    let r3 = firma_2of2
        .offline_restore("r4", "xprv", &r1.key.xprv.to_string())
        .unwrap();
    assert_eq!(r3.key.xpub, r1.key.xpub);
    let xpubs = vec![
        r1.public_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        r2.public_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    ];
    let cookie_file_str = format!("{}", bitcoind.cookie_file.display());
    let created_2of2_wallet = firma_2of2
        .online_create_wallet(&bitcoind.url, &cookie_file_str, 2, &xpubs)
        .unwrap();
    assert_eq!(&created_2of2_wallet.wallet.id.name, &name_2of2);

    // create firma 2of3 wallet
    let name_2of3 = "n2of3".to_string();
    let firma_2of3 = FirmaCommand::new(&firma_exe_dir, &name_2of3).unwrap();
    let mut vec = vec![];
    for i in 0..3 {
        vec.push(firma_2of3.offline_random(&format!("p{}", i), None).unwrap());
    }
    let xpubs_2of3: Vec<String> = vec.iter().map(|e| e.public_file_str().unwrap()).collect();
    let xprvs_2of3: Vec<String> = vec.iter().map(|e| e.private_file_str().unwrap()).collect();
    let created_2of3_wallet = firma_2of3
        .online_create_wallet(&bitcoind.url, &cookie_file_str, 2, &xpubs_2of3)
        .unwrap();
    assert_eq!(&created_2of3_wallet.wallet.id.name, &name_2of3);

    let created_2of3_wallet_err = firma_2of3
        .online_create_wallet(&bitcoind.url, &cookie_file_str, 2, &xpubs_2of3)
        .unwrap_err();
    assert!(created_2of3_wallet_err
        .to_string()
        .contains("already exist")); // error from bitcoin rpc

    // create address for firma 2of2
    let address_2of2 = firma_2of2.online_get_address().unwrap().address;
    let fund_2of2 = 100_000_000;
    client_send_to_address(&bitcoind.client, &address_2of2, fund_2of2).unwrap();

    // create address for firma 2of3
    let address_2of3 = firma_2of3.online_get_address().unwrap().address;
    let fund_2of3 = 100_000_000;
    client_send_to_address(&bitcoind.client, &address_2of3, fund_2of3).unwrap();

    // generate 1 block so funds are confirmed
    bitcoind.client.generate_to_address(1, &address).unwrap();

    // check balances 2of2
    let balance_2of2 = firma_2of2.online_balance().unwrap();
    assert_eq!(fund_2of2, balance_2of2.confirmed.satoshi);

    // check balances 2of3
    let balance_2of3 = firma_2of3.online_balance().unwrap();
    assert_eq!(fund_2of3, balance_2of3.confirmed.satoshi);

    // create a tx from firma 2of2 wallet and send back to myself (detecting script reuse)
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address_2of2.clone(), value_sent)];
    let create_tx = firma_2of2
        .online_create_tx(recipients, &rnd_string())
        .unwrap();
    let psbt_file_str = create_tx.psbt_file.to_str().unwrap();
    let sign_a_wrong = firma_2of2.offline_sign(psbt_file_str, psbt_file_str);
    assert_eq!(
        sign_a_wrong.unwrap_err().to_string(),
        Error::WrongKeyFileName.to_string()
    );

    let print_a = firma_2of2.offline_print(psbt_file_str).unwrap();
    let sign_a = firma_2of2
        .offline_sign(psbt_file_str, &r1.private_file.to_str().unwrap())
        .unwrap(); //TODO test passing public key
    assert_ne!(print_a, sign_a);
    assert_ne!(print_a.inputs, sign_a.inputs);
    assert_eq!(print_a.outputs, sign_a.outputs);
    assert_ne!(print_a.size, sign_a.size);
    assert_eq!(print_a.fee, sign_a.fee);
    assert_ne!(print_a.info, sign_a.info);
    assert!(sign_a.info.iter().any(|msg| msg.contains("#Address_reuse")));
    let sign_b = firma_2of2
        .offline_sign(psbt_file_str, &r2.private_file.to_str().unwrap())
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);

    let sent_tx = firma_2of2
        .online_send_tx(vec![
            &sign_a.psbt_file.to_str().unwrap(),
            &sign_b.psbt_file.to_str().unwrap(),
        ])
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of2 = firma_2of2.online_balance().unwrap();
    let expected = fund_2of2 - sign_a.fee.absolute; // since sending to myself deduct just the fee
    assert_eq!(expected, balance_2of2.confirmed.satoshi);

    // create a tx from firma 2of2 with rounded amount but same script types, check privacy analysis
    let value_sent = 1_000_000;
    let recipients = vec![(address_2of2.clone(), value_sent)];
    let create_tx = firma_2of2
        .online_create_tx(recipients, &rnd_string())
        .unwrap();
    let pstb_file_str = create_tx.psbt_file.to_str().unwrap();
    assert!(create_tx.address_reused.contains(&address_2of2));

    let sign_a = firma_2of2
        .offline_sign(pstb_file_str, &r1.private_file.to_str().unwrap())
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
    let create_tx = firma_2of3
        .online_create_tx(recipients, &rnd_string())
        .unwrap();
    let pstb_file_str = create_tx.psbt_file.to_str().unwrap();
    let sign_a = firma_2of3
        .offline_sign(pstb_file_str, &xprvs_2of3[0])
        .unwrap(); //TODO passing xpub file gives misleading error
    assert!(sign_a
        .info
        .iter()
        .any(|msg| msg.contains("#Sending_to_a_different_script_type"))); // core generates a different address type
    let sign_b = firma_2of3
        .offline_sign(pstb_file_str, &xprvs_2of3[1])
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![
            &sign_a.psbt_file.to_str().unwrap(),
            &sign_b.psbt_file.to_str().unwrap(),
        ])
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3 = firma_2of3.online_balance().unwrap();
    let expected = fund_2of3 - value_sent - sign_a.fee.absolute;
    assert_eq!(expected, balance_2of3.confirmed.satoshi);

    // create a tx from firma 2of3 wallet and send back to bitcoind with keys 1 and 2
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address.clone(), value_sent)];
    let create_tx = firma_2of3
        .online_create_tx(recipients, &rnd_string())
        .unwrap();
    let pstb_file_str = create_tx.psbt_file.to_str().unwrap();
    let sign_a = firma_2of3
        .offline_sign(pstb_file_str, &xprvs_2of3[1])
        .unwrap();
    let sign_b = firma_2of3
        .offline_sign(pstb_file_str, &xprvs_2of3[2])
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![
            &sign_a.psbt_file.to_str().unwrap(),
            &sign_b.psbt_file.to_str().unwrap(),
        ])
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3_2 = firma_2of3.online_balance().unwrap();
    let expected = balance_2of3.confirmed.satoshi - value_sent - sign_a.fee.absolute;
    assert_eq!(expected, balance_2of3_2.confirmed.satoshi);

    // create a tx from firma 2of3 wallet and send back to bitcoind with keys 0 and 2
    // sending in serial psbt->signer_a->signer_b->broadcast
    let value_sent = rng.gen_range(1_000, 1_000_000);
    let recipients = vec![(address.clone(), value_sent)];
    let create_tx = firma_2of3
        .online_create_tx(recipients, &rnd_string())
        .unwrap();
    let pstb_file_str = create_tx.psbt_file.to_str().unwrap();

    let sign_a = firma_2of3
        .offline_sign(pstb_file_str, &xprvs_2of3[0])
        .unwrap();
    let sign_b = firma_2of3
        .offline_sign(sign_a.psbt_file.to_str().unwrap(), &xprvs_2of3[2])
        .unwrap();
    assert_eq!(sign_a.fee.absolute, sign_b.fee.absolute);
    let sent_tx = firma_2of3
        .online_send_tx(vec![&sign_b.psbt_file.to_str().unwrap()])
        .unwrap();
    assert!(sent_tx.broadcasted);
    bitcoind.client.generate_to_address(1, &address).unwrap();
    let balance_2of3_3 = firma_2of3.online_balance().unwrap();
    let expected = balance_2of3_2.confirmed.satoshi - value_sent - sign_a.fee.absolute;
    assert_eq!(expected, balance_2of3_3.confirmed.satoshi);

    let coins_output = firma_2of3.online_list_coins().unwrap();
    assert!(!coins_output.coins.is_empty());

    let list_keys = firma_2of2.offline_list(Kind::Key, None).unwrap();
    assert!(list_keys
        .keys
        .iter()
        .any(|k| k.key.id.name == r1.key.id.name));
    assert!(list_keys
        .keys
        .iter()
        .any(|k| k.key.id.name == r2.key.id.name));
    let list_wallets = firma_2of2.offline_list(Kind::Wallet, None).unwrap();
    assert!(list_wallets
        .wallets
        .iter()
        .any(|w| w.wallet.id.name == name_2of2));
    let list_psbt = firma_2of2.offline_list(Kind::PSBT, None).unwrap();
    assert_eq!(list_psbt.psbts.len(), 2);
    let result = firma_2of3.online_rescan(); // TODO test restore a wallet, find funds with rescan
    assert!(result.is_ok());

    // test key encryption
    let encryption_key = Some(&[0u8; 32][..]);
    let e1 = firma_2of2
        .offline_random("key_encrypted", encryption_key)
        .unwrap();
    let list_keys = firma_2of2.offline_list(Kind::Key, None).unwrap();
    assert!(
        !list_keys
            .keys
            .iter()
            .any(|k| k.key.id.name == e1.key.id.name),
        "can see private key without encryption_key"
    );
    let list_keys = firma_2of2.offline_list(Kind::Key, encryption_key).unwrap();
    assert!(
        list_keys
            .keys
            .iter()
            .any(|k| k.key.id.name == e1.key.id.name),
        "can't see private key with encryption_key"
    );
    assert!(firma_2of2
        .offline_decrypt(e1.private_file.to_str().unwrap(), None)
        .is_err());
    let d1 = firma_2of2
        .offline_decrypt(e1.private_file.to_str().unwrap(), encryption_key)
        .unwrap();
    assert_eq!(d1.xprv, e1.key.xprv);

    // stop bitcoind
    bitcoind.client.stop().unwrap();
    let ecode = bitcoind.wait().unwrap();
    assert!(ecode.success());
}

struct FirmaCommand {
    pub exe_dir: String,
    pub work_dir: TempDir,
    pub wallet_name: String,
}

impl FirmaCommand {
    pub fn new(exe_dir: &str, wallet_name: &str) -> Result<Self> {
        let work_dir = TempDir::new().unwrap();
        Ok(FirmaCommand {
            exe_dir: exe_dir.to_string(),
            wallet_name: wallet_name.to_string(),
            work_dir,
        })
    }

    fn wallet_file(&self) -> String {
        format!(
            "{}/regtest/wallets/{}/descriptor.json",
            self.work_dir.path().display(),
            self.wallet_name
        )
    }

    pub fn online(&self, subcmd: &str, args: Vec<&str>) -> Result<Value> {
        let output = Command::new(format!("{}/firma-online", self.exe_dir))
            .arg("--firma-datadir")
            .arg(format!("{}", self.work_dir.path().display()))
            .arg("--network")
            .arg("regtest")
            .arg("--wallet-name")
            .arg(&self.wallet_name)
            .arg(subcmd)
            .args(&args)
            .output()
            .unwrap();
        if !output.status.success() {
            println!("{}", std::str::from_utf8(&output.stderr)?);
        }
        assert!(
            output.status.success(),
            format!("online subcmd:{} args:{:?}", subcmd, args)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        println!("{}", to_string_pretty(&value).unwrap());
        Ok(value)
    }

    pub fn online_create_wallet(
        &self,
        node_url: &str,
        cookie_file: &str,
        required_sig: u8,
        xpubs: &Vec<String>,
    ) -> Result<CreateWalletOutput> {
        let required_sig = format!("{}", required_sig);
        let mut args = vec![
            "--url",
            node_url,
            "--cookie-file",
            cookie_file,
            "-r",
            &required_sig,
        ];
        for xpub in xpubs {
            args.push("--xpub-file");
            args.push(xpub);
        }
        let result = self.online("create-wallet", args);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    fn online_get_address(&self) -> Result<GetAddressOutput> {
        Ok(from_value(self.online("get-address", vec![]).unwrap())?)
    }

    fn online_balance(&self) -> Result<BalanceOutput> {
        Ok(from_value(self.online("balance", vec![]).unwrap())?)
    }

    fn online_list_coins(&self) -> Result<ListCoinsOutput> {
        Ok(from_value(self.online("list-coins", vec![]).unwrap())?)
    }

    fn online_rescan(&self) -> Result<usize> {
        Ok(from_value(
            self.online("rescan", vec!["--start-from", "0"]).unwrap(),
        )?)
    }

    fn online_create_tx(
        &self,
        recipients: Vec<(Address, u64)>,
        psbt_name: &str,
    ) -> Result<CreateTxOutput> {
        let mut args = vec![];
        for recipient in recipients {
            args.push("--recipient".to_string());
            args.push(format!("{}:{}", recipient.0, recipient.1));
        }
        args.push("--psbt-name".to_string());
        args.push(psbt_name.to_string());
        args.push("--qr-version".to_string());
        args.push("20".to_string());
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        let output = from_value(self.online("create-tx", args).unwrap())?;
        Ok(output)
    }

    fn online_send_tx(&self, psbts: Vec<&str>) -> Result<SendTxOutput> {
        let mut args = vec!["--broadcast"];
        for psbt in psbts {
            args.push("--psbt-file");
            args.push(psbt);
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
            Some(_) => (Stdio::piped(), vec!["--read-stdin"]),
            None => (Stdio::null(), vec![]),
        };
        let mut process = Command::new(format!("{}/firma-offline", self.exe_dir))
            .stdin(stdin)
            .stdout(Stdio::piped())
            .arg("--firma-datadir")
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
            format!(
                "offline subcmd:{} args:{:?} encryption_key:{:?}",
                subcmd, args, encryption_key
            )
        );

        let value: Value = serde_json::from_slice(&output.stdout)?;
        println!("{:?}", to_string_pretty(&value));

        Ok(value)
    }

    pub fn offline_random(
        &self,
        key_name: &str,
        encryption_key: Option<&[u8]>,
    ) -> Result<MasterKeyOutput> {
        let result = self.offline("random", vec!["--key-name", key_name], encryption_key);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_decrypt(
        &self,
        file_name: &str,
        encryption_key: Option<&[u8]>,
    ) -> Result<PrivateMasterKeyJson> {
        let result = self.offline("decrypt", vec!["--path", file_name], encryption_key);
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_dice(
        &self,
        key_name: &str,
        launches: Vec<u32>,
        faces: u32,
    ) -> Result<MasterKeyOutput> {
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
    ) -> Result<MasterKeyOutput> {
        let result = self.offline(
            "restore",
            vec!["--key-name", key_name, "--nature", nature, value],
            None,
        );
        let value = map_json_error(result)?;
        let output = from_value(value).unwrap();
        Ok(output)
    }

    pub fn offline_sign(&self, psbt_file: &str, key_file: &str) -> Result<PsbtPrettyPrint> {
        let result = self.offline(
            "sign",
            vec![
                psbt_file,
                "--key",
                key_file,
                "--total-derivations",
                "20",
                "--wallet-descriptor-file",
                &self.wallet_file(),
            ],
            None,
        );
        let value = map_json_error(result)?;
        let output = from_value(value)?;
        Ok(output)
    }

    fn offline_list(&self, kind: Kind, encryption_key: Option<&[u8]>) -> Result<ListOutput> {
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
}

fn client_send_to_address(client: &Client, address: &Address, satoshi: u64) -> Result<Txid> {
    Ok(client
        .send_to_address(
            &address,
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
