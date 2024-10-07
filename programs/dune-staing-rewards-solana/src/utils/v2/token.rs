use anchor_lang::{prelude::*, solana_program};
use anchor_spl::memo::{self, BuildMemo, Memo};
use anchor_spl::token::Token;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::TransferFee;
use anchor_spl::token_2022::spl_token_2022::extension::{
    BaseStateWithExtensions, StateWithExtensions,
};
use anchor_spl::token_2022::spl_token_2022::{self, extension};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use spl_transfer_hook_interface;

use crate::errors::ErrorCode;
use crate::state::DunePool;

#[derive(Debug)]
pub struct TransferFeeExcludedAmount {
    pub amount: u64,
    pub transfer_fee: u64,
}

pub fn calculate_transfer_fee_excluded_amount(
    token_mint: &InterfaceAccount<'_, Mint>,
    transfer_fee_included_amount: u64,
) -> Result<TransferFeeExcludedAmount> {
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee = epoch_transfer_fee
            .calculate_fee(transfer_fee_included_amount)
            .unwrap();
        let transfer_fee_excluded_amount = transfer_fee_included_amount
            .checked_sub(transfer_fee)
            .unwrap();
        return Ok(TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount,
            transfer_fee,
        });
    }

    Ok(TransferFeeExcludedAmount {
        amount: transfer_fee_included_amount,
        transfer_fee: 0,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn transfer_from_vault_to_owner_v2<'info>(
    dunepool: &Account<'info, DunePool>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_vault: &InterfaceAccount<'info, TokenAccount>,
    token_owner_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    memo_program: &Program<'info, Memo>,
    transfer_hook_accounts: &Option<Vec<AccountInfo<'info>>>,
    amount: u64,
    memo: &[u8],
) -> Result<()> {
    // TransferFee extension
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        // log applied transfer fee
        // - Not must, but important for ease of investigation and replay when problems occur
        // - Use Memo because logs risk being truncated
        let transfer_fee_memo = format!(
            "TFe: {}, {}",
            u16::from(epoch_transfer_fee.transfer_fee_basis_points),
            u64::from(epoch_transfer_fee.maximum_fee),
        );
        memo::build_memo(
            CpiContext::new(memo_program.to_account_info(), BuildMemo {}),
            transfer_fee_memo.as_bytes(),
        )?;
    }

    // MemoTransfer extension
    if is_transfer_memo_required(token_owner_account)? {
        memo::build_memo(
            CpiContext::new(memo_program.to_account_info(), BuildMemo {}),
            memo,
        )?;
    }

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        // vault to owner
        &token_vault.key(),         // from (vault account)
        &token_mint.key(),          // mint
        &token_owner_account.key(), // to (owner account)
        &dunepool.key(),            // authority (pool)
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        token_program.to_account_info(),
        // vault to owner
        token_vault.to_account_info(),         // from (vault account)
        token_mint.to_account_info(),          // mint
        token_owner_account.to_account_info(), // to (owner account)
        dunepool.to_account_info(),            // authority (pool)
    ];

    // TransferHook extension
    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        if transfer_hook_accounts.is_none() {
            return Err(ErrorCode::NoExtraAccountsForTransferHook.into());
        }

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            // vault to owner
            token_vault.to_account_info(), // from (vault account)
            token_mint.to_account_info(),  // mint
            token_owner_account.to_account_info(), // to (owner account)
            dunepool.to_account_info(),    // authority (pool)
            amount,
            transfer_hook_accounts.as_ref().unwrap(),
        )?;
    }

    solana_program::program::invoke_signed(&instruction, &account_infos, &[&dunepool.seeds()])?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn transfer_from_owner_to_vault_v2<'info>(
    authority: &Signer<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_owner_account: &InterfaceAccount<'info, TokenAccount>,
    token_vault: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    memo_program: &Program<'info, Memo>,
    transfer_hook_accounts: &Option<Vec<AccountInfo<'info>>>,
    amount: u64,
) -> Result<()> {
    // TransferFee extension
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        // log applied transfer fee
        // - Not must, but important for ease of investigation and replay when problems occur
        // - Use Memo because logs risk being truncated
        let transfer_fee_memo = format!(
            "TFe: {}, {}",
            u16::from(epoch_transfer_fee.transfer_fee_basis_points),
            u64::from(epoch_transfer_fee.maximum_fee),
        );
        memo::build_memo(
            CpiContext::new(memo_program.to_account_info(), BuildMemo {}),
            transfer_fee_memo.as_bytes(),
        )?;
    }

    // MemoTransfer extension
    // The vault doesn't have MemoTransfer extension, so we don't need to use memo_program here

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,          // owner to vault
        &token_owner_account.key(), // from (owner account)
        &token_mint.key(),          // mint
        &token_vault.key(),         // to (vault account)
        authority.key,              // authority (owner)
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        token_program.to_account_info(),       // owner to vault
        token_owner_account.to_account_info(), // from (owner account)
        token_mint.to_account_info(),          // mint
        token_vault.to_account_info(),         // to (vault account)
        authority.to_account_info(),           // authority (owner)
    ];

    // TransferHook extension
    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        if transfer_hook_accounts.is_none() {
            return Err(ErrorCode::NoExtraAccountsForTransferHook.into());
        }

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            // owner to vault
            token_owner_account.to_account_info(), // from (owner account)
            token_mint.to_account_info(),          // mint
            token_vault.to_account_info(),         // to (vault account)
            authority.to_account_info(),           // authority (owner)
            amount,
            transfer_hook_accounts.as_ref().unwrap(),
        )?;
    }

    solana_program::program::invoke(&instruction, &account_infos)?;

    Ok(())
}

pub fn get_epoch_transfer_fee(
    token_mint: &InterfaceAccount<'_, Mint>,
) -> Result<Option<TransferFee>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    if let Ok(transfer_fee_config) =
        token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
    {
        let epoch = Clock::get()?.epoch;
        return Ok(Some(*transfer_fee_config.get_epoch_fee(epoch)));
    }

    Ok(None)
}

fn get_transfer_hook_program_id(token_mint: &InterfaceAccount<'_, Mint>) -> Result<Option<Pubkey>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    Ok(extension::transfer_hook::get_program_id(
        &token_mint_unpacked,
    ))
}

fn is_transfer_memo_required(token_account: &InterfaceAccount<'_, TokenAccount>) -> Result<bool> {
    let token_account_info = token_account.to_account_info();
    if *token_account_info.owner == Token::id() {
        return Ok(false);
    }

    let token_account_data = token_account_info.try_borrow_data()?;
    let token_account_unpacked =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
    let extension =
        token_account_unpacked.get_extension::<extension::memo_transfer::MemoTransfer>();

    if let Ok(memo_transfer) = extension {
        Ok(memo_transfer.require_incoming_transfer_memos.into())
    } else {
        Ok(false)
    }
}
