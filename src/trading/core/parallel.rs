use anyhow::{anyhow, Result};
use solana_hash::Hash;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair};
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::{
    common::PriorityFee,
    swqos::{SwqosClient, SwqosType, TradeType},
    trading::{
        common::{
            build_rpc_transaction, build_sell_tip_transaction_with_priority_fee,
            build_sell_transaction, build_tip_transaction_with_priority_fee,
        },
        core::timer::TradeTimer,
        MiddlewareManager,
    },
};

/// 并行执行交易的通用函数
pub async fn parallel_execute_with_tips(
    swqos_clients: Vec<Arc<SwqosClient>>,
    payer: Arc<Keypair>,
    instructions: Vec<Instruction>,
    priority_fee: PriorityFee,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    data_size_limit: u32,
    trade_type: TradeType,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
    wait_transaction_confirmed: bool,
) -> Result<()> {
    let cores = core_affinity::get_core_ids().unwrap();
    let mut handles: Vec<JoinHandle<Result<()>>> = vec![];

    for i in 0..swqos_clients.len() {
        let swqos_client = swqos_clients[i].clone();
        let payer = payer.clone();
        let instructions = instructions.clone();
        let mut priority_fee = priority_fee.clone();
        let core_id = cores[i % cores.len()];

        let middleware_manager = middleware_manager.clone();
        let protocol_name = protocol_name.clone();

        let handle = tokio::spawn(async move {
            core_affinity::set_for_current(core_id);

            let mut timer =
                TradeTimer::new(format!("构建交易指令: {:?}", swqos_client.get_swqos_type()));

            let transaction = if matches!(trade_type, TradeType::Sell)
                && swqos_client.get_swqos_type() == SwqosType::Default
            {
                build_sell_transaction(
                    payer,
                    &priority_fee,
                    instructions,
                    lookup_table_key,
                    recent_blockhash,
                    middleware_manager,
                    protocol_name,
                    is_buy,
                )
                .await?
            } else if matches!(trade_type, TradeType::Sell)
                && swqos_client.get_swqos_type() != SwqosType::Default
            {
                let tip_account = swqos_client.get_tip_account()?;
                let tip_account = Arc::new(Pubkey::from_str(&tip_account).map_err(|e| anyhow!(e))?);
                build_sell_tip_transaction_with_priority_fee(
                    payer,
                    &priority_fee,
                    instructions,
                    &tip_account,
                    lookup_table_key,
                    recent_blockhash,
                    middleware_manager,
                    protocol_name,
                    is_buy,
                )
                .await?
            } else if swqos_client.get_swqos_type() == SwqosType::Default {
                build_rpc_transaction(
                    payer,
                    &priority_fee,
                    instructions,
                    lookup_table_key,
                    recent_blockhash,
                    data_size_limit,
                    middleware_manager,
                    protocol_name,
                    is_buy,
                )
                .await?
            } else {
                let tip_account = swqos_client.get_tip_account()?;
                let tip_account = Arc::new(Pubkey::from_str(&tip_account).map_err(|e| anyhow!(e))?);
                priority_fee.buy_tip_fee = priority_fee.buy_tip_fees[i % priority_fee.buy_tip_fees.len()];

                build_tip_transaction_with_priority_fee(
                    payer,
                    &priority_fee,
                    instructions,
                    &tip_account,
                    lookup_table_key,
                    recent_blockhash,
                    data_size_limit,
                    middleware_manager,
                    protocol_name,
                    is_buy,
                )
                .await?
            };

            timer.stage(format!("提交交易指令: {:?}", swqos_client.get_swqos_type()));

            swqos_client.send_transaction(trade_type, &transaction).await?;

            timer.finish();
            Ok::<(), anyhow::Error>(())
        });

        handles.push(handle);
    }

    // 任意一个成功即返回
    let (tx, mut rx) = mpsc::channel(swqos_clients.len());

    // 启动监听任务
    for handle in handles {
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = handle.await;
            let _ = tx.send(result).await;
        });
    }
    drop(tx); // 关闭发送端

    // 等待第一个成功的结果
    let mut errors = Vec::new();

    if !wait_transaction_confirmed {
        return Ok(());
    }

    while let Some(result) = rx.recv().await {
        match result {
            Ok(Ok(_)) => {
                return Ok(());
            }
            Ok(Err(e)) => errors.push(format!("Task error: {}", e)),
            Err(e) => errors.push(format!("Join error: {}", e)),
        }
    }

    // 如果没有成功的，返回错误
    return Err(anyhow!("所有交易都失败了: {:?}", errors));
}
