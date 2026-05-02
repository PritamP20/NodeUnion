use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::env;
use std::process::Command;

#[derive(Clone)]
pub struct StellarClient {
    pub network: String,
    pub source_account: String,
    pub source_account_address: String,
    pub contract_id: String,
    pub token_address: String,
    pub treasury_address: String,
    pub rate_per_unit: u128,
}

impl StellarClient {
    pub fn from_env() -> Result<Self> {
        let network = env::var("STELLAR_NETWORK").unwrap_or_else(|_| "testnet".to_string());
        let source_account = env::var("STELLAR_SOURCE_ACCOUNT")
            .or_else(|_| env::var("STELLAR_ACCOUNT"))
            .unwrap_or_else(|_| "nodeunion-test".to_string());
        let contract_id = env::var("STELLAR_CONTRACT_ID").unwrap_or_else(|_| {
            "CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K".to_string()
        });
        let rate_per_unit = env::var("STELLAR_RATE_PER_UNIT")
            .ok()
            .and_then(|value| value.parse::<u128>().ok())
            .unwrap_or(100);

        let source_account_address = Self::resolve_identity_address(&source_account)?;
        let token_address = env::var("STELLAR_TOKEN_ADDRESS")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| source_account_address.clone());
        let treasury_address = env::var("STELLAR_TREASURY_ADDRESS")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| source_account_address.clone());

        Ok(Self {
            network,
            source_account,
            source_account_address,
            contract_id,
            token_address,
            treasury_address,
            rate_per_unit,
        })
    }

    fn resolve_identity_address(identity: &str) -> Result<String> {
        let output = Command::new("stellar")
            .args(["keys", "public-key", identity])
            .output()
            .with_context(|| format!("failed to resolve Stellar identity '{}'", identity))?;

        if !output.status.success() {
            return Err(anyhow!(
                "stellar keys public-key {} failed: {}",
                identity,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let address = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if address.is_empty() {
            return Err(anyhow!("stellar keys public-key returned an empty address"));
        }

        Ok(address)
    }

    fn job_key(job_id: &str) -> u64 {
        let digest = Sha256::digest(job_id.as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        let value = u64::from_le_bytes(bytes);
        if value == 0 { 1 } else { value }
    }

    fn invoke(&self, function_name: &str, args: &[String], send: bool) -> Result<String> {
        let mut command = Command::new("stellar");
        command.args([
            "contract",
            "invoke",
            "--id",
            &self.contract_id,
            "--source-account",
            &self.source_account,
            "--network",
            &self.network,
            "--send",
            if send { "yes" } else { "no" },
            "--",
            function_name,
        ]);

        for arg in args {
            command.arg(arg);
        }

        let output = command
            .output()
            .with_context(|| format!("failed to invoke Stellar contract function '{}'", function_name))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(stdout);
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(anyhow!(
            "stellar contract invoke {} failed: {}{}{}",
            function_name,
            stderr,
            if stderr.is_empty() || stdout.is_empty() { "" } else { " | " },
            stdout
        ))
    }

    fn invoke_view(&self, function_name: &str, args: &[String]) -> Result<String> {
        self.invoke(function_name, args, false)
    }

    fn invoke_mutation(&self, function_name: &str, args: &[String]) -> Result<String> {
        self.invoke(function_name, args, true)
    }

    pub async fn ensure_initialized(&self) -> Result<()> {
        if self.invoke_view("get_config", &[]).is_ok() {
            return Ok(());
        }

        let args = vec![
            "--authority".to_string(),
            self.source_account_address.clone(),
            "--token".to_string(),
            self.token_address.clone(),
            "--treasury".to_string(),
            self.treasury_address.clone(),
            "--rate_per_unit".to_string(),
            self.rate_per_unit.to_string(),
        ];

        self.invoke_mutation("initialize_config", &args).map(|_| ())
    }

    pub async fn get_network_price_per_unit(&self) -> Result<u128> {
        Ok(self.rate_per_unit)
    }

    pub async fn register_network_on_chain(
        &self,
        network_id: &str,
        _name: &str,
        _price_per_unit: u64,
    ) -> Result<String> {
        Ok(format!(
            "network '{}' is tracked locally; Stellar handles escrow settlement",
            network_id
        ))
    }

    pub async fn register_provider_on_chain(
        &self,
        network_id: &str,
        provider_id: &str,
        provider_wallet: &str,
    ) -> Result<String> {
        Ok(format!(
            "provider '{}' on network '{}' mapped to Stellar wallet {}",
            provider_id, network_id, provider_wallet
        ))
    }

    pub async fn open_escrow_on_chain(
        &self,
        job_id: &str,
        max_units: u128,
        deposit_amount: u128,
        provider_wallet: &str,
    ) -> Result<String> {
        let job_key = Self::job_key(job_id);
        let args = vec![
            "--job_id".to_string(),
            job_key.to_string(),
            "--max_units".to_string(),
            max_units.to_string(),
            "--deposit_amount".to_string(),
            deposit_amount.to_string(),
            "--provider_wallet".to_string(),
            provider_wallet.to_string(),
        ];

        self.invoke_mutation("open_escrow", &args)
    }

    pub async fn record_usage_on_chain(&self, job_id: &str, units: u128) -> Result<String> {
        let job_key = Self::job_key(job_id);
        let args = vec![
            "--job_id".to_string(),
            job_key.to_string(),
            "--units".to_string(),
            units.to_string(),
        ];

        self.invoke_mutation("record_usage", &args)
    }

    pub async fn close_escrow_on_chain(&self, job_id: &str) -> Result<String> {
        let job_key = Self::job_key(job_id);
        let args = vec!["--job_id".to_string(), job_key.to_string()];

        self.invoke_mutation("close_escrow", &args)
    }

    pub async fn check_token_balance(&self, token_account: &str) -> Result<u64> {
        if token_account.trim().is_empty() {
            return Err(anyhow!("token account cannot be empty"));
        }

        Ok(0)
    }

    pub async fn check_user_balance(&self, user_wallet: &str) -> Result<u64> {
        if user_wallet.trim().is_empty() {
            return Err(anyhow!("user wallet cannot be empty"));
        }

        Ok(0)
    }
}
