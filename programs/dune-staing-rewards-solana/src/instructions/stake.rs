use crate::{
    constants::pda_seed::USER_DUNE_INFO_SEED,
    utils::remaining_accounts_utils::{
        parse_remaining_accounts, AccountsType, RemainingAccountsInfo,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::state::{pool::DunePool, DuneUserInfo};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(constraint = *(deposit_token_mint.to_account_info().owner) == token_program_a.key())]
    pub token_program_a: Interface<'info, TokenInterface>,
    #[account(constraint = *(reward_token_mint.to_account_info().owner) == token_program_b.key())]
    pub token_program_b: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub token_authority: Signer<'info>,

    #[account(mut)]
    pub dunepool: Box<Account<'info, DunePool>>,

    #[account(init,
      seeds = [
        USER_DUNE_INFO_SEED,
        dunepool.key().as_ref(),
      ],
      bump,
      payer = token_authority,
      space = DuneUserInfo::LEN)
    ]
    pub user_info: Box<Account<'info, DuneUserInfo>>,
    //
    //#[account(
    //  init,
    //  payer = token_authority,
    //  token::mint = deposit_token_mint,
    //  token::token_program = token_program_a,
    //  token::authority = token_authority
    //)]
    //pub user_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = dunepool.deposit_token_vault)]
    pub deposit_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = dunepool.reward_token_vault)]
    pub reward_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = dunepool.deposit_token_mint)]
    pub deposit_token_mint: InterfaceAccount<'info, Mint>,

    #[account(address = dunepool.reward_token_mint)]
    pub reward_token_mint: InterfaceAccount<'info, Mint>,

    pub memo_program: Program<'info, Memo>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    // remaining accounts
    // - accounts for transfer hook program of token_mint_a
    // - accounts for transfer hook program of token_mint_b
}

pub fn handler(
    ctx: Context<Stake>,
    amount: u128,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<()> {
    let user_info = &mut ctx.accounts.user_info;
    let dunepool = &mut ctx.accounts.dunepool;
    let token_authority = &mut ctx.accounts.token_authority;

    if !user_info.is_initialized {
        let bump = ctx.bumps.user_info;
        user_info.initialize(&dunepool, bump, token_authority.key())?;
    }

    // Process remaining accounts
    let remaining_accounts = parse_remaining_accounts(
        ctx.remaining_accounts,
        &remaining_accounts_info,
        &[AccountsType::TransferHookA, AccountsType::TransferHookB],
    )?;

    //transfer_from_owner_to_vault_v2(
    //    authority,
    //    token_owner_account,
    //    token_vault,
    //    token_program,
    //    amount,
    //);
    //
    Ok(user_info.deposit(dunepool, amount)?)
}
