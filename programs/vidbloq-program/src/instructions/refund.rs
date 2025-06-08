use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Transfer, transfer as token_transfer},
    token_interface::{TokenAccount, TokenInterface}
};
use crate::state::{StreamState, StreamError, DonorAccount, StreamStatus, RefundProcessed};

#[derive(Accounts)]
pub struct Refund <'info> {

     /// CHECK: This is the donor public key
    pub donor: AccountInfo<'info>,

     #[account(
        mut,
        constraint = (initiator.key() == stream.host || initiator.key() == donor.key())
    )]
    pub initiator: Signer<'info>,

    #[account(
        mut, 
        seeds=[b"stream", stream.stream_name.as_str().as_bytes(), stream.host.key().as_ref()],
        bump=stream.bump
     )]
    pub stream: Account<'info, StreamState>,

    #[account(
        mut,
        seeds = [b"donor", stream.key().as_ref(), donor.key().as_ref()],
        bump = donor_account.bump,
        constraint = donor_account.donor == donor.key(),
        constraint = donor_account.stream == stream.key()
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

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> Refund <'info> {
    pub fn refund(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, StreamError::InvalidAmount);
        require!(self.donor_account.refunded == false, StreamError::AlreadyRefunded);
        require!(amount <= self.donor_account.amount, StreamError::InsufficientFunds);

        require!(
            self.stream.status != StreamStatus::Ended,
            StreamError::StreamAlreadyEnded
        );

        // Calculate available stream balance
        let available_balance = self.stream.total_deposited
            .checked_sub(self.stream.total_distributed)
            .ok_or(StreamError::MathOverflow)?;

        // Ensure sufficient funds in the stream
        require!(available_balance >= amount, StreamError::InsufficientFunds);

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.stream_ata.to_account_info(),
            to: self.donor_ata.to_account_info(),
            authority: self.stream.to_account_info(),
        };

        // let stream_seeds = &[
        //     b"stream".as_ref(),
        //     self.stream.host.as_ref(),
        //     self.stream.stream_name.as_bytes(),
        //     &[self.stream.bump],
        // ];
        let stream_seeds = &[
            b"stream".as_ref(),
            self.stream.stream_name.as_str().as_bytes(),
            self.stream.host.as_ref(),
            &[self.stream.bump],
        ];
        let signer = &[&stream_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token_transfer(cpi_ctx, amount)?;
        // Update donor account
        self.donor_account.amount = self.donor_account.amount.checked_sub(amount).ok_or(StreamError::MathOverflow)?;
        
        // Mark as fully refunded if all funds returned
        if self.donor_account.amount == 0 {
            self.donor_account.refunded = true;
        }
        
        // Update stream state
        self.stream.total_deposited = self.stream.total_deposited.checked_sub(amount).ok_or(StreamError::MathOverflow)?;

        emit!(RefundProcessed {
            stream: self.stream.key(),
            donor: self.donor.key(),
            amount,
            remaining_balance: self.donor_account.amount,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }
}