use anchor_lang::prelude::*;

#[error_code]
pub enum EcosystemError {
    #[msg("LaunchPad is paused")]
    LaunchpadPaused,

    #[msg("Launchpad not initialized")]
    LaunchpadNotInitialized,

    #[msg("Insufficient USDC balance")]
    InsufficientUsdc,

    #[msg("Insufficient token balance")]
    InsufficientTokens,

    #[msg("Vesting period not complete")]
    VestingLocked,

    #[msg("Cannot transfer locked tokens")]
    CannotTransferLocked,

    #[msg("User not whitelisted for Tier 2")]
    NotWhitelisted,

    #[msg("Invalid tier")]
    InvalidTier,

    #[msg("Tokens already staked")]
    AlreadyStaked,

    #[msg("No staked tokens to unstake")]
    NoStakedTokens,

    #[msg("No yield to claim")]
    NoYieldToClaim,

    #[msg("Yield snapshot not ready")]
    YieldSnapshotNotReady,

    #[msg("Cannot redeem Tier 2 tokens")]
    CannotRedeemTier2,

    #[msg("Invalid redeem amount")]
    InvalidRedeemAmount,

    #[msg("Snapshot frequency not met")]
    SnapshotFrequencyNotMet,

    #[msg("Treasury operation failed")]
    TreasuryOperationFailed,

    #[msg("Aave integration error")]
    AaveIntegrationError,

    #[msg("Invalid distribution percentages (must sum to 100)")]
    InvalidDistributionPercentages,

    #[msg("Insufficient yield in treasury")]
    InsufficientYield,

    #[msg("Invalid PDA")]
    InvalidPda,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}
