use anchor_lang::prelude::*;
use crate::errors::EcosystemError;

// ============================================================================
// RUSDY (Real World Asset) Investment Module
// Staked USDC is invested into RUSDY via Jupiter DCA routing
// RUSDY is a regulated RWA token backed by real-world assets
// Unstaking requires 48 business hours (weekends excluded) for regulatory compliance
// ============================================================================

/// RUSDY token mint on Solana mainnet
/// Update this to the actual RUSDY mint address before mainnet deploy
pub const RUSDY_MINT: &str = "RuSDyTokenMintAddress1111111111111111111111";

/// Simulated RUSDY annual yield (~5% APY for RWA tokens)
pub const RUSDY_APY_BPS: u64 = 500; // 5.00% in basis points

/// 48 business hours in seconds (2 days × 24 hours × 3600 seconds)
pub const UNSTAKE_BUSINESS_SECONDS: i64 = 172_800;

/// Day of week constants
const SUNDAY: i64 = 0;
const SATURDAY: i64 = 6;

// ============================================================================
// BUSINESS HOURS CALCULATION
// Calculates elapsed business seconds (Mon-Fri only, weekends paused)
// ============================================================================

/// Returns the day of week for a given Unix timestamp
/// 0 = Sunday, 1 = Monday, ..., 5 = Friday, 6 = Saturday
/// Jan 1 1970 was a Thursday (day 4)
pub fn day_of_week(ts: i64) -> i64 {
    let days_since_epoch = ts / 86_400;
    (days_since_epoch + 4).rem_euclid(7)
}

/// Returns true if the given timestamp falls on a weekend
pub fn is_weekend(ts: i64) -> bool {
    let dow = day_of_week(ts);
    dow == SUNDAY || dow == SATURDAY
}

/// Calculates how many business seconds have elapsed between start and now
/// Weekends (Saturday and Sunday) are excluded from the count
pub fn business_seconds_elapsed(start_ts: i64, now_ts: i64) -> i64 {
    if now_ts <= start_ts {
        return 0;
    }

    let mut elapsed: i64 = 0;
    let mut cursor = start_ts;

    while cursor < now_ts {
        // How many seconds until end of this day
        let seconds_remaining_in_day = 86_400 - (cursor % 86_400);
        let chunk = seconds_remaining_in_day.min(now_ts - cursor);

        // Only count business days
        if !is_weekend(cursor) {
            elapsed += chunk;
        }

        cursor += chunk;
    }

    elapsed
}

