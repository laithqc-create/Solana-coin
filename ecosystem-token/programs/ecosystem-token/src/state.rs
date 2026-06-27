use anchor_lang::prelude::*;

#[account]
pub struct LaunchpadState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub vault: Pubkey,
    pub tax_vault: Pubkey,
    pub mint_authority: Pubkey,
    pub tier1_price: u64,
    pub current_discount: u8,
    pub discount_threshold_1: u64,
    pub discount_threshold_2: u64,
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
    pub rwa_invested_usdc: u64,
    pub rwa_yield_earned: u64,
    pub last_investment_ts: i64,
    pub snapshot_frequency: i64,
    pub last_snapshot: i64,
    pub next_snapshot: i64,
    pub bump: u8,
}

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

/// Created when a user requests to unstake.
/// Tracks the 48-business-hour countdown, pausing on weekends.
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

#[account]
pub struct YieldSnapshot {
    pub snapshot_id: u64,
    pub total_staked: u64,
    pub yield_amount: u64,
    pub tax_collected: u64,
    pub snapshot_ts: i64,
    pub snapshot_time: i64,
    pub is_distributed: bool,
    pub bump: u8,
}

#[account]
pub struct TreasuryVault {
    pub authority: Pubkey,
    pub total_usdc: u64,
    pub total_deposited: u64,
    pub total_yields_earned: u64,
    pub total_yields_distributed: u64,
    pub rwa_balance: u64,
    pub rwa_yield_earned: u64,
    pub aave_position: u64,
    pub last_rwa_sync: i64,
    pub last_aave_claim: i64,
    pub bump: u8,
}

#[account]
pub struct RevenueDistribution {
    pub authority: Pubkey,
    pub user_pct: u64,
    pub user_percentage: u64,
    pub marketing_pct: u64,
    pub marketing_percentage: u64,
    pub manager_pct: u64,
    pub asset_manager_percentage: u64,
    pub owner_pct: u64,
    pub owner_percentage: u64,
    pub marketing_address: Pubkey,
    pub asset_manager_address: Pubkey,
    pub owner_address: Pubkey,
    pub total_distributed: u64,
    pub bump: u8,
}

#[account]
pub struct Tier2Whitelist {
    pub authority: Pubkey,
    pub user: Pubkey,
    pub is_whitelisted: bool,
    pub discount_pct: u64,
    pub whitelisted_at: i64,
    pub bump: u8,
}

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
