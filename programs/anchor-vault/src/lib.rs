#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

declare_id!("2GKEFVyeUV7v2PN9NWSJp6aE4RnowhQS6sr5y4ranuMw");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;

        emit!(InitializeEvent{
            user: ctx.accounts.user.key(),
        });

        Ok(())
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;

        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;

        emit!(WithdrawEvent {
            user: ctx.accounts.user.key(),
            amount
        });

        Ok(())
    }

    pub fn withdraw_and_close(ctx: Context<CloseVault>) -> Result<()> {
        let amount: u64 = ctx.accounts.vault.lamports();

        ctx.accounts.withdraw_and_close()?;

        emit!(WithdrawAndCloseEvent {
            user: ctx.accounts.user.key(),
            amount
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = VaultState::INIT_SPACE,
        seeds = [b"state", user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>
}

impl<'info> CloseVault<'info> {
    pub fn withdraw_and_close(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts: Transfer<'_> = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds: &[&[u8]; 3] = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let cpi_ctx: CpiContext<'_, '_, '_, '_, Transfer<'_>> = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, self.vault.lamports())
    }
}

impl<'info> Payment<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts: Transfer<'_> = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx: CpiContext<'_, '_, '_, '_, Transfer<'_>> = CpiContext::new(cpi_program, cpi_accounts);
        
        transfer(cpi_ctx, amount)
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault_balance: u64 = self.vault.lamports();
        let rent_exempt: u64 = Rent::get()?.minimum_balance(0);

        require!(vault_balance >= amount, VaultError::InsufficientBalance);
        require!(vault_balance - rent_exempt >= amount, VaultError:: InsufficientBalanceForRent);

        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts: Transfer<'_> = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds: &[&[u8]; 3] = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let cpi_ctx: CpiContext<'_, '_, '_, '_, Transfer<'_>> = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        
        transfer(cpi_ctx, amount)
    }
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {

        let rent_exempt: u64 = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts: Transfer<'_> = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx: CpiContext<'_, '_, '_, '_, Transfer<'_>> = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;

        Ok(())
    }
}

#[account]
pub struct VaultState{
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState{
    const INIT_SPACE: usize = 8 + 1 + 1;
}

#[event]
pub struct InitializeEvent {
    pub user: Pubkey
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64
}

#[event]
pub struct WithdrawAndCloseEvent {
    pub user: Pubkey,
    pub amount: u64
}

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient balance in vault")]
    InsufficientBalance,
    #[msg("Withdrawal would leave vault below rent-exempt threshold")]
    InsufficientBalanceForRent,
}