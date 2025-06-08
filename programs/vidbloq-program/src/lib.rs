#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use crate::instructions::*;
use crate::state::*;

declare_id!("14SYsuFUHifkTHbgcvrZ4xKMsqeFGCD3rV7qNoZLdoND");

#[program]
pub mod vidbloq_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, stream_type: StreamType, end_time: Option<i64>) -> Result<()> {
        ctx.accounts.initialize(name, stream_type, end_time, &ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount, &ctx.bumps)?;
        Ok(())
    }
    pub fn refund(ctx: Context<Refund>, amount: u64) -> Result<()> {
        ctx.accounts.refund(amount)?;
        Ok(())
    }
    pub fn distribute(ctx: Context<Distribute>, amount: u64) -> Result<()> {
        ctx.accounts.distribute(amount)?;
        Ok(())
    }
    pub fn start_stream(ctx: Context<StartStream>) -> Result<()> {
        ctx.accounts.start_stream()?;
        Ok(())
    }
    pub fn complete_stream(ctx: Context<CompleteStream>) -> Result<()> {
        ctx.accounts.complete_stream()?;
        Ok(())
    }
    pub fn update_stream(ctx: Context<UpdateStream>, new_end_time: Option<i64>, new_status: Option<StreamStatus>) -> Result<()> {
        ctx.accounts.update_stream(new_end_time, new_status)?;
        Ok(())
    }
}

