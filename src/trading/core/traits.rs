use std::sync::Arc;

use anyhow::Result;
use solana_sdk::instruction::Instruction;
use crate::trading::MiddlewareManager;

use super::params::{BuyParams, BuyWithTipParams, SellParams, SellWithTipParams};

/// 交易执行器trait - 定义了所有交易协议都需要实现的核心方法
#[async_trait::async_trait]
pub trait TradeExecutor: Send + Sync {
    /// 执行买入交易
    async fn buy(&self, params: BuyParams, middleware_manager: Option<Arc<MiddlewareManager>>) -> Result<()>;

    /// 使用MEV服务执行买入交易
    async fn buy_with_tip(&self, params: BuyWithTipParams, middleware_manager: Option<Arc<MiddlewareManager>>) -> Result<()>;

    /// 执行卖出交易
    async fn sell(&self, params: SellParams, middleware_manager: Option<Arc<MiddlewareManager>>) -> Result<()>;

    /// 使用MEV服务执行卖出交易
    async fn sell_with_tip(&self, params: SellWithTipParams, middleware_manager: Option<Arc<MiddlewareManager>>) -> Result<()>;

    /// 获取协议名称
    fn protocol_name(&self) -> &'static str;
}

/// 指令构建器trait - 负责构建协议特定的交易指令
#[async_trait::async_trait]
pub trait InstructionBuilder: Send + Sync {
    /// 构建买入指令
    async fn build_buy_instructions(&self, params: &BuyParams) -> Result<Vec<Instruction>>;

    /// 构建卖出指令
    async fn build_sell_instructions(&self, params: &SellParams) -> Result<Vec<Instruction>>;
}

/// 协议特定参数trait - 允许每个协议定义自己的参数
pub trait ProtocolParams: Send + Sync {
    /// 将参数转换为Any以便向下转型
    fn as_any(&self) -> &dyn std::any::Any;

    /// 克隆参数
    fn clone_box(&self) -> Box<dyn ProtocolParams>;
}

impl Clone for Box<dyn ProtocolParams> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
