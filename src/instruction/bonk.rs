use anyhow::{anyhow, Result};
use solana_sdk::{instruction::Instruction, signer::Signer};
use solana_system_interface::instruction::transfer;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_token::instruction::close_account;

use crate::{
    constants::bonk::{accounts, BUY_EXECT_IN_DISCRIMINATOR, SELL_EXECT_IN_DISCRIMINATOR},
    constants::trade::trade::DEFAULT_SLIPPAGE,
    trading::bonk::{
        common::{get_amount_out, get_pool_pda, get_vault_pda},
        pool::Pool,
    },
    trading::common::utils::get_token_balance,
    trading::core::{
        params::{BonkParams, BuyParams, SellParams},
        traits::InstructionBuilder,
    },
};

/// Bonk协议的指令构建器
pub struct BonkInstructionBuilder;

#[async_trait::async_trait]
impl InstructionBuilder for BonkInstructionBuilder {
    async fn build_buy_instructions(&self, params: &BuyParams) -> Result<Vec<Instruction>> {
        if params.sol_amount == 0 {
            return Err(anyhow!("Amount cannot be zero"));
        }
        self.build_buy_instructions_with_accounts(params).await
    }

    async fn build_sell_instructions(&self, params: &SellParams) -> Result<Vec<Instruction>> {
        self.build_sell_instructions_with_accounts(params).await
    }
}

impl BonkInstructionBuilder {
    /// 使用提供的账户信息构建买入指令
    async fn build_buy_instructions_with_accounts(
        &self,
        params: &BuyParams,
    ) -> Result<Vec<Instruction>> {
        let protocol_params = params
            .protocol_params
            .as_any()
            .downcast_ref::<BonkParams>()
            .ok_or_else(|| anyhow!("Invalid protocol params for Bonk"))?;

        let pool_state = get_pool_pda(&params.mint, &accounts::WSOL_TOKEN_ACCOUNT).unwrap();

        // 创建用户代币账户
        let user_base_token_account = spl_associated_token_account::get_associated_token_address(
            &params.payer.pubkey(),
            &params.mint,
        );
        let user_quote_token_account = spl_associated_token_account::get_associated_token_address(
            &params.payer.pubkey(),
            &accounts::WSOL_TOKEN_ACCOUNT,
        );

        // 获取池的代币账户
        let base_vault_account = get_vault_pda(&pool_state, &params.mint).unwrap();
        let quote_vault_account =
            get_vault_pda(&pool_state, &accounts::WSOL_TOKEN_ACCOUNT).unwrap();

        let mut virtual_base = protocol_params.virtual_base.unwrap_or(0);
        let mut virtual_quote = protocol_params.virtual_quote.unwrap_or(0);
        let mut real_base = protocol_params.real_base.unwrap_or(0);
        let mut real_quote = protocol_params.real_quote.unwrap_or(0);

        if virtual_base == 0 || virtual_quote == 0 || real_base == 0 || real_quote == 0 {
            let pool = Pool::fetch(params.rpc.as_ref().unwrap(), &pool_state).await?;
            virtual_base = pool.virtual_base as u128;
            virtual_quote = pool.virtual_quote as u128;
            real_base = pool.real_base as u128;
            real_quote = pool.real_quote as u128;
        }

        let amount_in: u64 = params.sol_amount;
        let share_fee_rate: u64 = 0;
        let minimum_amount_out: u64 = get_amount_out(
            amount_in,
            accounts::PROTOCOL_FEE_RATE,
            accounts::PLATFORM_FEE_RATE,
            accounts::SHARE_FEE_RATE,
            virtual_base,
            virtual_quote,
            real_base,
            real_quote,
            params.slippage_basis_points.unwrap_or(DEFAULT_SLIPPAGE) as u128,
        );

        let mut instructions = vec![];

        if protocol_params.auto_handle_wsol {
            // 插入wsol
            instructions.push(
                // 创建wSOL ATA账户，如果不存在
                create_associated_token_account_idempotent(
                    &params.payer.pubkey(),
                    &params.payer.pubkey(),
                    &accounts::WSOL_TOKEN_ACCOUNT,
                    &accounts::TOKEN_PROGRAM,
                ),
            );
            instructions.push(
                // 将SOL转入wSOL ATA账户
                transfer(&params.payer.pubkey(), &user_quote_token_account, amount_in),
            );

            // 同步wSOL余额
            instructions.push(
                spl_token::instruction::sync_native(
                    &accounts::TOKEN_PROGRAM,
                    &user_quote_token_account,
                )
                .unwrap(),
            );
        }

        // 创建用户的基础代币账户
        instructions.push(create_associated_token_account_idempotent(
            &params.payer.pubkey(),
            &params.payer.pubkey(),
            &params.mint,
            &accounts::TOKEN_PROGRAM,
        ));

        // 创建买入指令
        let accounts = vec![
            solana_sdk::instruction::AccountMeta::new(params.payer.pubkey(), true), // Payer (signer)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::AUTHORITY, false), // Authority (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::GLOBAL_CONFIG, false), // Global Config (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::PLATFORM_CONFIG, false), // Platform Config (readonly)
            solana_sdk::instruction::AccountMeta::new(pool_state, false), // Pool State
            solana_sdk::instruction::AccountMeta::new(user_base_token_account, false), // User Base Token
            solana_sdk::instruction::AccountMeta::new(user_quote_token_account, false), // User Quote Token
            solana_sdk::instruction::AccountMeta::new(base_vault_account, false), // Base Vault
            solana_sdk::instruction::AccountMeta::new(quote_vault_account, false), // Quote Vault
            solana_sdk::instruction::AccountMeta::new_readonly(params.mint, false), // Base Token Mint (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::WSOL_TOKEN_ACCOUNT, false), // Quote Token Mint (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::TOKEN_PROGRAM, false), // Base Token Program (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::TOKEN_PROGRAM, false), // Quote Token Program (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::EVENT_AUTHORITY, false), // Event Authority (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::BONK, false), // Program (readonly)
        ];
        // 创建指令数据
        let mut data = vec![];
        data.extend_from_slice(&BUY_EXECT_IN_DISCRIMINATOR);
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&minimum_amount_out.to_le_bytes());
        data.extend_from_slice(&share_fee_rate.to_le_bytes());

