use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface}
};

use crate::state::{StreamState, StreamStatus, StreamError, StreamType};

#[derive(Accounts)]
#[instruction(stream_name: String, stream_type: StreamType, end_date: Option<i64>)]
pub struct Initialize <'info> {
    #[account(mut)]
    pub host: Signer<'info>,

    #[account(
        init, 
        payer=host,
        space=StreamState::INIT_SPACE,
        seeds=[b"stream", 
        stream_name.as_str().as_bytes(),
        // &stream_name.as_bytes()[..std::cmp::min(stream_name.len(), 32)],
        host.key().as_ref()],
        bump
    )]
    pub stream: Account<'info, StreamState>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(init, 
        associated_token::mint=mint, 
        associated_token::authority=stream, 
        payer=host)]
    pub stream_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> Initialize <'info> {
    pub fn initialize(&mut self, name: String, stream_type: StreamType, end_time: Option<i64>, bumps: &InitializeBumps) -> Result<()> {

        require!(
            name.len() >= 4 && name.len() <= 32,
            StreamError::NameLengthInvalid
        );

        match &stream_type {
            StreamType::Prepaid { min_duration } => {
                require!(*min_duration > 0, StreamError::InvalidDuration);
            },
            StreamType::Conditional { min_amount, unlock_time } => {
                if let Some(amount) = min_amount {
                    require!(*amount > 0, StreamError::InvalidAmount);
                }
                if let Some(time) = unlock_time {
                    require!(*time > Clock::get()?.unix_timestamp, StreamError::InvalidTime);
                }
            }
            StreamType::Live => {
                // No additional validation needed
            }
        }
        self.stream.set_inner(StreamState {
            host: self.host.key(),
            stream_name: name,
            bump: bumps.stream,
            total_distributed: 0,
            total_deposited: 0,
            status: StreamStatus::Active,
            mint: self.mint.key(),
            end_time,
            stream_type,
            created_at: Clock::get()?.unix_timestamp,
            start_time: None,
        });
        Ok(())
    }
}