use anyhow::{Context, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

// System program ID
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
// Your deployed program ID from target/deploy/merkle_program-keypair.json
const PROGRAM_ID: &str = "AHpuc2M3wbZceufaiE4Q2wyDXh198ymB1SxxpbxCzj3H";

pub struct SolanaClient {
    rpc_client: RpcClient,
    authority_keypair: Keypair,
}

impl SolanaClient {
    /// Initialize Solana client with RPC URL and authority keypair path
    pub fn new(rpc_url: &str, keypair_path: &str) -> Result<Self> {
        let rpc_client =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

        let authority_keypair = read_keypair_file(keypair_path)
            .map_err(|e| anyhow::anyhow!("Failed to read authority keypair: {}", e))?;

        Ok(Self {
            rpc_client,
            authority_keypair,
        })
    }

    /// Derive the config PDA (must match the Anchor program)
    fn get_config_pda(&self) -> Result<(Pubkey, u8)> {
        let program_id = Pubkey::from_str(PROGRAM_ID)?;
        let (pda, bump) = Pubkey::find_program_address(&[b"config"], &program_id);
        Ok((pda, bump))
    }

    /// Initialize the subscription config with an initial merkle root
    pub async fn initialize_config(&self, initial_root: [u8; 32]) -> Result<Signature> {
        let program_id = Pubkey::from_str(PROGRAM_ID)?;
        let (config_pda, _bump) = self.get_config_pda()?;

        // Build instruction data: discriminator (8 bytes) + root (32 bytes)
        // Discriminator from IDL: [175, 175, 109, 31, 13, 152, 155, 237]
        let mut instruction_data = Vec::new();
        let discriminator: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
        instruction_data.extend_from_slice(&discriminator);
        instruction_data.extend_from_slice(&initial_root);

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(config_pda, false),
                AccountMeta::new(self.authority_keypair.pubkey(), true),
                AccountMeta::new_readonly(Pubkey::from_str(SYSTEM_PROGRAM_ID)?, false),
            ],
            data: instruction_data,
        };

        let signature = self.send_transaction(&[instruction]).await?;

        println!("✅ Initialized config on-chain");
        println!("   Config PDA: {}", config_pda);
        println!("   Signature: {}", signature);

        Ok(signature)
    }

    /// Update the merkle root on-chain
    pub async fn update_merkle_root(&self, new_root: [u8; 32]) -> Result<Signature> {
        let program_id = Pubkey::from_str(PROGRAM_ID)?;
        let (config_pda, _bump) = self.get_config_pda()?;

        // Build instruction data: discriminator + new_root
        // Discriminator from IDL: [58, 195, 57, 246, 116, 198, 170, 138]
        let mut instruction_data = Vec::new();
        let discriminator: [u8; 8] = [58, 195, 57, 246, 116, 198, 170, 138];
        instruction_data.extend_from_slice(&discriminator);
        instruction_data.extend_from_slice(&new_root);

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(config_pda, false),
                AccountMeta::new_readonly(self.authority_keypair.pubkey(), true),
            ],
            data: instruction_data,
        };

        let signature = self.send_transaction(&[instruction]).await?;

        println!("✅ Updated merkle root on-chain");
        println!("   New Root: {}", hex::encode(new_root));
        println!("   Signature: {}", signature);

        Ok(signature)
    }

    /// Get the current merkle root from on-chain config
    pub async fn get_current_root(&self) -> Result<[u8; 32]> {
        let (config_pda, _bump) = self.get_config_pda()?;

        let account_data = self
            .rpc_client
            .get_account_data(&config_pda)
            .context("Failed to fetch config account. Has it been initialized?")?;

        // Anchor account layout: 8-byte discriminator + account data
        // SubscriptionConfig: authority(32) + merkle_root(32) + bump(1)
        if account_data.len() < 8 + 32 + 32 {
            return Err(anyhow::anyhow!("Invalid account data length"));
        }

        // Root is at offset 8 (discriminator) + 32 (authority) = 40
        let mut root = [0u8; 32];
        root.copy_from_slice(&account_data[40..72]);

        Ok(root)
    }

    /// Helper to reduce code duplication
    async fn send_transaction(&self, instructions: &[Instruction]) -> Result<Signature> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.authority_keypair.pubkey()),
            &[&self.authority_keypair],
            recent_blockhash,
        );

        self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .context("Failed to send transaction")
    }
}
