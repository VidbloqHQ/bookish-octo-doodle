use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer as token_transfer, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

use crate::state::{
    BetPlaced, BettingMarket, BettorPosition, EligibleValidator, MarketCreated, MarketOutcome,
    MarketResolution, MarketType, OutcomePosition, RandomnessUseCase, ResolutionStatus,
    StreamError, StreamState, ValidationVote, ValidatorVote, WinningsClaimed,
};

// ============= CONSTANTS =============
pub const MARKET_SEED: &[u8] = b"betting_market";
pub const RESOLUTION_SEED: &[u8] = b"market_resolution";
pub const POSITION_SEED: &[u8] = b"bettor_position";
pub const MARKET_VAULT_SEED: &[u8] = b"market_vault";
pub const MIN_VALIDATORS: u8 = 3;
pub const MAX_VALIDATORS: u8 = 7;
pub const VALIDATOR_STAKE_REQUIREMENT: u64 = 10_000_000; // 10 USDC minimum
pub const DISPUTE_WINDOW: i64 = 3600; // 1 hour
pub const VALIDATOR_REWARD_BPS: u16 = 50; // 0.5% of pool

// ============= INSTRUCTIONS CONTEXTS =============

/// Initialize a betting market
#[derive(Accounts)]
#[instruction(market_type: MarketType, outcomes: Vec<String>)]
pub struct InitializeBettingMarket<'info> {
    #[account(mut)]
    pub host: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stream", stream.stream_name.as_str().as_bytes(), stream.host.key().as_ref()],
        bump = stream.bump,
        constraint = stream.host == host.key() @ StreamError::Unauthorized,
    )]
    pub stream: Account<'info, StreamState>,

    /// The mint for betting (should match stream mint - USDC)
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = host,
        space = 8 + 32 + 32 + 32 + 100 + (100 * 10) + 8 + 8 + 8 + 1 + 2 + 1 + 2 + 8 + 1,
        seeds = [MARKET_SEED, stream.key().as_ref()],
        bump
    )]
    pub betting_market: Account<'info, BettingMarket>,

    pub system_program: Program<'info, System>,
}

/// Place a bet
#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub bettor: Signer<'info>,

    #[account(
        mut,
        seeds = [MARKET_SEED, betting_market.stream.as_ref()],
        bump = betting_market.bump,
    )]
    pub betting_market: Account<'info, BettingMarket>,

    #[account(
        init_if_needed,
        payer = bettor,
        space = 8 + 32 + 32 + (50 * 10) + 8 + 8 + 1 + 1 + 8 + 1,
        seeds = [POSITION_SEED, betting_market.key().as_ref(), bettor.key().as_ref()],
        bump
    )]
    pub bettor_position: Account<'info, BettorPosition>,

    /// The mint for the token (USDC) - must match market's mint
    #[account(
        constraint = mint.key() == betting_market.mint @ StreamError::InvalidMint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = bettor_token.owner == bettor.key(),
        constraint = bettor_token.mint == mint.key(),
    )]
    pub bettor_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = bettor,
        seeds = [MARKET_VAULT_SEED, betting_market.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = betting_market,
    )]
    pub market_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Request randomness for market operations
#[vrf]
#[derive(Accounts)]
pub struct RequestMarketRandomness<'info> {
    #[account(mut)]
    pub requestor: Signer<'info>,

    #[account(
        mut,
        seeds = [MARKET_SEED, market.stream.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, BettingMarket>,

    #[account(
        init_if_needed,
        payer = requestor,
        space = 8 + 32 + 2 + (32 * 20) + (100 * 10) + 8 + 50 + 32 + 50 + 8 + (50 * 100) + 1,
        seeds = [RESOLUTION_SEED, market.key().as_ref()],
        bump
    )]
    pub resolution: Account<'info, MarketResolution>,

    /// CHECK: The oracle queue from Ephemeral VRF
    #[account(mut, address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE)]
    pub oracle_queue: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

/// Callback from Ephemeral VRF with randomness
#[derive(Accounts)]
pub struct CallbackProcessRandomness<'info> {
    /// CHECK: Must be Ephemeral VRF program identity
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, BettingMarket>,

    #[account(mut)]
    pub resolution: Account<'info, MarketResolution>,
}

/// Validator votes on proposed outcome
#[derive(Accounts)]
pub struct ValidatorVoteOnOutcome<'info> {
    #[account(mut)]
    pub validator: Signer<'info>,

    #[account(
        seeds = [MARKET_SEED, market.stream.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, BettingMarket>,

    #[account(
        mut,
        seeds = [RESOLUTION_SEED, market.key().as_ref()],
        bump = resolution.bump,
    )]
    pub resolution: Account<'info, MarketResolution>,

    #[account(
        seeds = [POSITION_SEED, market.key().as_ref(), validator.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, BettorPosition>,
}

