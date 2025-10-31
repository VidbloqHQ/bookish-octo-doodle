use anchor_lang::prelude::*;

#[account]
pub struct StreamState {
    pub host: Pubkey,
    pub stream_name: String,
    pub bump: u8,
    pub mint: Pubkey,
    pub status: StreamStatus,
    pub total_deposited: u64,
    pub total_distributed: u64, 
    pub created_at: i64,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,  
    pub stream_type: StreamType, 
}

impl Space for StreamState {
    const INIT_SPACE: usize = 8      // Discriminator
        + 32    // host: Pubkey
        + 4 + 32 // stream_name: String (max 32 bytes)
        + 1     // bump: u8
        + 32    // mint: Pubkey
        + 1     // status: StreamStatus
        + 8     // total_deposited: u64
        + 8     // total_distributed: u64 
        + 8     // created_at: i64
        + 1 + 8 // start_time: Option<i64> (1 byte for Some/None + 8 bytes data)
        + 1 + 8 // end_time: Option<i64>
        + 1 + 16; // stream_type: StreamType (1 byte variant + max variant size)
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamStatus {
    Active,
    Ended,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StreamType {
    Prepaid {
        min_duration: u64,
    },
    Live,
    Conditional {
        min_amount: Option<u64>,
        unlock_time: Option<i64>,
    }
}

#[event]
pub struct StreamInitialized {
    pub stream: Pubkey,
    pub host: Pubkey,
    pub stream_type: StreamType,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub stream: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event] 
pub struct FundsDistributed {
    pub stream: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RefundProcessed {
    pub stream: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub remaining_balance: u64,
    pub timestamp: i64,
}


#[error_code]
pub enum StreamError {

    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Math overflow error")]
    MathOverflow,

    #[msg("Stream is not active")]
    StreamNotActive,
    
    #[msg("Stream is still time-locked")]
    StreamStillLocked,

    #[msg("Donor has already been refunded")]
    AlreadyRefunded,

    #[msg("Minimum duration must be greater than 0")]
    InvalidDuration,

    #[msg("Unlock time must be in the future")]
    InvalidTime,

    #[msg("Deposits not allowed in current stream state")]
    DepositNotAllowed,

    #[msg("Stream has not started yet")]
    StreamNotStarted,

    #[msg("Stream has already started")]
    StreamAlreadyStarted,

    #[msg("Stream has already ended")]
    StreamAlreadyEnded,

    #[msg("Unauthorized action")]
    Unauthorized,

    #[msg("Duration not met")]
    DurationNotMet,

    #[msg("Minimum amount not met")]
    AmountNotMet,
    
    #[msg("Stream is time-locked")]
    TimeLocked,

    #[msg("Name must be between 4 and 32 characters")]
    NameLengthInvalid,

    // Betting errors
      #[msg("Invalid market setup")]
    InvalidMarketSetup,
    #[msg("Invalid outcome")]
    InvalidOutcome,
    #[msg("Market already resolved")]
    MarketResolved,
    #[msg("Betting period closed")]
    BettingClosed,
    #[msg("Market not ready for resolution")]
    MarketNotReady,
    #[msg("Insufficient validators")]
    InsufficientValidators,
    #[msg("Invalid mint for betting market")]
    InvalidMint,
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Market not resolved")]
    MarketNotResolved,
    #[msg("Already claimed")]
    AlreadyClaimed,
    #[msg("No winnings to claim")]
    NoWinnings,
    #[msg("Invalid resolution state")]
    InvalidResolutionState,
    #[msg("Not a validator")]
    NotValidator,
    #[msg("Insufficient stake for validation")]
    InsufficientStakeForValidation,
    #[msg("Already voted")]
    AlreadyVoted,
}

// Remember to add the enum that Ayo suggested to handle donations and refunds