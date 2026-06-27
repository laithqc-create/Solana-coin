use anchor_lang::prelude::*;

// ============================================================================
// ACCOUNT STATE DEFINITIONS
// ============================================================================

#[account]
pub struct LaunchpadState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub total_tokens_minted: u64,
    pub total_usdc_raised: u64,
    pub tier2_usdc_raised: u64,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct YieldConfig {
    pub total_staked: u64,
    pub total_yield_distributed: u64,
    pub tax_yield_accumulated: u64,
    pub rwa_invested_usdc: u64,      // USDC currently invested in RUSDY
    pub rwa_yield_earned: u64,       // Total yield earned from RUSDY
    pub last_investment_ts: i64,     // Last time USDC was routed to RUSDY
    pub next_snapshot: i64,
    pub bump: u8,
}

#[account]
pub struct UserTierInfo {
    pub user: Pubkey,
    pub is_tier2: bool,
    pub tokens_purchased: u64,
    pub usdc_paid: u64,
    pub discount_pct: u64,
    pub vesting_start: i64,
    pub bump: u8,
}

#[account]
pub struct StakingInfo {
    pub user: Pubkey,
    pub staked_amount: u64,
    pub staked_at: i64,
    pub last_claim_ts: i64,
    pub bump: u8,
}

/// Created when a user requests to unstake.
/// Tracks the 48-business-hour countdown, pausing on weekends.
#[account]
pub struct UnstakingRequest {
    pub user: Pubkey,
    pub amount: u64,                    // Token amount being unstaked
    pub usdc_to_return: u64,            // USDC equivalent (from RUSDY redemption)
    pub requested_at: i64,              // Unix timestamp when request was made
    pub completed: bool,                // True once USDC returned to user
    pub emergency_redeemed: bool,       // True if user chose DeFi emergency exit
    pub bump: u8,
}

#[account]
pub struct YieldSnapshot {
    pub snapshot_id: u64,
    pub total_staked: u64,
    pub yield_amount: u64,
    pub snapshot_ts: i64,
    pub is_distributed: bool,
    pub bump: u8,
}

#[account]
pub struct TreasuryVault {
    pub total_usdc: u64,
    pub total_yields_earned: u64,
    pub total_yields_distributed: u64,
    pub rwa_balance: u64,               // RUSDY balance (in USDC equivalent)
    pub last_rwa_sync: i64,             // Last time RWA balance was synced
    pub bump: u8,
}

#[account]
pub struct RevenueDistribution {
    pub user_pct: u64,
    pub marketing_pct: u64,
    pub manager_pct: u64,
    pub owner_pct: u64,
    pub bump: u8,
}

#[account]
pub struct Tier2Whitelist {
    pub user: Pubkey,
    pub is_whitelisted: bool,
    pub discount_pct: u64,
    pub bump: u8,
}

#[account]
pub struct VestingSchedule {
    pub user: Pubkey,
    pub total_amount: u64,
    pub start_ts: i64,
    pub duration_seconds: i64,
    pub claimed_amount: u64,
    pub bump: u8,
}
