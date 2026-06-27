use anchor_lang::prelude::*;

/// LaunchpadState: Global launchpad configuration and USDC vault
#[account]
pub struct LaunchpadState {
    pub token_mint: Pubkey,              // SPL Token mint
    pub usdc_mint: Pubkey,               // USDC mint
    pub vault: Pubkey,                   // Collateral vault ATA (USDC)
    pub tax_vault: Pubkey,               // Tax vault ATA (USDC)
    pub mint_authority: Pubkey,          // Hardcoded to vault PDA
    pub authority: Pubkey,               // Multisig authority
    pub total_usdc_raised: u64,          // Total USDC collected
    pub total_tokens_minted: u64,        // Total tokens minted (Tier 1 + Tier 2)
    pub tier1_price: u64,                // Base price: 1 USDC = 1e6 tokens
    pub current_discount: u8,            // Current tier discount (50, 40, 30)
    pub discount_threshold_1: u64,       // USDC raised threshold for tier 2 (1M)
    pub discount_threshold_2: u64,       // USDC raised threshold for tier 3 (2M)
    pub paused: bool,                    // Emergency pause flag
    pub bump: u8,
}

/// YieldConfig: Global yield snapshot configuration
#[account]
pub struct YieldConfig {
    pub snapshot_frequency: i64,         // Seconds between snapshots (604800 = 7 days)
    pub last_snapshot: i64,              // Timestamp of last snapshot
    pub next_snapshot: i64,              // When next snapshot is due
    pub bump: u8,
}

/// UserTierInfo: Per-user tier and vesting info
#[account]
pub struct UserTierInfo {
    pub user: Pubkey,
    pub is_tier2: bool,                  // true = Tier 2 (vested), false = Tier 1 (free)
    pub vesting_schedule: Option<VestingSchedule>,
    pub total_tokens_minted: u64,        // Lifetime tokens minted
    pub bump: u8,
}

/// VestingSchedule: 12-month linear vesting for Tier 2
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VestingSchedule {
    pub start_time: i64,                 // Unix timestamp vesting starts
    pub end_time: i64,                   // Unix timestamp vesting ends (start + 365 days)
    pub total_amount: u64,               // Total tokens vested
    pub claimed_amount: u64,             // Already claimed (unused, for future)
}

impl VestingSchedule {
    pub fn vested_amount(&self, now: i64) -> u64 {
        if now >= self.end_time {
            return self.total_amount;
        }
        if now <= self.start_time {
            return 0;
        }
        let elapsed = (now - self.start_time) as u128;
        let total_duration = (self.end_time - self.start_time) as u128;
        ((self.total_amount as u128 * elapsed) / total_duration) as u64
    }

    pub fn locked_amount(&self, now: i64) -> u64 {
        self.total_amount.saturating_sub(self.vested_amount(now))
    }
}

/// StakingInfo: Per-user staking data
#[account]
pub struct StakingInfo {
    pub user: Pubkey,
    pub staked_amount: u64,              // Currently staked tokens
    pub staked_at: i64,                  // When user first staked
    pub last_claim: i64,                 // Last yield claim timestamp
    pub total_yield_claimed: u64,        // Lifetime yield claimed
    pub bump: u8,
}

/// YieldSnapshot: Weekly snapshot of yield distribution
#[account]
pub struct YieldSnapshot {
    pub snapshot_time: i64,              // When snapshot was created
    pub total_staked: u64,               // Total tokens staked during this period
    pub tax_collected: u64,              // USDC collected from taxes (30% yield vault)
    pub is_distributed: bool,            // Has yield been distributed
    pub bump: u8,
}

/// TreasuryVault: Treasury management and Aave integration
#[account]
pub struct TreasuryVault {
    pub aave_position: u64,              // Current USDC deposited in Aave
    pub total_deposited: u64,            // Lifetime USDC deposits to Aave
    pub total_yields_earned: u64,        // Total interest earned from Aave
    pub total_yields_distributed: u64,   // Total yields distributed
    pub last_aave_claim: i64,            // Last yield claim timestamp
    pub authority: Pubkey,               // Multisig authority
    pub bump: u8,
}

/// RevenueDistribution: Revenue allocation percentages
#[account]
pub struct RevenueDistribution {
    pub user_percentage: u8,             // Percentage to users (default 40)
    pub marketing_percentage: u8,        // Percentage to marketing (default 20)
    pub asset_manager_percentage: u8,    // Percentage to asset manager (default 20)
    pub owner_percentage: u8,            // Percentage to owner (default 20)
    pub marketing_address: Pubkey,       // Marketing wallet address
    pub asset_manager_address: Pubkey,   // Asset manager wallet address
    pub owner_address: Pubkey,           // Owner wallet address
    pub total_distributed: u64,          // Total USDC distributed
    pub authority: Pubkey,               // Multisig authority
    pub bump: u8,
}

/// Tier2Whitelist: Per-user Tier 2 whitelist status
#[account]
pub struct Tier2Whitelist {
    pub user: Pubkey,
    pub is_whitelisted: bool,
    pub whitelisted_at: i64,
    pub authority: Pubkey,
    pub bump: u8,
}
