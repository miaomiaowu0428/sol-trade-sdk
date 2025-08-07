use solana_hash::Hash;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::sync::Arc;

use super::traits::ProtocolParams;
use crate::common::bonding_curve::BondingCurveAccount;
use crate::common::{PriorityFee, SolanaRpcClient};
use crate::constants::bonk::accounts::{PLATFORM_FEE_RATE, PROTOCOL_FEE_RATE, SHARE_FEE_RATE};
use crate::solana_streamer_sdk::streaming::event_parser::common::EventType;
use crate::solana_streamer_sdk::streaming::event_parser::protocols::bonk::BonkTradeEvent;
use crate::swqos::SwqosClient;
use crate::trading::bonk::common::{get_amount_in, get_amount_in_net, get_amount_out};

/// 通用买入参数
#[derive(Clone)]
pub struct BuyParams {
    pub rpc: Option<Arc<SolanaRpcClient>>,
    pub payer: Arc<Keypair>,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub sol_amount: u64,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee: PriorityFee,
    pub lookup_table_key: Option<Pubkey>,
    pub recent_blockhash: Hash,
    pub data_size_limit: u32,
    pub protocol_params: Box<dyn ProtocolParams>,
}

/// 带MEV服务的买入参数
#[derive(Clone)]
pub struct BuyWithTipParams {
    pub rpc: Option<Arc<SolanaRpcClient>>,
    pub swqos_clients: Vec<Arc<SwqosClient>>,
    pub payer: Arc<Keypair>,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub sol_amount: u64,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee: PriorityFee,
    pub lookup_table_key: Option<Pubkey>,
    pub recent_blockhash: Hash,
    pub data_size_limit: u32,
    pub protocol_params: Box<dyn ProtocolParams>,
}

/// 通用卖出参数
#[derive(Clone)]
pub struct SellParams {
    pub rpc: Option<Arc<SolanaRpcClient>>,
    pub payer: Arc<Keypair>,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub token_amount: Option<u64>,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee: PriorityFee,
    pub lookup_table_key: Option<Pubkey>,
    pub recent_blockhash: Hash,
    pub protocol_params: Box<dyn ProtocolParams>,
}

/// 带MEV服务的卖出参数
#[derive(Clone)]
pub struct SellWithTipParams {
    pub rpc: Option<Arc<SolanaRpcClient>>,
    pub swqos_clients: Vec<Arc<SwqosClient>>,
    pub payer: Arc<Keypair>,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub token_amount: Option<u64>,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee: PriorityFee,
    pub lookup_table_key: Option<Pubkey>,
    pub recent_blockhash: Hash,
    pub protocol_params: Box<dyn ProtocolParams>,
}

/// PumpFun协议特定参数
#[derive(Clone)]
pub struct PumpFunParams {
    pub bonding_curve: Option<Arc<BondingCurveAccount>>,
}

impl PumpFunParams {
    pub fn default() -> Self {
        Self {
            bonding_curve: None,
        }
    }
}

impl ProtocolParams for PumpFunParams {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ProtocolParams> {
        Box::new(self.clone())
    }
}

/// PumpSwap Protocol Specific Parameters
///
/// Parameters for configuring PumpSwap trading protocol, including liquidity pool information,
/// token configuration, and transaction amounts.
///
/// **Performance Note**: If these parameters are not provided, the system will attempt to
/// retrieve the relevant information from RPC, which will increase transaction time.
/// For optimal performance, it is recommended to provide all necessary parameters in advance.
#[derive(Clone)]
pub struct PumpSwapParams {
    /// Liquidity pool address
    /// If None, it will be queried via RPC, which adds latency
    pub pool: Option<Pubkey>,

    /// Base token mint address
    /// The mint account address of the base token in the trading pair
    /// If None, it will be queried via RPC, which adds latency
    pub base_mint: Option<Pubkey>,

    /// Quote token mint address
    /// The mint account address of the quote token in the trading pair, usually SOL or USDC
    /// If None, it will be queried via RPC, which adds latency
    pub quote_mint: Option<Pubkey>,

    /// Base token reserves in the pool
    pub pool_base_token_reserves: Option<u64>,

    /// Quote token reserves in the pool
    pub pool_quote_token_reserves: Option<u64>,

    /// Automatically handle WSOL wrapping
    /// When true, automatically handles wrapping and unwrapping operations between SOL and WSOL
    pub auto_handle_wsol: bool,
}

impl PumpSwapParams {
    pub fn default() -> Self {
        Self {
            pool: None,
            base_mint: None,
            quote_mint: None,
            pool_base_token_reserves: None,
            pool_quote_token_reserves: None,
            auto_handle_wsol: true,
        }
    }
}

impl ProtocolParams for PumpSwapParams {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ProtocolParams> {
        Box::new(self.clone())
    }
}

/// Bonk协议特定参数
#[derive(Clone)]
pub struct BonkParams {
    pub virtual_base: Option<u128>,
    pub virtual_quote: Option<u128>,
    pub real_base: Option<u128>,
    pub real_quote: Option<u128>,
    pub auto_handle_wsol: bool,
}

