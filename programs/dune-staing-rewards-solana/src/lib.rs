use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[doc(hidden)]
pub mod state;

#[doc(hidden)]
pub mod errors;

#[doc(hidden)]
pub mod constants;

#[doc(hidden)]
pub mod instructions;

#[doc(hidden)]
pub mod utils;

use instructions::*;
use utils::*;

#[program]
pub mod dune_staing_rewards_solana {
    use super::*;

    pub fn add_pool(ctx: Context<AddPool>) -> Result<()> {
        add_pool::handler(ctx)
    }

    pub fn stake(
        ctx: Context<Stake>,
        amount: u128,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        stake::handler(ctx, amount, remaining_accounts_info)
    }
}
