use anchor_lang::prelude::*;

// ============================================================================
// ACCOUNT STATE — Ecosystem Token
// All math uses checked arithmetic. No silent overflows.
// ============================================================================

/// Global launchpad configuration
#[account]
pub struct LaunchpadState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub vault: Pubkey,
    pub tax_vault: Pubkey,
    pub mint_authority: Pubkey,
    pub total_tokens_minted: u64,
    pub total_usdc_raised: u64,
    pub paused: bool,
    /// Morpho investment is paused until admin approves a pool
    pub investment_pending_approval: bool,
    pub bump: u8,
}

/// Global yield configuration — drives auto-yield for all holders
#[account]
pub struct YieldConfig {
    /// Grows monotonically; represents cumulative yield per token unit
    /// Stored as fixed-point with YIELD_INDEX_PRECISION decimals
    pub global_yield_index: u128,
    /// Normal holder APY in basis points (e.g. 4500 = 45%)
    pub normal_yield_bps: u64,
    /// Campaign holder APY in basis points (e.g. 7000 = 70%)
    pub campaign_yield_bps: u64,
    /// Campaign duration in seconds (default: 6 months = 15_552_000)
    pub campaign_duration_seconds: i64,
    /// Total USDC invested in Morpho
    pub morpho_invested_usdc: u64,
    /// Total yield earned from Morpho to date
    pub morpho_yield_earned: u64,
    /// Timestamp of last index update
    pub last_index_update: i64,
    /// 80% of pool is invested (800 = 80%)
    pub investment_ratio_bps: u64,
    pub bump: u8,
}

/// Per-holder state — created on first mint, auto-tracks yield
#[account]
pub struct HolderInfo {
    pub user: Pubkey,
    /// Snapshot of global_yield_index when user last claimed/bought
    pub yield_index_snapshot: u128,
    /// Unclaimed yield accumulated so far
    pub unclaimed_yield: u64,
    /// Timestamp when campaign yield period expires (0 if non-campaign)
    pub campaign_expires_at: i64,
    /// Whether this holder entered via launchpad (lower fee tier)
    pub is_launchpad_investor: bool,
    /// Original launchpad purchase amount (for fee tier calculation)
    pub launchpad_purchased_amount: u64,
    /// Total tokens currently held (tracked for yield calculation)
    pub token_balance: u64,
    pub bump: u8,
}

/// Unstaking request — tracks 48 business-hour countdown
#[account]
pub struct UnstakingRequest {
    pub user: Pubkey,
    pub amount: u64,
    pub usdc_to_return: u64,
    pub requested_at: i64,
    pub completed: bool,
    pub emergency_redeemed: bool,
    pub bump: u8,
}

/// Morpho investment pool candidate
/// Admin must approve before any USDC is invested
#[account]
pub struct MorphoPool {
    /// The Morpho pool/vault address (on-chain identifier)
    pub pool_address: Pubkey,
    /// Human-readable name (e.g. "USDC Isolated Vault")
    pub pool_name: [u8; 64],
    /// Pool type classification
    pub pool_type: MorphoPoolType,
    /// Current APY in basis points
    pub apy_bps: u64,
    /// Admin has approved this pool for investment
    pub approved: bool,
    /// Currently active (receiving new investment)
    pub is_active: bool,
    /// Amount currently invested in this pool
    pub invested_usdc: u64,
    /// Timestamp of admin approval
    pub approved_at: i64,
    pub bump: u8,
}

/// Morpho pool eligibility type — only these 3 types are eligible
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MorphoPoolType {
    /// Stablecoin-Isolated Vaults — single asset, no cross-collateral risk
    StablecoinIsolatedVault,
    /// Fixed-Asset Collateral Tiers — overcollateralized with fixed assets
    FixedAssetCollateralTier,
    /// 100% Cash/Cash-Equivalent Backed Pools — T-bills, money market
    CashEquivalentBacked,
}

/// Treasury vault — holds protocol USDC reserves
#[account]
pub struct TreasuryVault {
    pub authority: Pubkey,
    pub total_usdc: u64,
    pub total_deposited: u64,
    pub total_yields_earned: u64,
    pub total_yields_distributed: u64,
    /// USDC currently invested in Morpho
    pub morpho_balance: u64,
    /// Total yield from Morpho
    pub morpho_yield_earned: u64,
    /// Liquid USDC (not invested) = 20% kept for redemptions
    pub liquid_usdc: u64,
    pub last_update: i64,
    pub bump: u8,
}

