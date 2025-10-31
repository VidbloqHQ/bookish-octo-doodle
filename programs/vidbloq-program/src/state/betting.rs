use anchor_lang::prelude::*;

#[account]
pub struct BettingMarket {
    pub stream: Pubkey,
    pub host: Pubkey,
    pub mint: Pubkey,  // Store the mint (USDC) for this market
    pub market_type: MarketType,
    pub outcomes: Vec<MarketOutcome>,
    pub total_pool: u64,
    pub total_liquidity: u64,
    pub resolution_time: i64,
    pub resolved: bool,
    pub winning_outcome: Option<u8>,
    pub randomness_requested: bool,
    pub fee_percentage: u16,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct MarketResolution {
    pub market: Pubkey,
    pub proposed_outcome: Option<u8>,
    pub validators: Vec<Pubkey>,
    pub validator_votes: Vec<ValidatorVote>,
    pub dispute_end_time: i64,
    pub resolution_status: ResolutionStatus,
    pub randomness_seed: [u8; 32],
    pub randomness_use_case: RandomnessUseCase,
    pub total_stake_validating: u64,
    pub eligible_validators: Vec<EligibleValidator>,
    pub bump: u8,
}

#[account]
pub struct BettorPosition {
    pub bettor: Pubkey,
    pub market: Pubkey,
    pub positions: Vec<OutcomePosition>,
    pub total_invested: u64,
    pub total_returned: u64,
    pub has_claimed: bool,
    pub is_eligible_validator: bool,
    pub created_at: i64,
    pub bump: u8,
}

// ============= TYPES =============

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketOutcome {
    pub id: u8,
    pub description: String,
    pub total_shares: u64,
    pub liquidity_reserve: u64,
    pub total_backing: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidatorVote {
    pub validator: Pubkey,
    pub voted_outcome: u8,
    pub vote_timestamp: i64,
    pub stake_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EligibleValidator {
    pub pubkey: Pubkey,
    pub stake: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OutcomePosition {
    pub outcome_id: u8,
    pub shares: u64,
    pub avg_entry_price: u64,
    pub invested: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarketType {
    Binary,
    MultiOutcome { max: u8 },
    OverUnder { line: u64 },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ResolutionStatus {
    AwaitingRandomness,
    SelectingValidators,
    UnderValidation,
    Disputed,
    Finalized,
    ForcedByRandomness,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RandomnessUseCase {
    ValidatorSelection,
    TieBreaker,
    DisputeResolution,
    FairDistribution,
}

// ============= EVENTS =============

#[event]
pub struct RandomnessRequested {
    pub market: Pubkey,
    pub use_case: RandomnessUseCase,
    pub requestor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ValidatorsSelected {
    pub market: Pubkey,
    pub validators: Vec<Pubkey>,
    pub total_validators: u8,
    pub timestamp: i64,
}

#[event]
pub struct MarketResolved {
    pub market: Pubkey,
    pub winning_outcome: u8,
    pub total_pool: u64,
    pub used_randomness: bool,
    pub timestamp: i64,
}

#[event]
pub struct BetPlaced {
    pub market: Pubkey,
    pub bettor: Pubkey,
    pub outcome_id: u8,
    pub shares: u64,
    pub price: u64,
    pub timestamp: i64,
}

#[event]
pub struct WinningsClaimed {
    pub market: Pubkey,
    pub bettor: Pubkey,
    pub payout: u64,
    pub timestamp: i64,
}

#[event]
pub struct MarketCreated {
    pub market: Pubkey,
    pub stream: Pubkey,
    pub market_type: MarketType,
    pub outcomes: Vec<String>,
    pub timestamp: i64,
}

#[event]
pub struct ValidationVote {
    pub market: Pubkey,
    pub validator: Pubkey,
    pub voted_outcome: u8,
    pub stake_weight: u64,
    pub timestamp: i64,
}
