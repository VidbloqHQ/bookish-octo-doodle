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
    
    // ============= BETTING INSTRUCTIONS =============
    
    pub fn initialize_betting_market(
        ctx: Context<InitializeBettingMarket>,
        market_type: MarketType,
        outcomes: Vec<String>,
        resolution_time: i64,
        initial_liquidity: u64,
        fee_percentage: u16,
    ) -> Result<()> {
        ctx.accounts.initialize_market(market_type, outcomes, resolution_time, initial_liquidity, fee_percentage, &ctx.bumps)
    }
    
    pub fn place_bet(
        ctx: Context<PlaceBet>,
        outcome_id: u8,
        usdc_amount: u64,
        min_shares: u64,
    ) -> Result<()> {
        ctx.accounts.place_bet(outcome_id, usdc_amount, min_shares, &ctx.bumps)
    }
    
    pub fn request_market_randomness(
        ctx: Context<RequestMarketRandomness>,
        use_case: RandomnessUseCase,
        client_seed: [u8; 32],
        eligible_validators: Vec<EligibleValidator>,
    ) -> Result<()> {
        ctx.accounts.request_randomness(use_case, client_seed, eligible_validators, &ctx.bumps)
    }
    
    // VRF Callback - This MUST be in the main program module for Anchor to generate the discriminator
    pub fn callback_process_randomness(
        ctx: Context<CallbackProcessRandomness>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.process_randomness(randomness)
    }
    
    pub fn validator_vote_on_outcome(
        ctx: Context<ValidatorVoteOnOutcome>,
        outcome_id: u8,
    ) -> Result<()> {
        ctx.accounts.vote(outcome_id)
    }
    
    pub fn resolve_market(
        ctx: Context<ResolveMarket>,
        winning_outcome: u8,
    ) -> Result<()> {
        ctx.accounts.resolve_market(winning_outcome)
    }
    
    pub fn claim_winnings(
        ctx: Context<ClaimWinnings>,
    ) -> Result<()> {
        ctx.accounts.claim_winnings()
    }
}