/// Resolve the market with a winner
#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub host: Signer<'info>,

    #[account(
        mut,
        seeds = [MARKET_SEED, betting_market.stream.as_ref()],
        bump = betting_market.bump,
        constraint = betting_market.host == host.key() @ StreamError::Unauthorized,
    )]
    pub betting_market: Account<'info, BettingMarket>,
}

/// Claim winnings after market resolution
#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub bettor: Signer<'info>,

    #[account(
        seeds = [MARKET_SEED, betting_market.stream.as_ref()],
        bump = betting_market.bump,
    )]
    pub betting_market: Account<'info, BettingMarket>,

    #[account(
        mut,
        seeds = [POSITION_SEED, betting_market.key().as_ref(), bettor.key().as_ref()],
        bump = bettor_position.bump,
    )]
    pub bettor_position: Account<'info, BettorPosition>,

    #[account(
        mut,
        seeds = [MARKET_VAULT_SEED, betting_market.key().as_ref()],
        bump,
    )]
    pub market_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub bettor_token: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

// ============= IMPLEMENTATION =============

impl<'info> InitializeBettingMarket<'info> {
    pub fn initialize_market(
        &mut self,
        market_type: MarketType,
        outcomes: Vec<String>,
        resolution_time: i64,
        initial_liquidity: u64,
        fee_percentage: u16,
        bumps: &InitializeBettingMarketBumps,
    ) -> Result<()> {
        // Validate inputs
        match &market_type {
            MarketType::Binary => {
                require!(outcomes.len() == 2, StreamError::InvalidMarketSetup);
            }
            MarketType::MultiOutcome { max } => {
                require!(
                    outcomes.len() >= 2 && outcomes.len() <= *max as usize,
                    StreamError::InvalidMarketSetup
                );
            }
            _ => {}
        }

        require!(
            resolution_time > Clock::get()?.unix_timestamp,
            StreamError::InvalidTime
        );
        require!(fee_percentage <= 1000, StreamError::InvalidFeePercentage); // Max 10%

        // Initialize market outcomes
        let mut market_outcomes = Vec::new();
        let liquidity_per_outcome = if initial_liquidity > 0 {
            initial_liquidity / outcomes.len() as u64
        } else {
            1000_000_000 // 1000 USDC default liquidity per outcome
        };

        for (i, desc) in outcomes.iter().enumerate() {
            market_outcomes.push(MarketOutcome {
                id: i as u8,
                description: desc.clone(),
                total_shares: 0,
                liquidity_reserve: liquidity_per_outcome,
                total_backing: 0,
            });
        }

        // Set the market data
        self.betting_market.set_inner(BettingMarket {
            stream: self.stream.key(),
            host: self.host.key(),
            mint: self.mint.key(),
            market_type,
            outcomes: market_outcomes,
            total_pool: 0,
            total_liquidity: initial_liquidity,
            resolution_time,
            resolved: false,
            winning_outcome: None,
            randomness_requested: false,
            fee_percentage,
            created_at: Clock::get()?.unix_timestamp,
            bump: bumps.betting_market,
        });

        msg!(
            "Betting market initialized with {} outcomes",
            outcomes.len()
        );

        emit!(MarketCreated {
            market: self.betting_market.key(),
            stream: self.stream.key(),
            market_type: self.betting_market.market_type.clone(),
            outcomes,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> PlaceBet<'info> {
    pub fn place_bet(
        &mut self,
        outcome_id: u8,
        usdc_amount: u64,
        min_shares: u64,
        bumps: &PlaceBetBumps,
    ) -> Result<()> {
        // Validate market state
        require!(!self.betting_market.resolved, StreamError::MarketResolved);
        require!(
            Clock::get()?.unix_timestamp < self.betting_market.resolution_time,
            StreamError::BettingClosed
        );
        require!(
            (outcome_id as usize) < self.betting_market.outcomes.len(),
            StreamError::InvalidOutcome
        );
        require!(usdc_amount > 0, StreamError::InvalidAmount);

        // Calculate shares using AMM
        let shares_out = self.calculate_shares_for_purchase(outcome_id, usdc_amount)?;
        require!(shares_out >= min_shares, StreamError::SlippageExceeded);

        msg!("Purchasing {} shares for {} USDC", shares_out, usdc_amount);

        // Transfer USDC from bettor to market vault
        let cpi_accounts = Transfer {
            from: self.bettor_token.to_account_info(),
            to: self.market_vault.to_account_info(),
            authority: self.bettor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        token_transfer(cpi_ctx, usdc_amount)?;

        // Update market state
        let outcome = &mut self.betting_market.outcomes[outcome_id as usize];
        outcome.total_shares = outcome
            .total_shares
            .checked_add(shares_out)
            .ok_or(StreamError::MathOverflow)?;
        outcome.total_backing = outcome
            .total_backing
            .checked_add(usdc_amount)
            .ok_or(StreamError::MathOverflow)?;
        // Half goes to liquidity for AMM stability
        outcome.liquidity_reserve = outcome
            .liquidity_reserve
            .checked_add(usdc_amount / 2)
            .ok_or(StreamError::MathOverflow)?;

        self.betting_market.total_pool = self
            .betting_market
            .total_pool
            .checked_add(usdc_amount)
            .ok_or(StreamError::MathOverflow)?;

        // Initialize bettor position if needed
        if self.bettor_position.bettor == Pubkey::default() {
            self.bettor_position.set_inner(BettorPosition {
                bettor: self.bettor.key(),
                market: self.betting_market.key(),
                positions: Vec::new(),
                total_invested: 0,
                total_returned: 0,
                has_claimed: false,
                is_eligible_validator: false,
                created_at: Clock::get()?.unix_timestamp,
                bump: bumps.bettor_position,
            });
        }

        // Update or add outcome position
        let position_idx = self
            .bettor_position
            .positions
            .iter()
            .position(|p| p.outcome_id == outcome_id);

        if let Some(idx) = position_idx {
            // Update existing position
            let pos = &mut self.bettor_position.positions[idx];
            let new_total_invested = pos
                .invested
                .checked_add(usdc_amount)
                .ok_or(StreamError::MathOverflow)?;
            let new_total_shares = pos
                .shares
                .checked_add(shares_out)
                .ok_or(StreamError::MathOverflow)?;

            // Calculate new average price
            pos.avg_entry_price = new_total_invested
                .checked_mul(1_000_000)
                .ok_or(StreamError::MathOverflow)?
                .checked_div(new_total_shares)
                .ok_or(StreamError::MathOverflow)?;

            pos.shares = new_total_shares;
            pos.invested = new_total_invested;
        } else {
            // Create new position
            self.bettor_position.positions.push(OutcomePosition {
                outcome_id,
                shares: shares_out,
                avg_entry_price: usdc_amount
                    .checked_mul(1_000_000)
                    .ok_or(StreamError::MathOverflow)?
                    .checked_div(shares_out)
                    .ok_or(StreamError::MathOverflow)?,
                invested: usdc_amount,
            });
        }

        // Update total invested
        self.bettor_position.total_invested = self
            .bettor_position
            .total_invested
            .checked_add(usdc_amount)
            .ok_or(StreamError::MathOverflow)?;

        // Check if eligible for validation
        if self.bettor_position.total_invested >= VALIDATOR_STAKE_REQUIREMENT {
            self.bettor_position.is_eligible_validator = true;
        }

        emit!(BetPlaced {
            market: self.betting_market.key(),
            bettor: self.bettor.key(),
            outcome_id,
            shares: shares_out,
            price: usdc_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    fn calculate_shares_for_purchase(&self, outcome_id: u8, usdc_amount: u64) -> Result<u64> {
        let outcome = &self.betting_market.outcomes[outcome_id as usize];

        // Constant product AMM formula: shares_out = reserve * amount_in / (reserve + amount_in)
        // This ensures price increases as more people bet on the same outcome
        let shares = (outcome.liquidity_reserve as u128)
            .checked_mul(usdc_amount as u128)
            .ok_or(StreamError::MathOverflow)?
            .checked_div(
                (outcome.liquidity_reserve as u128)
                    .checked_add(usdc_amount as u128)
                    .ok_or(StreamError::MathOverflow)?,
            )
            .ok_or(StreamError::MathOverflow)? as u64;

        // Ensure we don't give 0 shares
        require!(shares > 0, StreamError::InvalidAmount);

        Ok(shares)
    }
}

impl<'info> RequestMarketRandomness<'info> {
    pub fn request_randomness(
        &mut self,
        use_case: RandomnessUseCase,
        client_seed: [u8; 32],
        eligible_validators: Vec<EligibleValidator>,
        bumps: &RequestMarketRandomnessBumps,
    ) -> Result<()> {
        msg!("Requesting randomness for {:?}", use_case);

        // Validate based on use case
        match &use_case {
            RandomnessUseCase::ValidatorSelection => {
                require!(
                    Clock::get()?.unix_timestamp >= self.market.resolution_time,
                    StreamError::MarketNotReady
                );
                require!(
                    !eligible_validators.is_empty(),
                    StreamError::InsufficientValidators
                );
            }
            _ => {}
        }

        // Initialize or update resolution account
        if self.resolution.market == Pubkey::default() {
            // First time initialization
            self.resolution.set_inner(MarketResolution {
                market: self.market.key(),
                proposed_outcome: None,
                validators: Vec::new(),
                validator_votes: Vec::new(),
                dispute_end_time: Clock::get()?.unix_timestamp + DISPUTE_WINDOW,
                resolution_status: ResolutionStatus::AwaitingRandomness,
                randomness_seed: [0u8; 32],
                randomness_use_case: use_case.clone(),
                total_stake_validating: 0,
                eligible_validators, // Store the eligible validators
                bump: bumps.resolution,
            });
        } else {
            // Update existing resolution
            self.resolution.randomness_use_case = use_case.clone();
            self.resolution.eligible_validators = eligible_validators;
            self.resolution.resolution_status = ResolutionStatus::AwaitingRandomness;
        }

        // Create the randomness request instruction
        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.requestor.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            // Use the instruction discriminator that Anchor generates
            callback_discriminator: crate::instruction::CallbackProcessRandomness::DISCRIMINATOR
                .to_vec(),
            caller_seed: client_seed,
            accounts_metas: Some(vec![
                SerializableAccountMeta {
                    pubkey: self.market.key(),
                    is_signer: false,
                    is_writable: true,
                },
                SerializableAccountMeta {
                    pubkey: self.resolution.key(),
                    is_signer: false,
                    is_writable: true,
                },
            ]),
            ..Default::default()
        });

        // Invoke the VRF instruction
        self.invoke_signed_vrf(&self.requestor.to_account_info(), &ix)?;

        Ok(())
    }
}

impl<'info> CallbackProcessRandomness<'info> {
    pub fn process_randomness(&mut self, randomness: [u8; 32]) -> Result<()> {
        msg!("Processing randomness callback");

        // Use Ephemeral VRF's random utilities
        match self.resolution.randomness_use_case {
            RandomnessUseCase::ValidatorSelection => {
                // Select validators using randomness
                let num_validators = MIN_VALIDATORS;
                let random_value =
                    ephemeral_vrf_sdk::rnd::random_u8_with_range(&randomness, 0, num_validators);
                msg!("Selected {} validators", random_value);
            }
            RandomnessUseCase::TieBreaker => {
                // Resolve tie with randomness
                let winner = ephemeral_vrf_sdk::rnd::random_u8_with_range(
                    &randomness,
                    0,
                    self.market.outcomes.len() as u8,
                );
                self.market.winning_outcome = Some(winner);
                self.market.resolved = true;
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'info> ValidatorVoteOnOutcome<'info> {
    pub fn vote(&mut self, outcome_id: u8) -> Result<()> {
        // Validate voting conditions
        require!(
            self.resolution.resolution_status == ResolutionStatus::UnderValidation,
            StreamError::InvalidResolutionState
        );
        require!(
            self.resolution.validators.contains(&self.validator.key()),
            StreamError::NotValidator
        );
        require!(
            self.position.total_invested >= VALIDATOR_STAKE_REQUIREMENT,
            StreamError::InsufficientStakeForValidation
        );
        require!(
            (outcome_id as usize) < self.market.outcomes.len(),
            StreamError::InvalidOutcome
        );

        // Check if already voted
        let already_voted = self
            .resolution
            .validator_votes
            .iter()
            .any(|v| v.validator == self.validator.key());
        require!(!already_voted, StreamError::AlreadyVoted);

        msg!(
            "Validator {} voting for outcome {}",
            self.validator.key(),
            outcome_id
        );

        // Record the vote
        self.resolution.validator_votes.push(ValidatorVote {
            validator: self.validator.key(),
            voted_outcome: outcome_id,
            vote_timestamp: Clock::get()?.unix_timestamp,
            stake_amount: self.position.total_invested,
        });

        // Update total stake validating
        self.resolution.total_stake_validating = self
            .resolution
            .total_stake_validating
            .checked_add(self.position.total_invested)
            .ok_or(StreamError::MathOverflow)?;

        // Check if we have enough votes for consensus (2/3 of validators)
        let required_votes = (self.resolution.validators.len() * 2) / 3;
        if self.resolution.validator_votes.len() >= required_votes {
            self.check_consensus()?;
        }

        emit!(ValidationVote {
            market: self.market.key(),
            validator: self.validator.key(),
            voted_outcome: outcome_id,
            stake_weight: self.position.total_invested,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    fn check_consensus(&mut self) -> Result<()> {
        // Count votes weighted by stake
        let mut outcome_stakes: Vec<(u8, u64)> = Vec::new();

        for vote in &self.resolution.validator_votes {
            if let Some(pos) = outcome_stakes
                .iter_mut()
                .find(|(id, _)| *id == vote.voted_outcome)
            {
                pos.1 = pos
                    .1
                    .checked_add(vote.stake_amount)
                    .ok_or(StreamError::MathOverflow)?;
            } else {
                outcome_stakes.push((vote.voted_outcome, vote.stake_amount));
            }
        }

        // Find outcome with most stake
        let mut winning_outcome = 0u8;
        let mut max_stake = 0u64;

        for (outcome, stake) in outcome_stakes.iter() {
            if *stake > max_stake {
                max_stake = *stake;
                winning_outcome = *outcome;
            }
        }

        // Check if we have super-majority (66%+ of total stake)
        let required_stake = (self.resolution.total_stake_validating * 2) / 3;
        if max_stake >= required_stake {
            msg!(
                "Consensus reached: outcome {} with {} stake",
                winning_outcome,
                max_stake
            );
            self.resolution.proposed_outcome = Some(winning_outcome);
            self.resolution.resolution_status = ResolutionStatus::Finalized;

            // Note: Actual market resolution should be done in a separate instruction
            // to maintain separation of concerns
        } else {
            msg!(
                "No consensus yet. Max stake: {}, required: {}",
                max_stake,
                required_stake
            );
        }

        Ok(())
    }
}

impl<'info> ResolveMarket<'info> {
    pub fn resolve_market(&mut self, winning_outcome: u8) -> Result<()> {
        msg!("Resolving market with outcome {}", winning_outcome);
        self.betting_market.winning_outcome = Some(winning_outcome);
        self.betting_market.resolved = true;
        Ok(())
    }
}

impl<'info> ClaimWinnings<'info> {
    pub fn claim_winnings(&mut self) -> Result<()> {
        // Validate market is resolved
        require!(self.betting_market.resolved, StreamError::MarketNotResolved);
        let winning_outcome = self
            .betting_market
            .winning_outcome
            .ok_or(StreamError::MarketNotResolved)?;

        // Check if already claimed
        require!(
            !self.bettor_position.has_claimed,
            StreamError::AlreadyClaimed
        );

        // Calculate winnings
        let mut payout = 0u64;
        let mut has_winning_position = false;

        for position in &self.bettor_position.positions {
            if position.outcome_id == winning_outcome {
                has_winning_position = true;

                // Calculate share of the total pool
                let winning_outcome_data = &self.betting_market.outcomes[winning_outcome as usize];

                if winning_outcome_data.total_shares > 0 {
                    // Calculate proportional share of the entire pool
                    let share_value = (self.betting_market.total_pool as u128)
                        .checked_mul(position.shares as u128)
                        .ok_or(StreamError::MathOverflow)?
                        .checked_div(winning_outcome_data.total_shares as u128)
                        .ok_or(StreamError::MathOverflow)?
                        as u64;

                    // Apply platform fee
                    let fee = (share_value as u128)
                        .checked_mul(self.betting_market.fee_percentage as u128)
                        .ok_or(StreamError::MathOverflow)?
                        .checked_div(10000)
                        .ok_or(StreamError::MathOverflow)? as u64;

                    let net_payout = share_value
                        .checked_sub(fee)
                        .ok_or(StreamError::MathOverflow)?;

                    payout = payout
                        .checked_add(net_payout)
                        .ok_or(StreamError::MathOverflow)?;
                }
            }
        }

        require!(has_winning_position, StreamError::NoWinnings);
        require!(payout > 0, StreamError::NoWinnings);

        msg!("Claiming {} USDC in winnings", payout);

        // Transfer winnings from market vault to bettor
        let market_seeds = &[
            MARKET_SEED,
            self.betting_market.stream.as_ref(),
            &[self.betting_market.bump],
        ];
        let signer = &[&market_seeds[..]];

        let cpi_accounts = Transfer {
            from: self.market_vault.to_account_info(),
            to: self.bettor_token.to_account_info(),
            authority: self.betting_market.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);
        token_transfer(cpi_ctx, payout)?;

        // Update bettor position
        self.bettor_position.has_claimed = true;
        self.bettor_position.total_returned = payout;

        emit!(WinningsClaimed {
            market: self.betting_market.key(),
            bettor: self.bettor.key(),
            payout,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}