/// Revenue split configuration — admin can update ratios
#[account]
pub struct RevenueDistribution {
    pub authority: Pubkey,
    /// % of yield going to coin holders (basis points, e.g. 2500 = 25%)
    pub holder_share_bps: u64,
    /// % of yield going to marketing agency
    pub marketing_share_bps: u64,
    /// % of yield going to asset manager
    pub asset_manager_share_bps: u64,
    /// % of yield going to protocol
    pub protocol_share_bps: u64,
    /// Marketing wallet address
    pub marketing_address: Pubkey,
    /// Asset manager wallet address
    pub asset_manager_address: Pubkey,
    /// Protocol/owner wallet address
    pub protocol_address: Pubkey,
    /// 80% of pool is invested (800 bps = 80%)
    pub investment_ratio_bps: u64,
    pub bump: u8,
}

/// Yield snapshot for distribution records
#[account]
pub struct YieldSnapshot {
    pub snapshot_id: u64,
    pub total_supply: u64,
    pub yield_amount: u64,
    pub snapshot_ts: i64,
    pub is_distributed: bool,
    pub bump: u8,
}

/// Backward-compatible staking info (kept for any existing stakers)
#[account]
pub struct StakingInfo {
    pub user: Pubkey,
    pub staked_amount: u64,
    pub staked_at: i64,
    pub last_claim_ts: i64,
    pub last_claim: i64,
    pub total_yield_claimed: u64,
    pub bump: u8,
}

/// User tier info (kept for Tier2 vesting enforcement)
#[account]
pub struct UserTierInfo {
    pub user: Pubkey,
    pub is_tier2: bool,
    pub tokens_purchased: u64,
    pub total_tokens_minted: u64,
    pub usdc_paid: u64,
    pub discount_pct: u64,
    pub vesting_start: i64,
    pub bump: u8,
}

/// VestingSchedule (kept for Tier2 vesting)
#[account]
pub struct VestingSchedule {
    pub user: Pubkey,
    pub total_amount: u64,
    pub start_ts: i64,
    pub start_time: i64,
    pub duration_seconds: i64,
    pub end_time: i64,
    pub claimed_amount: u64,
    pub bump: u8,
}

// ============================================================================
// YIELD STRATEGY STATE (sUSDS + sUSDe)
// ============================================================================

/// Tracks the USDC → USDS → sUSDS position (Sky Protocol)
#[account]
pub struct SkyPosition {
    pub authority: Pubkey,
    /// Total USDC allocated to this strategy
    pub usdc_deposited: u64,
    /// Current sUSDS balance (reported by keeper)
    pub susds_balance: u64,
    /// Total yield earned in USDC equivalent
    pub total_yield_earned: u64,
    /// Current APY in basis points (updated by keeper)
    pub current_apy_bps: u64,
    /// Timestamp of last keeper report
    pub last_update_ts: i64,
    /// Current status of position
    pub status: crate::yield_strategy::StrategyStatus,
    pub bump: u8,
}

/// Tracks the USDT → USDe → sUSDe position (Ethena via Meteora)
#[account]
pub struct EthenaPosition {
    pub authority: Pubkey,
    /// Total USDT allocated to this strategy
    pub usdt_deposited: u64,
    /// Current sUSDe balance (reported by keeper)
    pub susde_balance: u64,
    /// Total yield earned in USDT equivalent
    pub total_yield_earned: u64,
    /// Current APY in basis points (updated by keeper, variable)
    pub current_apy_bps: u64,
    /// Timestamp of last keeper report
    pub last_update_ts: i64,
    /// Current status of position
    pub status: crate::yield_strategy::StrategyStatus,
    pub bump: u8,
}

/// Combined pool state — tracks both USDC and USDT pools
#[account]
pub struct PoolState {
    pub authority: Pubkey,
    // USDC pool
    pub total_usdc: u64,
    pub liquid_usdc: u64,       // 20% kept liquid
    pub invested_usdc: u64,     // 80% in sUSDS
    // USDT pool
    pub total_usdt: u64,
    pub liquid_usdt: u64,       // 20% kept liquid
    pub invested_usdt: u64,     // 80% in sUSDe
    // Combined yield
    pub total_yield_distributed: u64,
    pub last_update_ts: i64,
    pub bump: u8,
}
