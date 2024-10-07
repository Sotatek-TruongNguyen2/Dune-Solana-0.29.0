use crate::{
    calculate_transfer_fee_excluded_amount,
    constants::{pda_seed::USER_DUNE_INFO_SEED, transfer_memo},
    events::DepositEvent,
    transfer_from_owner_to_vault_v2, transfer_from_vault_to_owner_v2,
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

    #[account(
      init,
      payer = token_authority,
      token::mint = deposit_token_mint,
      token::token_program = token_program_a,
      token::authority = token_authority
    )]
    pub user_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      init,
      payer = token_authority,
      token::mint = reward_token_mint,
      token::token_program = token_program_b,
      token::authority = token_authority
    )]
    pub reward_user_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

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

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Stake<'info>>,
    amount: u64,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<()> {
    let user_info = &mut ctx.accounts.user_info;
    let dunepool = &mut ctx.accounts.dunepool;
    let token_authority = &mut ctx.accounts.token_authority;
    let reward_token_mint = &mut ctx.accounts.reward_token_mint;
    let deposit_token_mint = &mut ctx.accounts.deposit_token_mint;
    let user_token_vault = &mut ctx.accounts.user_token_vault;
    let reward_token_vault = &mut ctx.accounts.reward_token_vault;
    let reward_user_token_vault = &mut ctx.accounts.reward_user_token_vault;
    let memo_program = &mut ctx.accounts.memo_program;

    if !user_info.is_initialized {
        let bump = ctx.bumps.user_info;
        user_info.initialize(&dunepool, bump, token_authority.key())?;
    }

    let deposit_amount = calculate_transfer_fee_excluded_amount(deposit_token_mint, amount)?;

    let earned_rewards = user_info.deposit(dunepool, deposit_amount.amount)?;

    // Process remaining accounts
    let remaining_accounts = parse_remaining_accounts(
        ctx.remaining_accounts,
        &remaining_accounts_info,
        &[AccountsType::TransferHookA, AccountsType::TransferHookB],
    )?;

    transfer_from_owner_to_vault_v2(
        token_authority,
        deposit_token_mint,
        user_token_vault,
        reward_token_vault,
        &ctx.accounts.token_program_a,
        memo_program,
        &remaining_accounts.transfer_hook_input,
        deposit_amount.amount,
    )?;

    if earned_rewards > 0 {
        transfer_from_vault_to_owner_v2(
            dunepool,
            reward_token_mint,
            reward_token_vault,
            reward_user_token_vault,
            &ctx.accounts.token_program_b,
            memo_program,
            &remaining_accounts.transfer_hook_output,
            earned_rewards,
            transfer_memo::TRANSFER_MEMO_DEPOSIT.as_bytes(),
        )?;
    }

    emit!(DepositEvent {
        pool_id: dunepool.key(),
        user: user_info.authority,
        earned_rewards,
        deposit_amount: deposit_amount.amount,
        total_stake: user_info.total_stake
    });

    Ok(())
}