impl BonkParams {
    pub fn default() -> Self {
        Self {
            virtual_base: None,
            virtual_quote: None,
            real_base: None,
            real_quote: None,
            auto_handle_wsol: true,
        }
    }
    pub fn from_trade(trade_info: BonkTradeEvent) -> Self {
        Self {
            virtual_base: Some(trade_info.virtual_base as u128),
            virtual_quote: Some(trade_info.virtual_quote as u128),
            real_base: Some(trade_info.real_base_after as u128),
            real_quote: Some(trade_info.real_quote_after as u128),
            auto_handle_wsol: true,
        }
    }

    pub fn from_dev_trade(trade_info: BonkTradeEvent) -> Self {
        const DEFAULT_VIRTUAL_BASE: u128 = 1073025605596382;
        const DEFAULT_VIRTUAL_QUOTE: u128 = 30000852951;
        let amount_in = if trade_info.metadata.event_type == EventType::BonkBuyExactIn {
            trade_info.amount_in
        } else {
            get_amount_in(
                trade_info.amount_out,
                PROTOCOL_FEE_RATE,
                PLATFORM_FEE_RATE,
                SHARE_FEE_RATE,
                DEFAULT_VIRTUAL_BASE,
                DEFAULT_VIRTUAL_QUOTE,
                0,
                0,
                0,
            )
        };
        let real_quote = get_amount_in_net(
            amount_in,
            PROTOCOL_FEE_RATE,
            PLATFORM_FEE_RATE,
            SHARE_FEE_RATE,
        ) as u128;
        let amount_out = if trade_info.metadata.event_type == EventType::BonkBuyExactIn {
            get_amount_out(
                trade_info.amount_in,
                PROTOCOL_FEE_RATE,
                PLATFORM_FEE_RATE,
                SHARE_FEE_RATE,
                DEFAULT_VIRTUAL_BASE,
                DEFAULT_VIRTUAL_QUOTE,
                0,
                0,
                0,
            ) as u128
        } else {
            trade_info.amount_out as u128
        };
        let real_base = amount_out;
        Self {
            virtual_base: Some(DEFAULT_VIRTUAL_BASE),
            virtual_quote: Some(DEFAULT_VIRTUAL_QUOTE),
            real_base: Some(real_base),
            real_quote: Some(real_quote),
            auto_handle_wsol: true,
        }
    }
}

impl ProtocolParams for BonkParams {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ProtocolParams> {
        Box::new(self.clone())
    }
}

/// RaydiumCpmm协议特定参数
#[derive(Clone)]
pub struct RaydiumCpmmParams {
    /// 池子状态账户地址
    pub pool_state: Option<Pubkey>,
    /// 代币程序ID
    /// 指定代币使用的程序，通常为 spl_token::ID 或 spl_token_2022::ID
    pub mint_token_program: Option<Pubkey>,
    /// 指定 mint_token 在 pool_state 账户数据中的索引位置
    /// 默认值为1，表示在索引1的位置
    pub mint_token_in_pool_state_index: Option<usize>,
    pub minimum_amount_out: Option<u64>,
    pub auto_handle_wsol: bool,
}

impl RaydiumCpmmParams {
    pub fn default() -> Self {
        Self {
            pool_state: None,
            mint_token_program: Some(spl_token::ID),
            mint_token_in_pool_state_index: Some(1),
            minimum_amount_out: None,
            auto_handle_wsol: true,
        }
    }
}

impl ProtocolParams for RaydiumCpmmParams {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ProtocolParams> {
        Box::new(self.clone())
    }
}

impl BuyParams {
    /// 转换为BuyWithTipParams
    pub fn with_tip(self, swqos_clients: Vec<Arc<SwqosClient>>) -> BuyWithTipParams {
        BuyWithTipParams {
            rpc: self.rpc,
            swqos_clients,
            payer: self.payer,
            mint: self.mint,
            creator: self.creator,
            sol_amount: self.sol_amount,
            slippage_basis_points: self.slippage_basis_points,
            priority_fee: self.priority_fee,
            lookup_table_key: self.lookup_table_key,
            recent_blockhash: self.recent_blockhash,
            data_size_limit: self.data_size_limit,
            protocol_params: self.protocol_params,
        }
    }
}

impl SellParams {
    /// 转换为SellWithTipParams
    pub fn with_tip(self, swqos_clients: Vec<Arc<SwqosClient>>) -> SellWithTipParams {
        SellWithTipParams {
            rpc: self.rpc,
            swqos_clients,
            payer: self.payer,
            mint: self.mint,
            creator: self.creator,
            token_amount: self.token_amount,
            slippage_basis_points: self.slippage_basis_points,
            priority_fee: self.priority_fee,
            lookup_table_key: self.lookup_table_key,
            recent_blockhash: self.recent_blockhash,
            protocol_params: self.protocol_params,
        }
    }
}
