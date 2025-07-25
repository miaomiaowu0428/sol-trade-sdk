use crate::swqos::common::{
    poll_transaction_confirmation, serialize_transaction_and_encode,
    FormatBase64VersionedTransaction,
};
use rand::seq::IndexedRandom;
use reqwest::Client;
use std::{sync::Arc, time::Instant};

use solana_transaction_status::UiTransactionEncoding;
use std::time::Duration;

use crate::swqos::SwqosClientTrait;
use crate::swqos::{SwqosType, TradeType};
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

use crate::{common::SolanaRpcClient, constants::swqos::BLOX_TIP_ACCOUNTS};

#[derive(Clone)]
pub struct BloxrouteClient {
    pub endpoint: String,
    pub auth_token: String,
    pub rpc_client: Arc<SolanaRpcClient>,
    pub http_client: Client,
}

#[async_trait::async_trait]
impl SwqosClientTrait for BloxrouteClient {
    async fn send_transaction(
        &self,
        trade_type: TradeType,
        transaction: &VersionedTransaction,
    ) -> Result<()> {
        self.send_transaction(trade_type, transaction).await
    }

    async fn send_transactions(
        &self,
        trade_type: TradeType,
        transactions: &Vec<VersionedTransaction>,
    ) -> Result<()> {
        self.send_transactions(trade_type, transactions).await
    }

    fn get_tip_account(&self) -> Result<String> {
        let tip_account = *BLOX_TIP_ACCOUNTS
            .choose(&mut rand::rng())
            .or_else(|| BLOX_TIP_ACCOUNTS.first())
            .unwrap();
        Ok(tip_account.to_string())
    }

    fn get_swqos_type(&self) -> SwqosType {
        SwqosType::Bloxroute
    }
}

impl BloxrouteClient {
    pub fn new(rpc_url: String, endpoint: String, auth_token: String) -> Self {
        let rpc_client = SolanaRpcClient::new(rpc_url);
        let http_client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(64)
            .tcp_keepalive(Some(Duration::from_secs(1200)))
            .http2_keep_alive_interval(Duration::from_secs(15))
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self {
            rpc_client: Arc::new(rpc_client),
            endpoint,
            auth_token,
            http_client,
        }
    }

    pub async fn send_transaction(
        &self,
        trade_type: TradeType,
        transaction: &VersionedTransaction,
    ) -> Result<()> {
        let start_time = Instant::now();
        let (content, signature) =
            serialize_transaction_and_encode(transaction, UiTransactionEncoding::Base64).await?;
        println!(" 交易编码base64: {:?}", start_time.elapsed());

        let body = serde_json::json!({
            "transaction": {
                "content": content,
            },
            "frontRunningProtection": false,
            "useStakedRPCs": true,
        });

        let endpoint = format!("{}/api/v2/submit", self.endpoint);
        let response_text = self
            .http_client
            .post(&endpoint)
            .body(body.to_string())
            .header("Content-Type", "application/json")
            .header("Authorization", self.auth_token.clone())
            .send()
            .await?
            .text()
            .await?;

        // 5. 用 `serde_json::from_str()` 解析 JSON，减少 `.json().await?` 额外等待
        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if response_json.get("result").is_some() {
                println!(" bloxroute{}提交: {:?}", trade_type, start_time.elapsed());
            } else if let Some(_error) = response_json.get("error") {
                eprintln!(" bloxroute{}提交失败: {:?}", trade_type, _error);
            }
        }

        let start_time: Instant = Instant::now();
        match poll_transaction_confirmation(&self.rpc_client, signature).await {
            Ok(_) => (),
            Err(_) => (),
        }

        println!(" bloxroute{}确认: {:?}", trade_type, start_time.elapsed());

        Ok(())
    }

    pub async fn send_transactions(
        &self,
        trade_type: TradeType,
        transactions: &Vec<VersionedTransaction>,
    ) -> Result<()> {
        let start_time = Instant::now();
        println!(" 交易编码base64: {:?}", start_time.elapsed());

        let body = serde_json::json!({
            "entries":  transactions
                .iter()
                .map(|tx| {
                    serde_json::json!({
                        "transaction": {
                            "content": tx.to_base64_string(),
                        },
                    })
                })
                .collect::<Vec<_>>(),
        });

        let endpoint = format!("{}/api/v2/submit-batch", self.endpoint);
        let response_text = self
            .http_client
            .post(&endpoint)
            .body(body.to_string())
            .header("Content-Type", "application/json")
            .header("Authorization", self.auth_token.clone())
            .send()
            .await?
            .text()
            .await?;

        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if response_json.get("result").is_some() {
                println!(" bloxroute{}提交: {:?}", trade_type, start_time.elapsed());
            } else if let Some(_error) = response_json.get("error") {
                eprintln!(" bloxroute{}提交失败: {:?}", trade_type, _error);
            }
        }

        Ok(())
    }
}
