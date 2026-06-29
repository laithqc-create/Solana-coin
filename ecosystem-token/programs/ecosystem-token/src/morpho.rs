use anchor_lang::prelude::*;
use crate::state::MorphoPoolType;

// ============================================================================
// MORPHO INVESTMENT MODULE
// Only 3 pool types are eligible for investment:
//   1. Stablecoin-Isolated Vaults
//   2. Fixed-Asset Collateral Tiers
//   3. 100% Cash/Cash-Equivalent Backed Pools
//
// CROSS-CHAIN NOTE: Morpho Protocol lives on Ethereum/Base. Investment
// execution is handled by an off-chain keeper that bridges USDC via
// Wormhole/LayerZero. This contract records intent and state; the keeper
// reports execution results back on-chain.
//
// ADMIN APPROVAL REQUIRED: No pool can receive investment until an admin
// explicitly approves it in the dashboard. Investment is blocked until then.
// ============================================================================

/// Precision multiplier for yield index (1e12)
pub const YIELD_INDEX_PRECISION: u128 = 1_000_000_000_000;

/// Normal holder APY: 45%
pub const NORMAL_YIELD_BPS: u64 = 4_500;

/// Campaign holder APY: 70% (for first 6 months after purchase)
pub const CAMPAIGN_YIELD_BPS: u64 = 7_000;

/// Campaign duration: 6 months in seconds
pub const CAMPAIGN_DURATION_SECONDS: i64 = 15_552_000; // 6 * 30 * 24 * 3600

/// 80% of pool is invested, 20% kept liquid for redemptions
pub const INVESTMENT_RATIO_BPS: u64 = 8_000;

/// Seconds per year for APY calculation
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

// ============================================================================
// YIELD INDEX MODEL
// Works like Aave's liquidity index:
//   - global_yield_index grows over time based on APY
//   - Each holder snapshots the index at purchase time
//   - yield_owed = (current_index - snapshot) * balance / PRECISION
// ============================================================================

/// Calculate the updated global yield index
/// Called whenever the index needs to be refreshed (mint, burn, claim)
pub fn accrue_yield_index(
    current_index: u128,
    apy_bps: u64,
    seconds_elapsed: i64,
) -> u128 {
    if seconds_elapsed <= 0 {
        return current_index;
    }

    // delta = index * apy * time / (10000 * SECONDS_PER_YEAR)
    let delta = (current_index as u128)
        .saturating_mul(apy_bps as u128)
        .saturating_mul(seconds_elapsed as u128)
        .checked_div(10_000u128 * SECONDS_PER_YEAR as u128)
        .unwrap_or(0);

    current_index.saturating_add(delta)
}

/// Calculate yield owed to a holder since their last snapshot
pub fn calculate_holder_yield(
    current_index: u128,
    holder_snapshot: u128,
    token_balance: u64,
) -> u64 {
    if current_index <= holder_snapshot || token_balance == 0 {
        return 0;
    }

    let index_delta = current_index.saturating_sub(holder_snapshot);
    let yield_amount = (index_delta as u128)
        .saturating_mul(token_balance as u128)
        .checked_div(YIELD_INDEX_PRECISION)
        .unwrap_or(0);

    yield_amount.min(u64::MAX as u128) as u64
}

/// Determine APY for a holder based on campaign status
pub fn holder_apy_bps(
    campaign_expires_at: i64,
    now: i64,
) -> u64 {
    if campaign_expires_at > 0 && now < campaign_expires_at {
        CAMPAIGN_YIELD_BPS  // 70% for campaign holders within 6 months
    } else {
        NORMAL_YIELD_BPS    // 45% for all other holders
    }
}

// ============================================================================
// FEE TIER CALCULATION
// Three tiers:
//   1. Launchpad investor: 0.1% (10 bps) on original purchase amount
//      If amount > original purchase: excess charged at standard rate
//   2. Standard trader: 0.5% (50 bps) on full amount
//   3. Arbitrage bot (detected by pattern): 0.5% (50 bps)
//
// ALL fees go 100% to protocol treasury (no yield pool split for fees)
// ============================================================================

pub const LAUNCHPAD_FEE_BPS: u64 = 10;   // 0.1%
pub const STANDARD_FEE_BPS: u64 = 50;    // 0.5%

/// Result of fee calculation
pub struct FeeResult {
    pub total_fee: u64,
    pub launchpad_portion_fee: u64,
    pub excess_portion_fee: u64,
}

