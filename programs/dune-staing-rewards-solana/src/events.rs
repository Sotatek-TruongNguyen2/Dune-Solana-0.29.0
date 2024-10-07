use anchor_lang::prelude::*;

/// Emitted when deposit
#[event]
#[cfg_attr(feature = "client", derive(Debug))]
pub struct DepositEvent {
    #[index]
    pub pool_id: Pubkey,
    #[index]
    pub user: Pubkey,

    pub earned_rewards: u64,
    pub deposit_amount: u64,

    pub total_stake: u64,
}
