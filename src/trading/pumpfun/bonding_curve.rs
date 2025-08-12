use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Represents a bonding curve for token pricing and liquidity management
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct PumpfunBondingCurveAccount {
    /// Unique identifier for the bonding curve
    pub discriminator: u64,
    /// Virtual token reserves used for price calculations
    pub virtual_token_reserves: u64,
    /// Virtual SOL reserves used for price calculations
    pub virtual_sol_reserves: u64,
    /// Actual token reserves available for trading
    pub real_token_reserves: u64,
    /// Actual SOL reserves available for trading
    pub real_sol_reserves: u64,
    /// Total supply of tokens
    pub token_total_supply: u64,
    /// Whether the bonding curve is complete/finalized
    pub complete: bool,
    /// Token creator's address
    pub creator: Pubkey,
}

impl PumpfunBondingCurveAccount {
    /// Creates a new bonding curve instance
    ///
    /// # Arguments
    /// * `discriminator` - Unique identifier for the curve
    /// * `virtual_token_reserves` - Virtual token reserves for price calculations
    /// * `virtual_sol_reserves` - Virtual SOL reserves for price calculations
    /// * `real_token_reserves` - Actual token reserves available
    /// * `real_sol_reserves` - Actual SOL reserves available
    /// * `token_total_supply` - Total supply of tokens
    /// * `complete` - Whether the curve is complete
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        discriminator: u64,
        virtual_token_reserves: u64,
        virtual_sol_reserves: u64,
        real_token_reserves: u64,
        real_sol_reserves: u64,
        token_total_supply: u64,
        complete: bool,
        creator: Pubkey,
    ) -> Self {
        Self {
            discriminator,
            virtual_token_reserves,
            virtual_sol_reserves,
            real_token_reserves,
            real_sol_reserves,
            token_total_supply,
            complete,
            creator,
        }
    }

    /// Calculates the amount of tokens received for a given SOL amount
    ///
    /// # Arguments
    /// * `amount` - Amount of SOL to spend
    ///
    /// # Returns
    /// * `Ok(u64)` - Amount of tokens that would be received
    /// * `Err(&str)` - Error message if curve is complete
    pub fn get_buy_price(&self, amount: u64) -> Result<u64, &'static str> {
        if self.complete {
            return Err("Curve is complete");
        }

        if amount == 0 {
            return Ok(0);
        }

        // Calculate the product of virtual reserves using u128 to avoid overflow
        let n: u128 = (self.virtual_sol_reserves as u128) * (self.virtual_token_reserves as u128);

        // Calculate the new virtual sol reserves after the purchase
        let i: u128 = (self.virtual_sol_reserves as u128) + (amount as u128);

        // Calculate the new virtual token reserves after the purchase
        let r: u128 = n / i + 1;

        // Calculate the amount of tokens to be purchased
        let s: u128 = (self.virtual_token_reserves as u128) - r;

        // Convert back to u64 and return the minimum of calculated tokens and real reserves
        let s_u64 = s as u64;
        Ok(if s_u64 < self.real_token_reserves { s_u64 } else { self.real_token_reserves })
    }

    /// Calculates the amount of SOL received for selling tokens
    ///
    /// # Arguments
    /// * `amount` - Amount of tokens to sell
    /// * `fee_basis_points` - Fee in basis points (1/100th of a percent)
    ///
    /// # Returns
    /// * `Ok(u64)` - Amount of SOL that would be received after fees
    /// * `Err(&str)` - Error message if curve is complete
    pub fn get_sell_price(&self, amount: u64, fee_basis_points: u64) -> Result<u64, &'static str> {
        if self.complete {
            return Err("Curve is complete");
        }

        if amount == 0 {
            return Ok(0);
        }

        // Calculate the proportional amount of virtual sol reserves to be received using u128
        let n: u128 = ((amount as u128) * (self.virtual_sol_reserves as u128))
            / ((self.virtual_token_reserves as u128) + (amount as u128));

        // Calculate the fee amount in the same units
        let a: u128 = (n * (fee_basis_points as u128)) / 10000;

        // Return the net amount after deducting the fee, converting back to u64
        Ok((n - a) as u64)
    }

    /// Calculates the current market cap in SOL
    pub fn get_market_cap_sol(&self) -> u64 {
        if self.virtual_token_reserves == 0 {
            return 0;
        }

        ((self.token_total_supply as u128) * (self.virtual_sol_reserves as u128)
            / (self.virtual_token_reserves as u128)) as u64
    }

    /// Calculates the final market cap in SOL after all tokens are sold
    ///
    /// # Arguments
    /// * `fee_basis_points` - Fee in basis points (1/100th of a percent)
    pub fn get_final_market_cap_sol(&self, fee_basis_points: u64) -> u64 {
        let total_sell_value: u128 =
            self.get_buy_out_price(self.real_token_reserves, fee_basis_points) as u128;
        let total_virtual_value: u128 = (self.virtual_sol_reserves as u128) + total_sell_value;
        let total_virtual_tokens: u128 =
            (self.virtual_token_reserves as u128) - (self.real_token_reserves as u128);

        if total_virtual_tokens == 0 {
            return 0;
        }

        ((self.token_total_supply as u128) * total_virtual_value / total_virtual_tokens) as u64
    }

    /// Calculates the price to buy out all remaining tokens
    ///
    /// # Arguments
    /// * `amount` - Amount of tokens to buy
    /// * `fee_basis_points` - Fee in basis points (1/100th of a percent)
    pub fn get_buy_out_price(&self, amount: u64, fee_basis_points: u64) -> u64 {
        // Get the effective amount of sol tokens
        let sol_tokens: u128 = if amount < self.real_sol_reserves {
            self.real_sol_reserves as u128
        } else {
            amount as u128
        };

        // Calculate total sell value
        let total_sell_value: u128 = (sol_tokens * (self.virtual_sol_reserves as u128))
            / ((self.virtual_token_reserves as u128) - sol_tokens)
            + 1;

        // Calculate fee
        let fee: u128 = (total_sell_value * (fee_basis_points as u128)) / 10000;

        // Return total including fee, converting back to u64
        (total_sell_value + fee) as u64
    }
}
