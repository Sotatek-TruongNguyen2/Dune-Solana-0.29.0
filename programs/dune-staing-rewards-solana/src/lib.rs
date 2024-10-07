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

#[doc(hidden)]
pub mod events;

use instructions::*;
use utils::*;

#[program]
pub mod dune_staing_rewards_solana {
    use super::*;

    pub fn add_pool(ctx: Context<AddPool>) -> Result<()> {
        add_pool::handler(ctx)
    }

    pub fn stake<'info>(
        ctx: Context<'_, '_, '_, 'info, Stake<'info>>,
        amount: u64,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        stake::handler(ctx, amount, remaining_accounts_info)
    }
}
