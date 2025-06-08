use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Transfer, transfer as token_transfer},
    token_interface::{TokenAccount, TokenInterface, Mint}
    // token::{Transfer, transfer as token_transfer, TokenAccount, Token},
};

use crate::state::{StreamState, StreamStatus, StreamError, StreamType, FundsDistributed};

#[derive(Accounts)]
pub struct Distribute <'info> {
    #[account(mut)]
    pub host: Signer<'info>,

    /// CHECK: This is the recipient public key
    pub recipient: AccountInfo<'info>,

    /// VERIFIED MINT: Must match stream.mint
    #[account(
        address = stream.mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut, 
        constraint = stream.host == host.key(),
        seeds=[b"stream", stream.stream_name.as_str().as_bytes(), stream.host.key().as_ref()],
        bump=stream.bump
     )]
    pub stream: Account<'info, StreamState>,

    #[account(
        mut,
        constraint = stream_ata.mint == stream.mint,
        constraint = stream_ata.owner == stream.key()
    )]
    pub stream_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = host,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> Distribute <'info> {
    pub fn distribute(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, StreamError::InvalidAmount);

        require!(
            self.stream.host == self.host.key(),
            StreamError::Unauthorized
        );

        // Check if stream is still active
        require!(self.stream.status == StreamStatus::Active, StreamError::StreamNotActive);

        // StreamType-specific checks
        match self.stream.stream_type {
            StreamType::Prepaid { min_duration } => {
                // For prepaid, must meet minimum duration
                let elapsed = Clock::get()?.unix_timestamp 
                    - self.stream.start_time.ok_or(StreamError::StreamNotStarted)?;
                require!(
                    elapsed >= min_duration as i64,
                    StreamError::DurationNotMet
                );
            },
            StreamType::Live => {
                // No additional restrictions for live streams
            },
            StreamType::Conditional { min_amount, unlock_time } => {
                // Check minimum amount if specified
                if let Some(min) = min_amount {
                    require!(
                        self.stream.total_deposited >= min,
                        StreamError::AmountNotMet
                    );
                }
                // Check unlock time if specified
                if let Some(time) = unlock_time {
                    require!(
                        Clock::get()?.unix_timestamp >= time,
                        StreamError::TimeLocked
                    );
                }
            }
        }
        

        if let Some(end_time) = self.stream.end_time {
            let current_time = Clock::get()?.unix_timestamp;
            require!(
                current_time >= end_time,
                StreamError::StreamStillLocked
            );
        }

        // Calculate available balance
        let available_balance = self.stream.total_deposited
            .checked_sub(self.stream.total_distributed)
            .ok_or(StreamError::MathOverflow)?;
            
        // Ensure sufficient funds
        require!(available_balance >= amount, StreamError::InsufficientFunds);

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.stream_ata.to_account_info(),
            to: self.recipient_ata.to_account_info(),
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

        self.stream.total_distributed = self.stream.total_distributed.checked_add(amount).ok_or(StreamError::MathOverflow)?;

        emit!(FundsDistributed {
            stream: self.stream.key(),
            recipient: self.recipient.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }
}