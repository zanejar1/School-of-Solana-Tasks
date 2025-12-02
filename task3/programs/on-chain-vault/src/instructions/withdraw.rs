//-------------------------------------------------------------------------------
///
/// TASK: Implement the withdraw functionality for the on-chain vault
/// 
/// Requirements:
/// - Verify that the vault is not locked
/// - Verify that the vault has enough balance to withdraw
/// - Transfer lamports from vault to vault authority
/// - Emit a withdraw event after successful transfer
/// 
///-------------------------------------------------------------------------------

use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::errors::VaultError;
use crate::events::WithdrawEvent;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The authority that owns the vault
    #[account(mut)]
    pub vault_authority: Signer<'info>,

    /// The vault account holding SOL
    #[account(
        mut,
        seeds = [b"vault", vault.vault_authority.as_ref()],
        bump,
        constraint = vault.vault_authority == vault_authority.key() @ VaultError::VaultLocked,
        constraint = !vault.locked @ VaultError::VaultLocked
    )]
    pub vault: Account<'info, Vault>,

    /// System program (not strictly required if we use direct lamport manipulation, 
    /// but included for consistency)
    pub system_program: Program<'info, System>,
}

pub fn _withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault_info = &ctx.accounts.vault.to_account_info();
    let authority_info = &ctx.accounts.vault_authority.to_account_info();

    // Check that vault has enough lamports
    if **vault_info.lamports.borrow() < amount {
        return Err(VaultError::InsufficientBalance.into());
    }

    // Check vault not locked
    if ctx.accounts.vault.locked {
        return Err(VaultError::VaultLocked.into());
    }

    // Transfer lamports directly (no CPI needed since both are owned by system program)
    **vault_info.try_borrow_mut_lamports()? -= amount;
    **authority_info.try_borrow_mut_lamports()? += amount;

    // Emit event
    emit!(WithdrawEvent {
        amount,
        vault_authority: ctx.accounts.vault_authority.key(),
        vault: ctx.accounts.vault.key(),
    });

    Ok(())
}
