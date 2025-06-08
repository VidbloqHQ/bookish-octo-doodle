use anchor_lang::prelude::*;

use crate::state::{StreamState, StreamStatus, StreamError};

#[derive(Accounts)]
pub struct StartStream<'info> {
    #[account(mut)]
    pub host: Signer<'info>,
    
    #[account(
        mut,
        has_one = host,
        seeds = [
            b"stream",
            stream.stream_name.as_bytes(),
            stream.host.as_ref()
        ],
        bump = stream.bump
    )]
    pub stream: Account<'info, StreamState>,
}

impl<'info> StartStream<'info> {
    pub fn start_stream(&mut self) -> Result<()> {
        require!(
            self.stream.status == StreamStatus::Active,
            StreamError::StreamNotActive
        );
        require!(
            self.stream.start_time.is_none(),
            StreamError::StreamAlreadyStarted
        );
        
        self.stream.start_time = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CompleteStream<'info> {
    #[account(mut)]
    pub host: Signer<'info>,
    
    #[account(
        mut,
        has_one = host,
        seeds = [
            b"stream",
            stream.stream_name.as_bytes(),
            stream.host.as_ref()
        ],
        bump = stream.bump
    )]
    pub stream: Account<'info, StreamState>,
}

impl<'info> CompleteStream<'info> {
    pub fn complete_stream(&mut self) -> Result<()> {
        require!(
            self.stream.status == StreamStatus::Active,
            StreamError::StreamNotActive
        );
        require!(
            self.stream.start_time.is_some(),
            StreamError::StreamNotStarted
        );
        
        self.stream.status = StreamStatus::Ended;
        self.stream.end_time = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateStream<'info> {
    #[account(mut)]
    pub host: Signer<'info>,
    
    #[account(
        mut,
        has_one = host,
        seeds = [b"stream", stream.stream_name.as_bytes(), host.key().as_ref()],
        bump = stream.bump
    )]
    pub stream: Account<'info, StreamState>,
}

impl<'info> UpdateStream<'info> {
    pub fn update_stream(
        &mut self,
        new_end_time: Option<i64>,
        new_status: Option<StreamStatus>
    ) -> Result<()> {
        if let Some(end_time) = new_end_time {
            self.stream.end_time = Some(end_time);
        }
        if let Some(status) = new_status {
            self.stream.status = status;
        }
        Ok(())
    }
}