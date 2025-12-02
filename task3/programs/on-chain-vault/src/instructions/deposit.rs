//-------------------------------------------------------------------------------
///
/// TASK: Implement the deposit functionality for the on-chain vault
/// 
/// Requirements:
/// - Verify that the user has enough balance to deposit
/// - Verify that the vault is not locked
/// - Transfer lamports from user to vault using CPI (Cross-Program Invocation)
/// - Emit a deposit event after successful transfer
/// 
///-------------------------------------------------------------------------------

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};
use crate::state::Vault;
use crate::errors::VaultError;
use crate::events::DepositEvent;

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The user depositing SOL into the vault
    #[account(mut)]
    pub user: Signer<'info>,

    /// The vault account to receive SOL
    #[account(
        mut,
        seeds = [b"vault", vault.vault_authority.as_ref()],
        bump,
        constraint = !vault.locked @ VaultError::VaultLocked
    )]
    pub vault: Account<'info, Vault>,

    /// System program for CPI
    pub system_program: Program<'info, System>,
}

pub fn _deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let user_info = &ctx.accounts.user.to_account_info();
    let vault_info = &ctx.accounts.vault.to_account_info();

    // Check user balance
    if **user_info.lamports.borrow() < amount {
        return Err(VaultError::InsufficientBalance.into());
    }

    // Vault lock is already enforced by the constraint above, but double-check for safety
    if ctx.accounts.vault.locked {
        return Err(VaultError::VaultLocked.into());
    }

    // Transfer SOL from user to vault
    let ix = transfer(&user_info.key(), &vault_info.key(), amount);
    invoke(
        &ix,
        &[
            user_info.clone(),
            vault_info.clone(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Emit event
    emit!(DepositEvent {
        amount,
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
    });

    Ok(())
}
