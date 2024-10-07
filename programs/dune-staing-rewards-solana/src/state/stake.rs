use crate::{constants::WAD, errors::ErrorCode};
use anchor_lang::prelude::*;

use super::DunePool;

#[account]
#[derive(Default)]
pub struct DuneUserInfo {
    pub authority: Pubkey, // 32
    pub bump: u8,          // 1

    pub dunepool: Pubkey, //32

    pub total_claimed: u128, // 16
    pub total_stake: u64,    // 8
    pub last_updated: u64,   // 8

    pub reward_debt: u64, // 16

    pub blacklisted: bool,    // 1
    pub is_initialized: bool, // 1
}

impl DuneUserInfo {
    pub const LEN: usize = 8 + 32 + 1 + 32 + 16 + 8 + 8 + 16 + 1 + 1;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        dunepool: &Account<DunePool>,
        bump: u8,
        authority: Pubkey,
    ) -> Result<()> {
        if !self.is_initialized {
            self.dunepool = dunepool.key();
            self.authority = authority;
            self.bump = bump;
            self.blacklisted = false;
            self.is_initialized = true;

            return Ok(());
        }

        Err(ErrorCode::AccountIsInitialized.into())
    }

    pub fn deposit(&mut self, dunepool: &mut Account<DunePool>, amount: u64) -> Result<u64> {
        self.enforce_initialized()?;

        if amount == 0 {
            return Err(ErrorCode::InvalidDepositAmount.into());
        }

        // We need to update the pool before doing anything further
        dunepool.update()?;

        let mut pending_reward = 0;

        if self.total_stake > 0 {
            // There're some rewards needs to be paid in here
            pending_reward = (self.total_stake as u128 * dunepool.acc_reward_per_share / WAD)
                as u64
                - self.reward_debt;
        }

        // Update total stake amount for the pool
        dunepool.update_total_stake(amount)?;

        self.total_stake += amount;
        self.update_user_info(dunepool)?;

        Ok(pending_reward)
    }

    fn enforce_initialized(&self) -> Result<()> {
        if !self.is_initialized {
            return Err(ErrorCode::AccountIsInitialized.into());
        }

        Ok(())
    }

    fn update_user_info(&mut self, dunepool: &Account<DunePool>) -> Result<()> {
        let clock = Clock::get()?;
        let current_block = clock.slot;

        self.reward_debt = (self.total_stake as u128 * dunepool.acc_reward_per_share / WAD) as u64;
        self.last_updated = current_block;

        Ok(())
    }
}
