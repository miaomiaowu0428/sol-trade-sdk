pub mod bonk;
pub mod pumpfun;
pub mod pumpswap;
pub mod raydium_cpmm;
pub mod swqos;
pub mod trade;
pub mod raydium_amm_v4;
pub mod decimals;

pub mod trade_platform {
    pub const PUMPFUN: &'static str = "pumpfun";
    pub const PUMPFUN_SWAP: &'static str = "pumpswap";
    pub const BONK: &'static str = "bonk";
    pub const RAYDIUM_CPMM: &'static str = "raydium_cpmm";
    pub const RAYDIUM_CLMM: &'static str = "raydium_clmm";
    pub const RAYDIUM_AMM_V4: &'static str = "raydium_amm_v4";
}
