use crate::swqos::common::{poll_transaction_confirmation, serialize_transaction_and_encode};
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde_json::json;
use std::{sync::Arc, time::Instant};

use solana_transaction_status::UiTransactionEncoding;
use std::time::Duration;

use crate::swqos::SwqosClientTrait;
use crate::swqos::{SwqosType, TradeType};
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

use crate::{common::SolanaRpcClient, constants::swqos::ZEROSLOT_TIP_ACCOUNTS};

#[derive(Clone)]
pub struct ZeroSlotClient {
    pub endpoint: String,
    pub auth_token: String,
    pub rpc_client: Arc<SolanaRpcClient>,
    pub http_client: Client,
}

#[async_trait::async_trait]
impl SwqosClientTrait for ZeroSlotClient {
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
        let tip_account = *ZEROSLOT_TIP_ACCOUNTS
            .choose(&mut rand::rng())
            .or_else(|| ZEROSLOT_TIP_ACCOUNTS.first())
            .unwrap();
        Ok(tip_account.to_string())
    }

    fn get_swqos_type(&self) -> SwqosType {
        SwqosType::ZeroSlot
    }
}

impl ZeroSlotClient {
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

        let request_body = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                content,
                { "encoding": "base64", "skipPreflight": true }
            ]
        }))?;

        let mut url = String::with_capacity(self.endpoint.len() + self.auth_token.len() + 20);
        url.push_str(&self.endpoint);
        url.push_str("/?api-key=");
        url.push_str(&self.auth_token);

        // 4. 直接使用 `text().await?`，避免 `json().await?` 的异步 JSON 解析
        let response_text = self
            .http_client
            .post(&url)
            .body(request_body) // 直接传字符串，避免 `json()` 开销
            .header("Content-Type", "application/json") // 显式指定 JSON 头
            .send()
            .await?
            .text()
            .await?;

        // 5. 用 `serde_json::from_str()` 解析 JSON，减少 `.json().await?` 额外等待
        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if response_json.get("result").is_some() {
                println!(" 0slot{}提交: {:?}", trade_type, start_time.elapsed());
            } else if let Some(_error) = response_json.get("error") {
                eprintln!(" 0slot{}提交失败: {:?}", trade_type, _error);
            }
        }

        let start_time: Instant = Instant::now();
        match poll_transaction_confirmation(&self.rpc_client, signature).await {
            Ok(_) => (),
            Err(_) => (),
        }

        println!(" 0slot{}确认: {:?}", trade_type, start_time.elapsed());

        Ok(())
    }

    pub async fn send_transactions(
        &self,
        trade_type: TradeType,
        transactions: &Vec<VersionedTransaction>,
    ) -> Result<()> {
        for transaction in transactions {
            self.send_transaction(trade_type, transaction).await?;
        }
        Ok(())
    }
}
