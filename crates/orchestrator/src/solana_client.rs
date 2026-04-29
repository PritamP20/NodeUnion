use anyhow::{anyhow, Context, Result};
use borsh::BorshSerialize;
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    system_program,
    transaction::Transaction,
};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

/// Solana client for interacting with NodeUnion billing contract on devnet
#[derive(Clone)]
pub struct SolanaClient {
    pub rpc_url: String,
    pub payer_keypair_path: String,
    pub program_id: String,
    pub token_mint: Option<String>,
    pub rpc_client: Arc<RpcClient>,
}

#[derive(BorshSerialize)]
struct RegisterNetworkArgs {
    network_id: String,
    name: String,
    price_per_unit: u64,
}

#[derive(BorshSerialize)]
struct RegisterProviderArgs {
    network_id: String,
    provider_id: String,
    provider_wallet: Pubkey,
}

#[derive(BorshSerialize)]
struct OpenEscrowArgs {
    job_id: String,
    network_id: String,
    provider_id: String,
    max_units: u64,
    deposit_amount: u64,
}

#[derive(BorshSerialize)]
struct RecordUsageArgs {
    units: u64,
}

#[derive(BorshSerialize)]
struct CloseEscrowArgs {}

impl SolanaClient {
    pub fn from_env() -> Result<Self> {
        let rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let keypair_path = env::var("SOLANA_PAYER_KEYPAIR")
            .unwrap_or_else(|_| format!("{}/.config/solana/id.json", env::var("HOME").unwrap_or_else(|_| ".".to_string())));
        
        let program_id = env::var("SOLANA_PROGRAM_ID")
            .unwrap_or_else(|_| "9EELXCE4Y27Crja8RttcnTdKxL7rMbYCt1W7efoNmzQo".to_string());

        let token_mint = env::var("SOLANA_TOKEN_MINT").ok();

        Ok(SolanaClient {
            rpc_client: Arc::new(RpcClient::new(rpc_url.clone())),
            rpc_url,
            payer_keypair_path: keypair_path,
            program_id,
            token_mint,
        })
    }

    fn load_keypair(&self) -> Result<Keypair> {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = self.payer_keypair_path.replace("~", &home);
        read_keypair_file(&path).map_err(|e| anyhow!("Failed to read keypair from {}: {}", path, e))
    }

    async fn get_payer_pubkey(&self) -> Result<String> {
        let keypair = self.load_keypair()?;
        Ok(keypair.pubkey().to_string())
    }

    fn parse_pubkey(&self, value: &str, label: &str) -> Result<Pubkey> {
        Pubkey::from_str(value).with_context(|| format!("Invalid {} pubkey: {}", label, value))
    }

    fn program_pubkey(&self) -> Result<Pubkey> {
        self.parse_pubkey(&self.program_id, "program id")
    }

    fn token_mint_pubkey(&self) -> Result<Pubkey> {
        let mint = self
            .token_mint
            .as_ref()
            .ok_or_else(|| anyhow!("SOLANA_TOKEN_MINT is required for escrow operations"))?;
        self.parse_pubkey(mint, "token mint")
    }

    fn token_program_pubkey() -> Result<Pubkey> {
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            .map_err(|e| anyhow!("Invalid SPL token program id: {}", e))
    }

    pub async fn get_network_price_per_unit(&self, network_id: &str) -> Result<u64> {
        let program_id = self.program_pubkey()?;
        let (registry_pubkey, _) = Pubkey::find_program_address(
            &[b"network", network_id.as_bytes()],
            &program_id,
        );

        // Fetch the account data from the RPC
        let account = self
            .rpc_client
            .get_account(&registry_pubkey)
            .await
            .map_err(|e| anyhow!("Failed to fetch network registry account: {}", e))?;

        // Parse the account data
        // Format: 8 bytes discriminator + borsh-encoded NetworkRegistry
        if account.data.len() < 8 {
            return Err(anyhow!("Invalid network registry account data size"));
        }

        // Skip 8-byte Anchor discriminator
        let data = &account.data[8..];

        // Manually deserialize to extract price_per_unit (it's a u64)
        // NetworkRegistry structure (in order):
        // - network_id: String (4 bytes len + data)
        // - name: String (4 bytes len + data)
        // - price_per_unit: u64 (8 bytes)
        // - authority: Pubkey (32 bytes)
        // - created_at: i64 (8 bytes)
        // - bump: u8 (1 byte)

        let mut offset = 0;

        // Skip network_id string
        if data.len() < offset + 4 {
            return Err(anyhow!("Invalid network registry data"));
        }
        let id_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4 + id_len;

        // Skip name string
        if data.len() < offset + 4 {
            return Err(anyhow!("Invalid network registry data"));
        }
        let name_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4 + name_len;

        // Read price_per_unit
        if data.len() < offset + 8 {
            return Err(anyhow!("Invalid network registry data"));
        }
        let price_per_unit = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        Ok(price_per_unit)
    }

