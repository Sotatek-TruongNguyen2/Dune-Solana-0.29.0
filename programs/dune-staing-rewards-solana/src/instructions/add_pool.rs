use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::DUNE_POOL_SEED,
    state::{config::DunePoolConfig, pool::DunePool},
};

#[derive(Accounts)]
pub struct AddPool<'info> {
    pub dunepool_config: Box<Account<'info, DunePoolConfig>>,

    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,
    pub reward_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub guardian: Signer<'info>,

    #[account(init,
      seeds = [
        DUNE_POOL_SEED,
        dunepool_config.key().as_ref(),
        deposit_token_mint.key().as_ref(),
        reward_token_mint.key().as_ref(),
      ],
      bump,
      payer = guardian,
      space = DunePool::LEN)
    ]
    pub dunepool: Box<Account<'info, DunePool>>,

    #[account(init,
      payer = guardian,
      token::token_program = token_program_b,
      token::mint = reward_token_mint,
      token::authority = dunepool)]
    pub reward_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(init,
      payer = guardian,
      token::token_program = token_program_a,
      token::mint = deposit_token_mint,
      token::authority = dunepool)]
    pub deposit_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(constraint = *(deposit_token_mint.to_account_info().owner) == token_program_a.key())]
    pub token_program_a: Interface<'info, TokenInterface>,
    #[account(constraint = *(reward_token_mint.to_account_info().owner) == token_program_b.key())]
    pub token_program_b: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<AddPool>) -> Result<()> {
    let deposit_token_mint = ctx.accounts.deposit_token_mint.key();
    let reward_token_mint = ctx.accounts.reward_token_mint.key();
    let deposit_token_vault = ctx.accounts.deposit_token_vault.key();
    let reward_token_vault = ctx.accounts.reward_token_vault.key();

    let dunepool = &mut ctx.accounts.dunepool;
    let dunepool_config = &ctx.accounts.dunepool_config;

    // ignore the bump passed and use one Anchor derived
    let bump = ctx.bumps.dunepool;

    dunepool.initialize(
        dunepool_config,
        bump,
        deposit_token_mint.key(),
        deposit_token_vault.key(),
        reward_token_mint.key(),
        reward_token_vault.key(),
    )?;

    Ok(())
}
