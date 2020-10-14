use bitcoincore_rpc::{Auth, Client, RpcApi};
use firma::*;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;
use std::{env, thread};
use tempdir::TempDir;

pub struct BitcoinD {
    process: Child,
    pub client: Client,
    _work_dir: TempDir, // to keep the temp directory as long as the process runs
    pub cookie_file: PathBuf,
    pub url: String,
}

impl BitcoinD {
    pub fn new() -> BitcoinD {
        let exe = env::var("BITCOIND_EXE").expect("BITCOIND_EXE env var must be set");
        let _work_dir = TempDir::new("bitcoin_test_firma").unwrap();
        let cookie_file = _work_dir.path().join("regtest").join(".cookie");
        let rpc_port = 18242u16;
        let url = format!("http://127.0.0.1:{}", rpc_port);

        let process = Command::new(exe)
            .arg(format!("-datadir={}", _work_dir.path().display()))
            .arg(format!("-rpcport={}", rpc_port))
            .arg("-regtest")
            .arg("-listen=0")
            .arg("-fallbackfee=0.0001")
            .spawn()
            .unwrap();

        let node_url_default = format!("{}/wallet/default", url);
        // wait bitcoind is ready, use default wallet
        let client = loop {
            thread::sleep(Duration::from_millis(500));
            assert!(process.stderr.is_none());
            let client_result = Client::new(url.clone(), Auth::CookieFile(cookie_file.clone()));
            if let Ok(client_base) = client_result {
                if let Ok(_) = client_base.get_blockchain_info() {
                    client_base
                        .create_wallet("default", None, None, None, None)
                        .unwrap();
                    break Client::new(
                        node_url_default.clone(),
                        Auth::CookieFile(cookie_file.clone()),
                    )
                    .unwrap();
                }
            }
        };

        BitcoinD {
            process,
            client,
            _work_dir,
            cookie_file,
            url,
        }
    }

    pub fn wait(&mut self) -> Result<ExitStatus> {
        Ok(self.process.wait()?)
    }
}

impl Drop for BitcoinD {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
