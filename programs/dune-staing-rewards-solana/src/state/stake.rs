use anchor_lang::prelude::*;

use crate::{
    constants::WAD,
    errors::ErrorCode,
    utils::{transfer_from_vault_to_owner, v2::token::transfer_from_owner_to_vault_v2},
};

use super::DunePool;

#[account]
#[derive(Default)]
pub struct DuneUserInfo {
    pub authority: Pubkey, // 32
    pub bump: u8,          // 1

    pub dunepool: Pubkey, //32

    pub total_claimed: u128, // 16
    pub total_stake: u128,   // 16
    pub last_updated: u64,   // 8

    pub reward_debt: u128,  // 16
    pub total_earned: u128, // 16

    pub blacklisted: bool,    // 1
    pub is_initialized: bool, // 1
}

impl DuneUserInfo {
    pub const LEN: usize = 8 + 32 + 1 + 32 + 16 + 16 + 8 + 16 + 16 + 1 + 1;

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

    pub fn deposit(&mut self, dunepool: &mut Account<DunePool>, amount: u128) -> Result<()> {
        self.enforce_initialized();

        if amount == 0 {
            return Err(ErrorCode::InvalidDepositAmount.into());
        }

        // We need to update the pool before doing anything further
        dunepool.update();

        if self.total_stake > 0 {
            let pending = self.total_stake * dunepool.acc_reward_per_share / WAD - self.reward_debt;
            self.total_earned += pending;
        }

        dunepool.update_total_stake(amount);

        let clock = Clock::get()?;
        let current_block = clock.slot;

        self.total_stake += amount;

        self.reward_debt = self.total_stake * dunepool.acc_reward_per_share / WAD;
        self.last_updated = current_block;

        Ok(())
    }

    fn enforce_initialized(&self) -> Result<()> {
        if !self.is_initialized {
            return Err(ErrorCode::AccountIsInitialized.into());
        }

        Ok(())
    }
}
