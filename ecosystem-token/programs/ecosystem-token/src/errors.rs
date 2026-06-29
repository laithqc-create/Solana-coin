use anchor_lang::prelude::*;

#[error_code]
pub enum EcosystemError {
    // ── Launchpad ──────────────────────────────────────────────────────────
    #[msg("Launchpad is paused")]
    LaunchpadPaused,
    #[msg("Insufficient USDC balance")]
    InsufficientUsdc,
    #[msg("Tier 2 tokens must complete the unstake flow before burning")]
    CannotRedeemTier2,
    #[msg("Invalid amount — must be greater than zero")]
    InvalidRedeemAmount,

    // ── Yield / Holders ────────────────────────────────────────────────────
    #[msg("No yield available to claim")]
    NoYieldToClaim,
    #[msg("Yield snapshot not ready for distribution")]
    YieldSnapshotNotReady,
    #[msg("Snapshot frequency not met")]
    SnapshotFrequencyNotMet,
    #[msg("Insufficient yield in treasury")]
    InsufficientYield,

    // ── Unstaking ─────────────────────────────────────────────────────────
    #[msg("48 business hours have not elapsed — funds still routing from Morpho to USDC")]
    UnstakeCooldownNotMet,
    #[msg("Unstake request already completed")]
    UnstakeAlreadyCompleted,
    #[msg("No pending unstake request found")]
    NoUnstakeRequest,
    #[msg("Unstake request already pending")]
    UnstakeAlreadyPending,

    // ── Morpho Investment ─────────────────────────────────────────────────
    #[msg("Investment blocked — waiting for admin approval of Morpho pool")]
    InvestmentPendingApproval,
    #[msg("Morpho pool type is not eligible (must be Isolated, Fixed-Collateral, or Cash-Backed)")]
    IneligiblePoolType,
    #[msg("Morpho pool not yet approved by admin")]
    PoolNotApproved,
    #[msg("Morpho pool is not currently active")]
    PoolNotActive,
    #[msg("Insufficient treasury balance for investment")]
    InsufficientTreasury,
    #[msg("No Morpho yield available to claim")]
    NoMorphoYield,
    #[msg("Invalid investment amount")]
    InvalidInvestmentAmount,

    // ── Revenue Distribution ───────────────────────────────────────────────
    #[msg("Revenue distribution shares must sum to 10000 basis points")]
    InvalidDistributionShares,
    #[msg("Investment ratio must be between 0 and 10000 basis points")]
    InvalidInvestmentRatio,

    // ── Vesting ───────────────────────────────────────────────────────────
    #[msg("Tokens are locked during the vesting period")]
    VestingLocked,
    #[msg("Cannot transfer locked Tier 2 tokens")]
    CannotTransferLocked,

    // ── Tokens ────────────────────────────────────────────────────────────
    #[msg("Insufficient token balance")]
    InsufficientTokens,
    #[msg("No tokens to operate on")]
    NoStakedTokens,

    // ── General ───────────────────────────────────────────────────────────
    #[msg("Arithmetic overflow — operation rejected for safety")]
    MathOverflow,
    #[msg("Unauthorized — signer is not the program authority")]
    Unauthorized,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}