/// Calculate fee based on holder tier
///
/// - Launchpad investor (whitelisted):
///     amount ≤ original_purchase → fee = amount × 0.1%
///     amount > original_purchase → fee = (original × 0.1%) + (excess × 0.5%)
///
/// - Standard trader / non-whitelisted:
///     fee = amount × 0.5% (always)
pub fn calculate_fee(
    amount: u64,
    is_launchpad_investor: bool,
    original_purchase: u64,
) -> FeeResult {
    if is_launchpad_investor && original_purchase > 0 {
        if amount <= original_purchase {
            // Entire amount at launchpad rate (0.1%)
            let fee = amount
                .saturating_mul(LAUNCHPAD_FEE_BPS)
                .checked_div(10_000)
                .unwrap_or(0);
            FeeResult {
                total_fee: fee,
                launchpad_portion_fee: fee,
                excess_portion_fee: 0,
            }
        } else {
            // Split: original at 0.1%, excess at 0.5%
            let launchpad_fee = original_purchase
                .saturating_mul(LAUNCHPAD_FEE_BPS)
                .checked_div(10_000)
                .unwrap_or(0);
            let excess = amount.saturating_sub(original_purchase);
            let excess_fee = excess
                .saturating_mul(STANDARD_FEE_BPS)
                .checked_div(10_000)
                .unwrap_or(0);
            let total = launchpad_fee.saturating_add(excess_fee);
            FeeResult {
                total_fee: total,
                launchpad_portion_fee: launchpad_fee,
                excess_portion_fee: excess_fee,
            }
        }
    } else {
        // Standard / everyday trader / arbitrage bot: 0.5%
        let fee = amount
            .saturating_mul(STANDARD_FEE_BPS)
            .checked_div(10_000)
            .unwrap_or(0);
        FeeResult {
            total_fee: fee,
            launchpad_portion_fee: 0,
            excess_portion_fee: fee,
        }
    }
}

// ============================================================================
// MORPHO POOL ELIGIBILITY
// Before proposing a pool for admin approval, it must pass eligibility checks.
// Only pools matching one of the 3 approved types can be proposed.
// ============================================================================

/// Validate that a pool type meets investment criteria
pub fn is_eligible_pool_type(pool_type: &MorphoPoolType) -> bool {
    matches!(
        pool_type,
        MorphoPoolType::StablecoinIsolatedVault
            | MorphoPoolType::FixedAssetCollateralTier
            | MorphoPoolType::CashEquivalentBacked
    )
}

/// Calculate how much USDC should be invested based on 80% ratio
pub fn calculate_investment_amount(
    total_liquid_usdc: u64,
    investment_ratio_bps: u64,
) -> u64 {
    total_liquid_usdc
        .saturating_mul(investment_ratio_bps)
        .checked_div(10_000)
        .unwrap_or(0)
}

/// Calculate Morpho yield earned over a period
/// Uses the pool's reported APY
pub fn calculate_morpho_yield(
    invested_usdc: u64,
    pool_apy_bps: u64,
    seconds_elapsed: u64,
) -> u64 {
    if invested_usdc == 0 || seconds_elapsed == 0 {
        return 0;
    }

    let yield_amount = (invested_usdc as u128)
        .saturating_mul(pool_apy_bps as u128)
        .saturating_mul(seconds_elapsed as u128)
        .checked_div(10_000u128 * SECONDS_PER_YEAR as u128)
        .unwrap_or(0);

    yield_amount.min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launchpad_fee_within_purchase() {
        // Launchpad investor, amount ≤ original: 0.1% fee
        let result = calculate_fee(1_000_000, true, 2_000_000);
        assert_eq!(result.total_fee, 100); // 0.1% of 1_000_000
        assert_eq!(result.excess_portion_fee, 0);
    }

    #[test]
    fn test_launchpad_fee_excess() {
        // Launchpad investor, amount > original: split fee
        let result = calculate_fee(3_000_000, true, 1_000_000);
        let expected_launchpad = 100; // 0.1% of 1_000_000
        let expected_excess = 1_000; // 0.5% of 2_000_000
        assert_eq!(result.launchpad_portion_fee, expected_launchpad);
        assert_eq!(result.excess_portion_fee, expected_excess);
        assert_eq!(result.total_fee, expected_launchpad + expected_excess);
    }

    #[test]
    fn test_standard_fee() {
        // Non-launchpad: always 0.5%
        let result = calculate_fee(1_000_000, false, 0);
        assert_eq!(result.total_fee, 500); // 0.5% of 1_000_000
    }

    #[test]
    fn test_campaign_yield() {
        let now = 1_700_000_000i64;
        let expires = now + 1000;
        assert_eq!(holder_apy_bps(expires, now), CAMPAIGN_YIELD_BPS); // 70%
    }

    #[test]
    fn test_normal_yield_after_campaign() {
        let now = 1_700_000_000i64;
        let expires = now - 1000; // expired
        assert_eq!(holder_apy_bps(expires, now), NORMAL_YIELD_BPS); // 45%
    }

    #[test]
    fn test_yield_index_accrual() {
        let index = YIELD_INDEX_PRECISION; // start at 1.0
        let one_year = SECONDS_PER_YEAR as i64;
        let new_index = accrue_yield_index(index, NORMAL_YIELD_BPS, one_year);
        // After 1 year at 45% APY, index should be ~1.45x
        let ratio = new_index * 100 / index;
        assert!(ratio >= 144 && ratio <= 146, "ratio was {}", ratio);
    }

    #[test]
    fn test_pool_eligibility() {
        assert!(is_eligible_pool_type(&MorphoPoolType::StablecoinIsolatedVault));
        assert!(is_eligible_pool_type(&MorphoPoolType::FixedAssetCollateralTier));
        assert!(is_eligible_pool_type(&MorphoPoolType::CashEquivalentBacked));
    }
}
