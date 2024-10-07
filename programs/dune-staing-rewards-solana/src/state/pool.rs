use anchor_lang::prelude::*;

use crate::{
    constants::{DUNE_POOL_SEED, WAD},
    utils::to_timestamp_u64,
};

use super::config::DunePoolConfig;

#[account]
#[derive(Default)]
pub struct DunePool {
    pub pool_config: Pubkey, // 32
    pub bump: u8,            // 1

    pub deposit_token_mint: Pubkey,  // 32
    pub deposit_token_vault: Pubkey, // 32

    pub reward_token_mint: Pubkey,  // 32
    pub reward_token_vault: Pubkey, // 32

    pub paused: bool, // 1

    // == Staking details == //
    pub reward_per_block: u128,     // 16
    pub total_stake: u128,          // 16
    pub acc_reward_per_share: u128, // 16
    pub last_updated: u64,          // 8
}

impl DunePool {
    pub const LEN: usize = 8 + 32 + 1 + 32 * 4 + 1 + 16 + 16 + 16 + 8;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        dunepool_config: &Account<DunePoolConfig>,
        bump: u8,                    //
        deposit_token_mint: Pubkey,  // 32
        deposit_token_vault: Pubkey, // 32
        reward_token_mint: Pubkey,   // 32
        reward_token_vault: Pubkey,  // 32
    ) -> Result<()> {
        self.pool_config = dunepool_config.key();
        self.deposit_token_vault = deposit_token_vault;
        self.deposit_token_mint = deposit_token_mint;

        self.reward_token_mint = reward_token_mint;
        self.reward_token_vault = reward_token_vault;

        self.bump = bump;

        let clock = Clock::get()?;
        self.last_updated = to_timestamp_u64(clock.unix_timestamp)?;

        Ok(())
    }

    pub fn update_total_stake(&mut self, deposit_amount: u128) -> Result<()> {
        self.total_stake += deposit_amount;
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        let current_block = clock.slot;

        // In case the current timetsamp <= last_updated, there's no update
        if current_block <= self.last_updated {
            return Ok(());
        }

        let total_spawning_rewards = self.spawning_rewards(self.last_updated, current_block)?;

        self.acc_reward_per_share += total_spawning_rewards * WAD / self.total_stake;
        self.last_updated = current_block;

        Ok(())
    }

    pub fn spawning_rewards(&self, from: u64, to: u64) -> Result<u128> {
        let total_blocks = from - to;
        let total_spawning_rewards = self.reward_per_block * total_blocks as u128;
        Ok(total_spawning_rewards)
    }

    pub fn seeds(&self) -> [&[u8]; 4] {
        [
            DUNE_POOL_SEED,
            self.pool_config.as_ref(),
            self.deposit_token_mint.as_ref(),
            self.reward_token_mint.as_ref(),
        ]
    }
}