        instructions.push(Instruction { program_id: accounts::BONK, accounts, data });

        if protocol_params.auto_handle_wsol {
            // 关闭wSOL ATA账户，回收租金
            instructions.push(
                spl_token::instruction::close_account(
                    &accounts::TOKEN_PROGRAM,
                    &user_quote_token_account,
                    &params.payer.pubkey(),
                    &params.payer.pubkey(),
                    &[],
                )
                .unwrap(),
            );
        }

        Ok(instructions)
    }

    /// 使用提供的账户信息构建卖出指令
    async fn build_sell_instructions_with_accounts(
        &self,
        params: &SellParams,
    ) -> Result<Vec<Instruction>> {
        if params.rpc.is_none() {
            return Err(anyhow!("RPC is not set"));
        }
        let rpc = params.rpc.as_ref().unwrap().clone();

        // 获取代币余额
        let mut amount = params.token_amount;
        if params.token_amount.is_none() || params.token_amount.unwrap_or(0) == 0 {
            let balance_u64 =
                get_token_balance(rpc.as_ref(), &params.payer.pubkey(), &params.mint).await?;
            amount = Some(balance_u64);
        }
        let amount = amount.unwrap_or(0);

        if amount == 0 {
            return Err(anyhow!("Amount cannot be zero"));
        }

        // 计算预期的SOL数量
        let minimum_amount_out: u64 = 1;

        let pool_state = get_pool_pda(&params.mint, &accounts::WSOL_TOKEN_ACCOUNT).unwrap();

        // 创建用户代币账户
        let user_base_token_account = spl_associated_token_account::get_associated_token_address(
            &params.payer.pubkey(),
            &params.mint,
        );
        let user_quote_token_account = spl_associated_token_account::get_associated_token_address(
            &params.payer.pubkey(),
            &accounts::WSOL_TOKEN_ACCOUNT,
        );

        // 获取池的代币账户
        let base_vault_account = get_vault_pda(&pool_state, &params.mint).unwrap();
        let quote_vault_account =
            get_vault_pda(&pool_state, &accounts::WSOL_TOKEN_ACCOUNT).unwrap();

        let share_fee_rate: u64 = 0;

        let mut instructions = vec![];

        // 插入wsol
        instructions.push(
            // 创建wSOL ATA账户，如果不存在
            create_associated_token_account_idempotent(
                &params.payer.pubkey(),
                &params.payer.pubkey(),
                &accounts::WSOL_TOKEN_ACCOUNT,
                &accounts::TOKEN_PROGRAM,
            ),
        );

        // 创建卖出指令
        let accounts = vec![
            solana_sdk::instruction::AccountMeta::new(params.payer.pubkey(), true), // Payer (signer)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::AUTHORITY, false), // Authority (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::GLOBAL_CONFIG, false), // Global Config (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::PLATFORM_CONFIG, false), // Platform Config (readonly)
            solana_sdk::instruction::AccountMeta::new(pool_state, false), // Pool State
            solana_sdk::instruction::AccountMeta::new(user_base_token_account, false), // User Base Token
            solana_sdk::instruction::AccountMeta::new(user_quote_token_account, false), // User Quote Token
            solana_sdk::instruction::AccountMeta::new(base_vault_account, false), // Base Vault
            solana_sdk::instruction::AccountMeta::new(quote_vault_account, false), // Quote Vault
            solana_sdk::instruction::AccountMeta::new_readonly(params.mint, false), // Base Token Mint (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::WSOL_TOKEN_ACCOUNT, false), // Quote Token Mint (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::TOKEN_PROGRAM, false), // Base Token Program (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::TOKEN_PROGRAM, false), // Quote Token Program (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::EVENT_AUTHORITY, false), // Event Authority (readonly)
            solana_sdk::instruction::AccountMeta::new_readonly(accounts::BONK, false), // Program (readonly)
        ];

        // 创建指令数据
        let mut data = vec![];
        data.extend_from_slice(&SELL_EXECT_IN_DISCRIMINATOR);
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&minimum_amount_out.to_le_bytes());
        data.extend_from_slice(&share_fee_rate.to_le_bytes());

        instructions.push(Instruction { program_id: accounts::BONK, accounts, data });

        let protocol_params = params
            .protocol_params
            .as_any()
            .downcast_ref::<BonkParams>()
            .ok_or_else(|| anyhow!("Invalid protocol params for Bonk"))?;

        if protocol_params.auto_handle_wsol {
            instructions.push(
                close_account(
                    &accounts::TOKEN_PROGRAM,
                    &user_quote_token_account,
                    &params.payer.pubkey(),
                    &params.payer.pubkey(),
                    &[&params.payer.pubkey()],
                )
                .unwrap(),
            );
        }

        Ok(instructions)
    }
}