/// Returns human-readable countdown message for the dashboard
pub fn unstake_status_message(
    requested_at: i64,
    now_ts: i64,
) -> UnstakeStatus {
    let elapsed = business_seconds_elapsed(requested_at, now_ts);
    let remaining = (UNSTAKE_BUSINESS_SECONDS - elapsed).max(0);

    if remaining == 0 {
        return UnstakeStatus::Ready;
    }

    let is_currently_weekend = is_weekend(now_ts);
    let remaining_hours = remaining / 3600;
    let remaining_minutes = (remaining % 3600) / 60;

    UnstakeStatus::Pending {
        remaining_business_seconds: remaining,
        remaining_hours,
        remaining_minutes,
        paused_for_weekend: is_currently_weekend,
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum UnstakeStatus {
    /// Funds are ready to be claimed
    Ready,
    /// Countdown still running
    Pending {
        remaining_business_seconds: i64,
        remaining_hours: i64,
        remaining_minutes: i64,
        /// Timer is currently paused because it is a weekend
        paused_for_weekend: bool,
    },
}

// ============================================================================
// RUSDY YIELD CALCULATION
// Simulates RWA yield. In production this is fetched from the RUSDY oracle.
// ============================================================================

/// Calculate RUSDY yield earned on principal over a time period
/// Uses simple interest: yield = principal * apy_bps/10000 * (days/365)
pub fn calculate_rwa_yield(
    principal_usdc: u64,
    seconds_elapsed: u64,
) -> Result<u64> {
    if principal_usdc == 0 || seconds_elapsed == 0 {
        return Ok(0);
    }

    // Fixed-point integer math (no external decimal/float libraries):
    //   yield = principal * apy_bps * seconds_elapsed / (10_000 * seconds_per_year)
    // All multiplication happens before the single final division, which is
    // both more precise (avoids the two separate intermediate roundings the
    // old Decimal-based version performed) and safer (checked arithmetic
    // throughout — no silent `unwrap_or(0)` swallowing an overflow).
    const SECONDS_PER_YEAR: u128 = 365u128 * 24 * 3600;

    let numerator = (principal_usdc as u128)
        .checked_mul(RUSDY_APY_BPS as u128)
        .ok_or(EcosystemError::MathOverflow)?
        .checked_mul(seconds_elapsed as u128)
        .ok_or(EcosystemError::MathOverflow)?;

    let denominator = 10_000u128
        .checked_mul(SECONDS_PER_YEAR)
        .ok_or(EcosystemError::MathOverflow)?;

    let yield_amount = numerator
        .checked_div(denominator)
        .ok_or(EcosystemError::MathOverflow)?;

    Ok(u64::try_from(yield_amount).map_err(|_| EcosystemError::MathOverflow)?)
}

// ============================================================================
// JUPITER DCA ROUTING
// In production, the actual Jupiter DCA swap is executed by an off-chain keeper.
// The smart contract records intent and the keeper reports execution.
// This prevents MEV and ensures best price via DCA across multiple blocks.
// ============================================================================

/// Records a pending RUSDY investment instruction for the off-chain keeper
/// Keeper will execute Jupiter DCA swap: USDC → RUSDY
pub fn record_rwa_investment(
    treasury_usdc_balance: u64,
    investment_pct: u64, // percentage of treasury to invest (e.g. 70)
) -> u64 {
    // Calculate USDC amount to route through Jupiter DCA → RUSDY
    treasury_usdc_balance
        .checked_mul(investment_pct)
        .unwrap_or(0)
        .checked_div(100)
        .unwrap_or(0)
}

/// Validates a slippage-safe DeFi redemption
/// Returns the minimum USDC expected after slippage tolerance
pub fn calculate_emergency_redemption(
    rusdy_amount: u64,
    slippage_bps: u64, // e.g. 100 = 1%
) -> u64 {
    let after_slippage = rusdy_amount
        .checked_mul(10_000u64.saturating_sub(slippage_bps))
        .unwrap_or(0)
        .checked_div(10_000)
        .unwrap_or(0);
    after_slippage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_of_week() {
        // Jan 1 2024 00:00:00 UTC = 1704067200 → Monday
        assert_eq!(day_of_week(1704067200), 1);
        // Jan 6 2024 = Saturday
        assert_eq!(day_of_week(1704499200), 6);
        // Jan 7 2024 = Sunday
        assert_eq!(day_of_week(1704585600), 0);
    }

    #[test]
    fn test_business_seconds_weekend_pause() {
        // Start on Friday 6pm, end on Monday 6am
        // Only Friday evening (6hrs) + Monday morning (6hrs) should count
        let friday_6pm: i64 = 1704412800; // rough Friday timestamp
        let monday_6am: i64 = friday_6pm + (3 * 86_400); // 3 days later
        let elapsed = business_seconds_elapsed(friday_6pm, monday_6am);
        // Should be less than 72 hours (3 days) since weekend is excluded
        assert!(elapsed < 3 * 86_400);
    }

    #[test]
    fn test_rwa_yield() {
        // 1000 USDC at 5% APY for 365 days = 50 USDC yield
        let yield_1yr = calculate_rwa_yield(1_000_000_000, 365 * 24 * 3600).unwrap();
        // Should be approximately 50 USDC (50_000_000 microUSDC)
        assert!(yield_1yr > 49_000_000 && yield_1yr < 51_000_000);
    }
}
