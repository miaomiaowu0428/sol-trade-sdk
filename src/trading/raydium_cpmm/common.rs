use crate::{
    common::SolanaRpcClient,
    constants::{
        self,
        raydium_cpmm::accounts::{self},
    },
};
use anyhow::anyhow;
use solana_sdk::pubkey::Pubkey;
use solana_streamer_sdk::streaming::event_parser::protocols::raydium_cpmm::types::{
    pool_state_decode, PoolState,
};

pub async fn fetch_pool_state(
    rpc: &SolanaRpcClient,
    pool_address: &Pubkey,
) -> Result<PoolState, anyhow::Error> {
    let account = rpc.get_account(pool_address).await?;
    if account.owner != accounts::RAYDIUM_CPMM {
        return Err(anyhow!("Account is not owned by Raydium Cpmm program"));
    }
    let pool_state = pool_state_decode(&account.data[8..])
        .ok_or_else(|| anyhow!("Failed to decode pool state"))?;
    Ok(pool_state)
}

pub fn get_pool_pda(amm_config: &Pubkey, mint1: &Pubkey, mint2: &Pubkey) -> Option<Pubkey> {
    let seeds: &[&[u8]; 4] = &[
        constants::raydium_cpmm::seeds::POOL_SEED,
        amm_config.as_ref(),
        mint1.as_ref(),
        mint2.as_ref(),
    ];
    let program_id: &Pubkey = &constants::raydium_cpmm::accounts::RAYDIUM_CPMM;
    let pda: Option<(Pubkey, u8)> = Pubkey::try_find_program_address(seeds, program_id);
    pda.map(|pubkey| pubkey.0)
}

pub fn get_vault_pda(pool_state: &Pubkey, mint: &Pubkey) -> Option<Pubkey> {
    let seeds: &[&[u8]; 3] =
        &[constants::raydium_cpmm::seeds::POOL_VAULT_SEED, pool_state.as_ref(), mint.as_ref()];
    let program_id: &Pubkey = &constants::raydium_cpmm::accounts::RAYDIUM_CPMM;
    let pda: Option<(Pubkey, u8)> = Pubkey::try_find_program_address(seeds, program_id);
    pda.map(|pubkey| pubkey.0)
}

pub fn get_observation_state_pda(pool_state: &Pubkey) -> Option<Pubkey> {
    let seeds: &[&[u8]; 2] =
        &[constants::raydium_cpmm::seeds::OBSERVATION_STATE_SEED, pool_state.as_ref()];
    let program_id: &Pubkey = &constants::raydium_cpmm::accounts::RAYDIUM_CPMM;
    let pda: Option<(Pubkey, u8)> = Pubkey::try_find_program_address(seeds, program_id);
    pda.map(|pubkey| pubkey.0)
}

/// 获取池子中两个代币的余额
///
/// # 返回值
/// 返回 token0_balance, token1_balance
pub async fn get_pool_token_balances(
    rpc: &SolanaRpcClient,
    pool_state: &Pubkey,
    token0_mint: &Pubkey,
    token1_mint: &Pubkey,
) -> Result<(u64, u64), anyhow::Error> {
    let token0_vault = get_vault_pda(pool_state, token0_mint).unwrap();
    let token0_balance = rpc.get_token_account_balance(&token0_vault).await?;
    let token1_vault = get_vault_pda(pool_state, token1_mint).unwrap();
    let token1_balance = rpc.get_token_account_balance(&token1_vault).await?;

    // 解析余额字符串为 u64
    let token0_amount =
        token0_balance.amount.parse::<u64>().map_err(|e| anyhow!("解析 token0 余额失败: {}", e))?;

    let token1_amount =
        token1_balance.amount.parse::<u64>().map_err(|e| anyhow!("解析 token1 余额失败: {}", e))?;

    Ok((token0_amount, token1_amount))
}

/// 计算代币价格 (token1/token0)
///
/// # 返回值
/// 返回 token1 相对于 token0 的价格
pub async fn calculate_price(
    token0_amount: u64,
    token1_amount: u64,
    mint0_decimals: u8,
    mint1_decimals: u8,
) -> Result<f64, anyhow::Error> {
    if token0_amount == 0 {
        return Err(anyhow!("Token0 余额为零，无法计算价格"));
    }
    // 考虑小数位精度
    let token0_adjusted = token0_amount as f64 / 10_f64.powi(mint0_decimals as i32);
    let token1_adjusted = token1_amount as f64 / 10_f64.powi(mint1_decimals as i32);
    let price = token1_adjusted / token0_adjusted;
    Ok(price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey;

    #[test]
    fn test_get_pool_pda() {
        // 测试get_pool_pda函数
        let amm_config = constants::raydium_cpmm::accounts::AMM_CONFIG;
        let input_mint = pubkey!("So11111111111111111111111111111111111111112"); // WSOL
        let output_mint = pubkey!("BnwbwoqPm5ZNx7YTJ8g9jR2qCpYeHBC7xxpU8zEtbonk"); // USDC
        let pool_state = pubkey!("E9rRRpcdsKAseeLFbwC1Ewxd3aYG27meqwTTrMfCTbSG");
        let result = get_pool_pda(&amm_config, &input_mint, &output_mint);
        assert_eq!(result, Some(pool_state));
    }

    #[test]
    fn test_get_vault_pda() {
        // 测试get_vault_pda函数
        let pool_state = pubkey!("HBMkgQvt4NAFx6XzNav23bNcv6K3oiC5UfY3JsE22scY");
        let mint = pubkey!("DeESECsL3cLXno1LFquss98kNQSno1xpQC2ERCqSbonk"); // WSOL
        let vault_pda = pubkey!("7rkgNG3A8z636DuzhchKeqAJTaH3H5ZFWmBQeStydovA");
        let result = get_vault_pda(&pool_state, &mint);
        assert_eq!(result, Some(vault_pda));
    }

    #[test]
    fn test_get_observation_state_pda() {
        let pool_state = pubkey!("HBMkgQvt4NAFx6XzNav23bNcv6K3oiC5UfY3JsE22scY");
        let observation_state_pda = pubkey!("Gq8u9N18ASjq3AK2gCk6RtGSNyjXZf9EZDb6vTtB9JRs");
        let result = get_observation_state_pda(&pool_state);
        assert_eq!(result, Some(observation_state_pda));
    }
}
