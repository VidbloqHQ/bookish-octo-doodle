use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Transfer, transfer as token_transfer},
    token_interface::{TokenAccount, TokenInterface}
};

use crate::state::{StreamState, StreamError, DonorAccount, StreamType, StreamStatus, DepositMade};

#[derive(Accounts)]
pub struct Deposit <'info> {
    #[account(mut)]
    pub donor: Signer<'info>,

     #[account(
        mut, 
        seeds=[b"stream", stream.stream_name.as_str().as_bytes(), stream.host.key().as_ref()],
        bump=stream.bump
     )]
    pub stream: Account<'info, StreamState>,

    #[account(
        init_if_needed,
        payer = donor,
        space = DonorAccount::INIT_SPACE,
        seeds = [b"donor", stream.key().as_ref(), donor.key().as_ref()],
        bump
    )]
    pub donor_account: Account<'info, DonorAccount>,

    #[account(
        mut,
        constraint = donor_ata.owner == donor.key(),
        constraint = donor_ata.mint == stream.mint
    )]
    pub donor_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = stream_ata.mint == stream.mint,
        constraint = stream_ata.owner == stream.key()
    )]
    pub stream_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> Deposit <'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        require!(amount > 0, StreamError::InvalidAmount);

        match self.stream.stream_type {
            StreamType::Prepaid { .. } => {
                // For prepaid, deposits allowed anytime before start
                require!(
                    self.stream.start_time.is_none(),
                    StreamError::StreamAlreadyStarted
                );
            },
            StreamType::Live => {
                // For live streams, must be active and started
                require!(
                    self.stream.status == StreamStatus::Active && 
                    self.stream.start_time.is_some(),
                    StreamError::DepositNotAllowed
                );
            },
            StreamType::Conditional { .. } => {
                // For conditional, check if stream is active
                require!(
                    self.stream.status == StreamStatus::Active,
                    StreamError::StreamNotActive
                );
            }
        }

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.donor_ata.to_account_info(),
            to: self.stream_ata.to_account_info(),
            authority: self.donor.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token_transfer(cpi_ctx, amount)?;

        self.donor_account.set_inner(DonorAccount {
            stream: self.stream.key(),
            donor: self.donor.key(),
            amount: self.donor_account.amount.checked_add(amount).ok_or(StreamError::MathOverflow)?,
            refunded: false,
            bump: bumps.donor_account,
        });
        self.stream.total_deposited += self.stream.total_deposited.checked_add(amount).ok_or(StreamError::MathOverflow)?;
        emit!(DepositMade {
            stream: self.stream.key(),
            donor: self.donor.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }
}