    fn anchor_discriminator(name: &str) -> [u8; 8] {
        let preimage = format!("global:{}", name);
        let digest = Sha256::digest(preimage.as_bytes());
        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(&digest[..8]);
        discriminator
    }

    fn anchor_ix_data<T: BorshSerialize>(name: &str, args: &T) -> Result<Vec<u8>> {
        let mut data = Self::anchor_discriminator(name).to_vec();
        data.extend(
            borsh::to_vec(args)
                .map_err(|e| anyhow!("Failed to serialize args for {}: {}", name, e))?,
        );
        Ok(data)
    }

    async fn submit_transaction(&self, instructions: Vec<Instruction>) -> Result<Signature> {
        let payer = self.load_keypair()?;
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        let sig = self.rpc_client.send_and_confirm_transaction(&tx).await?;
        println!("[SOLANA] Transaction submitted: {}", sig);
        Ok(sig)
    }

    pub async fn register_network_on_chain(
        &self,
        network_id: &str,
        name: &str,
        price_per_unit: u64,
    ) -> Result<String> {
        println!("[SOLANA] Registering network {} with price {} per unit on devnet", network_id, price_per_unit);

        let program_id = self.program_pubkey()?;
        let payer = self.load_keypair()?;
        let (registry, _) = Pubkey::find_program_address(&[b"network", network_id.as_bytes()], &program_id);

        let data = Self::anchor_ix_data(
            "register_network",
            &RegisterNetworkArgs {
                network_id: network_id.to_string(),
                name: name.to_string(),
                price_per_unit,
            },
        )?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(registry, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        let sig = self.submit_transaction(vec![ix]).await?;
        Ok(sig.to_string())
    }

    pub async fn register_provider_on_chain(
        &self,
        network_id: &str,
        provider_id: &str,
        provider_wallet: &str,
    ) -> Result<String> {
        println!("[SOLANA] Registering provider {} in network {} on devnet", provider_id, network_id);

        let provider_wallet = self.parse_pubkey(provider_wallet, "provider wallet")?;
        let program_id = self.program_pubkey()?;
        let payer = self.load_keypair()?;
        let (provider, _) = Pubkey::find_program_address(
            &[b"provider", network_id.as_bytes(), provider_id.as_bytes()],
            &program_id,
        );

        let data = Self::anchor_ix_data(
            "register_provider",
            &RegisterProviderArgs {
                network_id: network_id.to_string(),
                provider_id: provider_id.to_string(),
                provider_wallet,
            },
        )?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(provider, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        let sig = self.submit_transaction(vec![ix]).await?;
        Ok(sig.to_string())
    }

    pub async fn open_escrow_on_chain(
        &self,
        job_id: &str,
        network_id: &str,
        provider_id: &str,
        max_units: u64,
        deposit_amount: u64,
        user_wallet: &str,
    ) -> Result<String> {
        println!("[SOLANA] Opening escrow for job {} on devnet", job_id);

        let program_id = self.program_pubkey()?;
        let payer = self.load_keypair()?;
        let user = self.parse_pubkey(user_wallet, "user wallet")?;
        if user != payer.pubkey() {
            return Err(anyhow!(
                "open_escrow requires a signer for user_wallet; current backend supports only custodial mode where user_wallet == payer"
            ));
        }

        let _mint = self.token_mint_pubkey()?;
        let token_program = Self::token_program_pubkey()?;

        let (escrow, _) = Pubkey::find_program_address(&[b"escrow", job_id.as_bytes()], &program_id);
        let (config, _) = Pubkey::find_program_address(&[b"config"], &program_id);
        let (network_registry, _) =
            Pubkey::find_program_address(&[b"network", network_id.as_bytes()], &program_id);
        let (provider_registry, _) = Pubkey::find_program_address(
            &[b"provider", network_id.as_bytes(), provider_id.as_bytes()],
            &program_id,
        );

        let user_token_account = self.parse_pubkey(
            &env::var("SOLANA_USER_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_USER_TOKEN_ACCOUNT is required for open_escrow_on_chain"
                )
            })?,
            "user token account",
        )?;

        let escrow_token_account = self.parse_pubkey(
            &env::var("SOLANA_ESCROW_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_ESCROW_TOKEN_ACCOUNT is required for open_escrow_on_chain"
                )
            })?,
            "escrow token account",
        )?;

        let data = Self::anchor_ix_data(
            "open_escrow",
            &OpenEscrowArgs {
                job_id: job_id.to_string(),
                network_id: network_id.to_string(),
                provider_id: provider_id.to_string(),
                max_units,
                deposit_amount,
            },
        )?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(user, true),
                AccountMeta::new(user_token_account, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(escrow_token_account, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(network_registry, false),
                AccountMeta::new_readonly(provider_registry, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        let sig = self.submit_transaction(vec![ix]).await?;
        Ok(sig.to_string())
    }

    pub async fn record_usage_on_chain(
        &self,
        job_id: &str,
        units: u64,
    ) -> Result<String> {
        println!("[SOLANA] Recording usage for job {} ({} units)", job_id, units);

        let program_id = self.program_pubkey()?;
        let payer = self.load_keypair()?;
        let token_program = Self::token_program_pubkey()?;

        // Derive escrow PDA
        let (escrow, _) = Pubkey::find_program_address(&[b"escrow", job_id.as_bytes()], &program_id);
        
        // Derive config PDA
        let (config, _) = Pubkey::find_program_address(&[b"config"], &program_id);

        // Get token account addresses from environment
        let escrow_token_account = self.parse_pubkey(
            &env::var("SOLANA_ESCROW_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_ESCROW_TOKEN_ACCOUNT is required for record_usage_on_chain"
                )
            })?,
            "escrow token account",
        )?;

        let provider_token_account = self.parse_pubkey(
            &env::var("SOLANA_PROVIDER_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_PROVIDER_TOKEN_ACCOUNT is required for record_usage_on_chain"
                )
            })?,
            "provider token account",
        )?;

        let data = Self::anchor_ix_data("record_usage", &RecordUsageArgs { units })?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(escrow, false),
                AccountMeta::new(escrow_token_account, false),
                AccountMeta::new(provider_token_account, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(token_program, false),
            ],
            data,
        };

        let sig = self.submit_transaction(vec![ix]).await?;
        Ok(sig.to_string())
    }

    pub async fn close_escrow_on_chain(
        &self,
        job_id: &str,
    ) -> Result<String> {
        println!("[SOLANA] Closing escrow for job {}", job_id);

        let program_id = self.program_pubkey()?;
        let payer = self.load_keypair()?;
        let token_program = Self::token_program_pubkey()?;

        // Derive escrow PDA
        let (escrow, _) = Pubkey::find_program_address(&[b"escrow", job_id.as_bytes()], &program_id);
        
        // Derive config PDA
        let (config, _) = Pubkey::find_program_address(&[b"config"], &program_id);

        // Get token account addresses from environment
        let escrow_token_account = self.parse_pubkey(
            &env::var("SOLANA_ESCROW_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_ESCROW_TOKEN_ACCOUNT is required for close_escrow_on_chain"
                )
            })?,
            "escrow token account",
        )?;

        let user_token_account = self.parse_pubkey(
            &env::var("SOLANA_USER_TOKEN_ACCOUNT").map_err(|_| {
                anyhow!(
                    "SOLANA_USER_TOKEN_ACCOUNT is required for close_escrow_on_chain"
                )
            })?,
            "user token account",
        )?;

        let data = Self::anchor_ix_data("close_escrow", &CloseEscrowArgs {})?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),  // authority
                AccountMeta::new(escrow, false),
                AccountMeta::new(escrow_token_account, false),
                AccountMeta::new(user_token_account, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(token_program, false),
            ],
            data,
        };

        let sig = self.submit_transaction(vec![ix]).await?;
        Ok(sig.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solana_client_from_env() {
        let client = SolanaClient::from_env().unwrap();
        assert_eq!(client.rpc_url, "https://api.devnet.solana.com");
        assert!(client.program_id.len() > 0);
    }
}
