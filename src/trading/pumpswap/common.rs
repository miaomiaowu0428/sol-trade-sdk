use crate::common::SolanaRpcClient;
use crate::constants::pumpswap::accounts;
use anyhow::anyhow;
use solana_account_decoder::UiAccountEncoding;
use solana_sdk::pubkey::Pubkey;
use solana_streamer_sdk::streaming::event_parser::protocols::pumpswap::types::{pool_decode, Pool};

// Find a pool for a specific mint
pub async fn find_pool(rpc: &SolanaRpcClient, mint: &Pubkey) -> Result<Pubkey, anyhow::Error> {
    let (pool_address, _) = find_by_mint(rpc, mint).await?;
    Ok(pool_address)
}

pub(crate) fn coin_creator_vault_authority(coin_creator: Pubkey) -> Pubkey {
    let (pump_pool_authority, _) = Pubkey::find_program_address(
        &[b"creator_vault", &coin_creator.to_bytes()],
        &crate::constants::pumpswap::accounts::AMM_PROGRAM,
    );
    pump_pool_authority
}

pub(crate) fn coin_creator_vault_ata(coin_creator: Pubkey, quote_mint: Pubkey) -> Pubkey {
    let creator_vault_authority = coin_creator_vault_authority(coin_creator);
    let associated_token_creator_vault_authority =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &creator_vault_authority,
            &quote_mint,
            &crate::constants::pumpswap::accounts::TOKEN_PROGRAM,
        );
    associated_token_creator_vault_authority
}

pub(crate) fn fee_recipient_ata(fee_recipient: Pubkey, quote_mint: Pubkey) -> Pubkey {
    let associated_token_fee_recipient =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &fee_recipient,
            &quote_mint,
            &crate::constants::pumpswap::accounts::TOKEN_PROGRAM,
        );
    associated_token_fee_recipient
}

pub fn get_user_volume_accumulator_pda(user: &Pubkey) -> Option<Pubkey> {
    let seeds: &[&[u8]; 2] =
        &[&crate::constants::pumpswap::seeds::USER_VOLUME_ACCUMULATOR_SEED, user.as_ref()];
    let program_id: &Pubkey = &&crate::constants::pumpswap::accounts::AMM_PROGRAM;
    let pda: Option<(Pubkey, u8)> = Pubkey::try_find_program_address(seeds, program_id);
    pda.map(|pubkey| pubkey.0)
}

pub fn get_global_volume_accumulator_pda() -> Option<Pubkey> {
    let seeds: &[&[u8]; 1] = &[&crate::constants::pumpswap::seeds::GLOBAL_VOLUME_ACCUMULATOR_SEED];
    let program_id: &Pubkey = &&crate::constants::pumpswap::accounts::AMM_PROGRAM;
    let pda: Option<(Pubkey, u8)> = Pubkey::try_find_program_address(seeds, program_id);
    pda.map(|pubkey| pubkey.0)
}

pub async fn fetch_pool(
    rpc: &SolanaRpcClient,
    pool_address: &Pubkey,
) -> Result<Pool, anyhow::Error> {
    let account = rpc.get_account(pool_address).await?;
    if account.owner != accounts::AMM_PROGRAM {
        return Err(anyhow!("Account is not owned by PumpSwap program"));
    }
    let pool = pool_decode(&account.data[8..]).ok_or_else(|| anyhow!("Failed to decode pool"))?;
    Ok(pool)
}

pub async fn find_by_base_mint(
    rpc: &SolanaRpcClient,
    base_mint: &Pubkey,
) -> Result<(Pubkey, Pool), anyhow::Error> {
    // 使用getProgramAccounts查找给定mint的池子
    let filters = vec![
        // solana_rpc_client_api::filter::RpcFilterType::DataSize(211), // Pool账户的大小
        solana_rpc_client_api::filter::RpcFilterType::Memcmp(
            solana_client::rpc_filter::Memcmp::new_base58_encoded(43, &base_mint.to_bytes()),
        ),
    ];
    let config = solana_rpc_client_api::config::RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: solana_rpc_client_api::config::RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: None,
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let program_id = crate::constants::pumpswap::accounts::AMM_PROGRAM;
    let accounts = rpc.get_program_accounts_with_config(&program_id, config).await?;
    if accounts.is_empty() {
        return Err(anyhow!("No pool found for mint {}", base_mint));
    }
    let mut pools: Vec<_> = accounts
        .into_iter()
        .filter_map(|(addr, acc)| pool_decode(&acc.data).map(|pool| (addr, pool)))
        .collect();
    pools.sort_by(|a, b| b.1.lp_supply.cmp(&a.1.lp_supply));
    let (address, pool) = pools[0].clone();
    Ok((address, pool))
}

pub async fn find_by_quote_mint(
    rpc: &SolanaRpcClient,
    quote_mint: &Pubkey,
) -> Result<(Pubkey, Pool), anyhow::Error> {
    // 使用getProgramAccounts查找给定mint的池子
    let filters = vec![
        // solana_rpc_client_api::filter::RpcFilterType::DataSize(211), // Pool账户的大小
        solana_rpc_client_api::filter::RpcFilterType::Memcmp(
            solana_client::rpc_filter::Memcmp::new_base58_encoded(75, &quote_mint.to_bytes()),
        ),
    ];
    let config = solana_rpc_client_api::config::RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: solana_rpc_client_api::config::RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: None,
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let program_id = crate::constants::pumpswap::accounts::AMM_PROGRAM;
    let accounts = rpc.get_program_accounts_with_config(&program_id, config).await?;
    if accounts.is_empty() {
        return Err(anyhow!("No pool found for mint {}", quote_mint));
    }
    let mut pools: Vec<_> = accounts
        .into_iter()
        .filter_map(|(addr, acc)| pool_decode(&acc.data).map(|pool| (addr, pool)))
        .collect();
    pools.sort_by(|a, b| b.1.lp_supply.cmp(&a.1.lp_supply));
    let (address, pool) = pools[0].clone();
    Ok((address, pool))
}

pub async fn find_by_mint(
    rpc: &SolanaRpcClient,
    mint: &Pubkey,
) -> Result<(Pubkey, Pool), anyhow::Error> {
    if let Ok((address, pool)) = find_by_base_mint(rpc, mint).await {
        return Ok((address, pool));
    }
    if let Ok((address, pool)) = find_by_quote_mint(rpc, mint).await {
        return Ok((address, pool));
    }
    Err(anyhow!("No pool found for mint {}", mint))
}

pub async fn get_token_balances(
    pool: &Pool,
    rpc: &SolanaRpcClient,
) -> Result<(u64, u64), anyhow::Error> {
    let base_balance = rpc.get_token_account_balance(&pool.pool_base_token_account).await?;
    let quote_balance = rpc.get_token_account_balance(&pool.pool_quote_token_account).await?;

    let base_amount = base_balance.amount.parse::<u64>().map_err(|e| anyhow!(e))?;
    let quote_amount = quote_balance.amount.parse::<u64>().map_err(|e| anyhow!(e))?;

    Ok((base_amount, quote_amount))
}
