use std::{sync::Arc, time::Instant};

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{commitment_config::CommitmentLevel, transaction::VersionedTransaction};
use solana_transaction_status::UiTransactionEncoding;

use crate::swqos::SwqosClientTrait;
use crate::{
    common::SolanaRpcClient,
    swqos::{common::poll_transaction_confirmation, SwqosType, TradeType},
};
use anyhow::Result;

#[derive(Clone)]
pub struct SolRpcClient {
    pub rpc_client: Arc<SolanaRpcClient>,
}

#[async_trait::async_trait]
impl SwqosClientTrait for SolRpcClient {
    async fn send_transaction(
        &self,
        trade_type: TradeType,
        transaction: &VersionedTransaction,
    ) -> Result<()> {
        let signature = self
            .rpc_client
            .send_transaction_with_config(
                transaction,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    preflight_commitment: Some(CommitmentLevel::Processed),
                    encoding: Some(UiTransactionEncoding::Base64),
                    max_retries: Some(3),
                    min_context_slot: Some(0),
                },
            )
            .await?;

        let start_time = Instant::now();
        match poll_transaction_confirmation(&self.rpc_client, signature).await {
            Ok(_) => (),
            Err(_) => (),
        }
        println!(" signature: {:?}", signature);
        println!(" rpc{}确认: {:?}", trade_type, start_time.elapsed());

        Ok(())
    }

    async fn send_transactions(
        &self,
        trade_type: TradeType,
        transactions: &Vec<VersionedTransaction>,
    ) -> Result<()> {
        for transaction in transactions {
            self.send_transaction(trade_type, transaction).await?;
        }
        Ok(())
    }

    fn get_tip_account(&self) -> Result<String> {
        Ok("".to_string())
    }

    fn get_swqos_type(&self) -> SwqosType {
        SwqosType::Default
    }
}

impl SolRpcClient {
    pub fn new(rpc_client: Arc<SolanaRpcClient>) -> Self {
        Self { rpc_client }
    }
}
