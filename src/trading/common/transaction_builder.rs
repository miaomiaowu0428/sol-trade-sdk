use solana_hash::Hash;
use solana_sdk::{
    instruction::Instruction,
    message::{v0, VersionedMessage},
    native_token::sol_str_to_lamports,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::VersionedTransaction,
};
use solana_system_interface::instruction::transfer;
use std::sync::Arc;

use super::{
    address_lookup_manager::get_address_lookup_table_accounts,
    compute_budget_manager::{
        add_rpc_compute_budget_instructions, add_tip_compute_budget_instructions,
    },
    nonce_manager::{add_nonce_instruction, get_transaction_blockhash},
};
use crate::{
    common::PriorityFee,
    trading::{
        common::{add_sell_compute_budget_instructions, add_sell_tip_compute_budget_instructions},
        MiddlewareManager,
    },
};

/// 构建标准的RPC交易
pub async fn build_rpc_transaction(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    data_size_limit: u32,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    let mut instructions = vec![];

    // 添加nonce指令
    if let Err(e) = add_nonce_instruction(&mut instructions, payer.as_ref()) {
        return Err(e);
    }

    // 添加计算预算指令
    add_rpc_compute_budget_instructions(&mut instructions, priority_fee, data_size_limit);

    // 添加业务指令
    instructions.extend(business_instructions);

    // 获取交易使用的blockhash
    let blockhash = get_transaction_blockhash(recent_blockhash);

    // 获取地址查找表账户
    let address_lookup_table_accounts = get_address_lookup_table_accounts(lookup_table_key).await;

    // 构建交易
    build_versioned_transaction(
        payer,
        instructions,
        address_lookup_table_accounts,
        blockhash,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}

/// 构建带小费的交易
pub async fn build_tip_transaction(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    tip_account: &Pubkey,
    tip_amount: f64,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    data_size_limit: u32,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    let mut instructions = vec![];

    // 添加nonce指令
    if let Err(e) = add_nonce_instruction(&mut instructions, payer.as_ref()) {
        return Err(e);
    }

    // 添加计算预算指令
    add_tip_compute_budget_instructions(&mut instructions, priority_fee, data_size_limit);

    // 添加业务指令
    instructions.extend(business_instructions);

    // 添加小费转账指令
    instructions.push(transfer(
        &payer.pubkey(),
        tip_account,
        sol_str_to_lamports(tip_amount.to_string().as_str()).unwrap_or(0),
    ));

    // 获取交易使用的blockhash
    let blockhash = get_transaction_blockhash(recent_blockhash);

    // 获取地址查找表账户
    let address_lookup_table_accounts = get_address_lookup_table_accounts(lookup_table_key).await;

    // 构建交易
    build_versioned_transaction(
        payer,
        instructions,
        address_lookup_table_accounts,
        blockhash,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}

/// 构建版本化交易的底层函数
async fn build_versioned_transaction(
    payer: Arc<Keypair>,
    instructions: Vec<Instruction>,
    address_lookup_table_accounts: Vec<solana_sdk::message::AddressLookupTableAccount>,
    blockhash: Hash,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    let full_instructions = match middleware_manager {
        Some(middleware_manager) => middleware_manager
            .apply_middlewares_process_full_instructions(instructions, protocol_name, is_buy)?,
        None => instructions,
    };
    let v0_message: v0::Message = v0::Message::try_compile(
        &payer.pubkey(),
        &full_instructions,
        &address_lookup_table_accounts,
        blockhash,
    )?;

    let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message.clone());
    let transaction = VersionedTransaction::try_new(versioned_message, &[payer.as_ref()])?;

    Ok(transaction)
}

/// 构建带小费的交易（使用PriorityFee中的tip_fee）
pub async fn build_tip_transaction_with_priority_fee(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    tip_account: &Pubkey,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    data_size_limit: u32,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    build_tip_transaction(
        payer,
        priority_fee,
        business_instructions,
        tip_account,
        priority_fee.buy_tip_fee,
        lookup_table_key,
        recent_blockhash,
        data_size_limit,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}

/// 构建标准的RPC交易
pub async fn build_sell_transaction(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    let mut instructions = vec![];

    // 添加计算预算指令
    add_sell_compute_budget_instructions(&mut instructions, priority_fee);

    // 添加业务指令
    instructions.extend(business_instructions);

    // 获取地址查找表账户
    let address_lookup_table_accounts = get_address_lookup_table_accounts(lookup_table_key).await;

    // 构建交易
    build_versioned_transaction(
        payer,
        instructions,
        address_lookup_table_accounts,
        recent_blockhash,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}

pub async fn build_sell_tip_transaction(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    tip_account: &Pubkey,
    tip_amount: f64,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    let mut instructions = vec![];

    // 添加计算预算指令
    add_sell_tip_compute_budget_instructions(&mut instructions, priority_fee);

    // 添加业务指令
    instructions.extend(business_instructions);

    // 添加小费转账指令
    instructions.push(transfer(
        &payer.pubkey(),
        tip_account,
        sol_str_to_lamports(tip_amount.to_string().as_str()).unwrap_or(0),
    ));

    // 获取地址查找表账户
    let address_lookup_table_accounts = get_address_lookup_table_accounts(lookup_table_key).await;

    // 构建交易
    build_versioned_transaction(
        payer,
        instructions,
        address_lookup_table_accounts,
        recent_blockhash,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}

pub async fn build_sell_tip_transaction_with_priority_fee(
    payer: Arc<Keypair>,
    priority_fee: &PriorityFee,
    business_instructions: Vec<Instruction>,
    tip_account: &Pubkey,
    lookup_table_key: Option<Pubkey>,
    recent_blockhash: Hash,
    middleware_manager: Option<Arc<MiddlewareManager>>,
    protocol_name: String,
    is_buy: bool,
) -> Result<VersionedTransaction, anyhow::Error> {
    build_sell_tip_transaction(
        payer,
        priority_fee,
        business_instructions,
        tip_account,
        priority_fee.sell_tip_fee,
        lookup_table_key,
        recent_blockhash,
        middleware_manager,
        protocol_name,
        is_buy,
    )
    .await
}
