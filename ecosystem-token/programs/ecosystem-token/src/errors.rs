use anchor_lang::prelude::*;

#[error_code]
pub enum EcosystemError {
    // ── Minting ────────────────────────────────────────────────────────────
    #[msg("Launchpad is paused")]
    LaunchpadPaused,
    #[msg("Insufficient USDC balance")]
    InsufficientUsdc,
    #[msg("Tier 2 tokens cannot be redeemed directly — use unstake")]
    CannotRedeemTier2,
    #[msg("Invalid redeem amount")]
    InvalidRedeemAmount,

    // ── Staking ────────────────────────────────────────────────────────────
    #[msg("Insufficient token balance")]
    InsufficientTokens,
    #[msg("No tokens staked")]
    NoStakedTokens,
    #[msg("Vesting period is still active — tokens are locked")]
    VestingLocked,
    #[msg("Cannot transfer locked Tier 2 tokens during vesting")]
    CannotTransferLocked,

    // ── Unstaking / RWA redemption ─────────────────────────────────────────
    #[msg("Unstake request already pending for this user")]
    UnstakeAlreadyPending,
    #[msg("48 business hours have not elapsed yet — funds are still in RUSDY")]
    UnstakeCooldownNotMet,
    #[msg("Unstake request already completed")]
    UnstakeAlreadyCompleted,
    #[msg("No pending unstake request found")]
    NoUnstakeRequest,

    // ── RWA / RUSDY ────────────────────────────────────────────────────────
    #[msg("RUSDY investment amount must be greater than zero")]
    InvalidRwaAmount,
    #[msg("Insufficient treasury balance for RWA investment")]
    InsufficientTreasury,
    #[msg("No RWA yield available to claim")]
    NoRwaYield,

    // ── Yield / Snapshot ───────────────────────────────────────────────────
    #[msg("Snapshot frequency not met")]
    SnapshotFrequencyNotMet,
    #[msg("Yield snapshot not ready for distribution")]
    YieldSnapshotNotReady,
    #[msg("No yield available to claim")]
    NoYieldToClaim,
    #[msg("Insufficient yield balance")]
    InsufficientYield,

    // ── Distribution ───────────────────────────────────────────────────────
    #[msg("Distribution percentages must sum to 100")]
    InvalidDistributionPercentages,

    // ── General ────────────────────────────────────────────────────────────
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}
