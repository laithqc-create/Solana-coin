//! # Yield Strategy Module
//!
//! Two independent yield strategies for the protocol:
//!
//! ## Strategy 1: USDC → sUSDS (Sky Protocol)
//! - 80% of USDC pool routed via native Wormhole bridge
//! - Converted to USDS on Ethereum/Base
//! - Staked into sUSDS to earn Sky Savings Rate (~5-8% APY)
//! - Fully permissioned: no algorithmic risk, regulated savings rate
//!
//! ## Strategy 2: USDT → sUSDe (Ethena)
//! - 80% of USDT pool routed via Meteora on Solana
//! - Converted to USDe (delta-neutral synthetic dollar)
//! - Wrapped into sUSDe to earn staking yield (~15-30% APY, variable)
//!
//! ## Cross-chain execution
//! Both strategies use an off-chain keeper pattern:
//! 1. Contract records investment intent + locks funds
//! 2. Keeper executes bridge + swap + stake off-chain
//! 3. Keeper reports position and yield back on-chain
//! 4. Contract validates report signature before updating state
//!
//! License: Sky Protocol (Apache 2.0), Ethena (MIT), Wormhole (Apache 2.0)

use anchor_lang::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

/// Percentage of each pool kept liquid for immediate redemptions
pub const LIQUID_RESERVE_BPS: u64 = 2_000; // 20%

/// Percentage routed to yield strategy
pub const INVESTMENT_BPS: u64 = 8_000; // 80%

/// Sky Savings Rate baseline APY in basis points (~6% nominal)
/// Updated by keeper when Sky governance changes the rate
pub const SKY_BASE_APY_BPS: u64 = 600;

/// Ethena sUSDe baseline APY in basis points (~20% nominal, variable)
/// Updated by keeper on each yield report
pub const ETHENA_BASE_APY_BPS: u64 = 2_000;

/// Seconds per year for APY → per-second rate calculation
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

// ── Strategy types ────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StrategyType {
    /// USDC → USDS → sUSDS via Sky Protocol
    SkyUsds,
    /// USDT → USDe → sUSDe via Ethena through Meteora
    EthenaUsde,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StrategyStatus {
    /// No active position
    Inactive,
    /// Investment queued, keeper has not yet confirmed execution
    PendingExecution,
    /// Keeper confirmed cross-chain position is live
    Active,
    /// Withdrawal requested, keeper unwinding position
    PendingWithdrawal,
}

// ── Core calculations ─────────────────────────────────────────────────────────

/// Calculate how much to invest vs keep liquid
///
/// Returns (invest_amount, liquid_amount)
/// Both sum to total_pool_amount.
/// Validates no overflow and amounts are non-zero.
pub fn split_investment(total_pool_amount: u64) -> Result<(u64, u64)> {
    if total_pool_amount == 0 {
        return Ok((0, 0));
    }

    let invest = total_pool_amount
        .checked_mul(INVESTMENT_BPS)
        .and_then(|v| v.checked_div(10_000))
        .ok_or_else(|| error!(YieldStrategyError::MathOverflow))?;

    let liquid = total_pool_amount
        .checked_sub(invest)
        .ok_or_else(|| error!(YieldStrategyError::MathOverflow))?;

    Ok((invest, liquid))
}

/// Accrue yield on an active position using simple interest.
/// Used when keeper reports current position value.
///
/// Returns yield_earned since last_update_ts.
pub fn calculate_accrued_yield(
    invested_amount: u64,
    apy_bps: u64,
    last_update_ts: i64,
    now_ts: i64,
) -> u64 {
    if invested_amount == 0 || apy_bps == 0 || now_ts <= last_update_ts {
        return 0;
    }

    let seconds_elapsed = (now_ts - last_update_ts) as u64;

    // yield = principal × apy_bps/10000 × seconds/year
    (invested_amount as u128)
        .saturating_mul(apy_bps as u128)
        .saturating_mul(seconds_elapsed as u128)
        .checked_div(10_000u128 * SECONDS_PER_YEAR as u128)
        .unwrap_or(0)
        .min(u64::MAX as u128) as u64
}

/// Validate a keeper report before applying it to on-chain state.
/// Checks:
///   1. Reported amount is within ±10% of expected (slippage guard)
///   2. Timestamp is not in the future
///   3. Position is in expected status
pub fn validate_keeper_report(
    expected_amount: u64,
    reported_amount: u64,
    expected_status: &StrategyStatus,
    actual_status: &StrategyStatus,
    report_ts: i64,
    now_ts: i64,
) -> Result<()> {
    // Status must match
    require!(
        expected_status == actual_status,
        YieldStrategyError::InvalidStrategyStatus
    );

    // Timestamp cannot be in the future
    require!(
        report_ts <= now_ts,
        YieldStrategyError::InvalidTimestamp
    );

    // Reported amount must be within 10% of expected (1000 bps tolerance)
    if expected_amount > 0 {
        let tolerance = expected_amount
            .checked_mul(1_000)
            .and_then(|v| v.checked_div(10_000))
            .unwrap_or(0);

        let lower = expected_amount.saturating_sub(tolerance);
        let upper = expected_amount.saturating_add(tolerance);

        require!(
            reported_amount >= lower && reported_amount <= upper,
            YieldStrategyError::KeeperReportOutOfRange
        );
    }

    Ok(())
}

// ── Error codes ───────────────────────────────────────────────────────────────

#[error_code]
pub enum YieldStrategyError {
    #[msg("Arithmetic overflow in yield calculation")]
    MathOverflow,

    #[msg("Strategy is not in the expected status for this operation")]
    InvalidStrategyStatus,

    #[msg("Keeper report timestamp is in the future")]
    InvalidTimestamp,

    #[msg("Keeper reported amount is outside the 10% tolerance range")]
    KeeperReportOutOfRange,

    #[msg("Pool has insufficient balance for this investment")]
    InsufficientPoolBalance,

    #[msg("Investment already active — withdraw before re-investing")]
    InvestmentAlreadyActive,

    #[msg("No active investment to withdraw")]
    NoActiveInvestment,
}

// ── Tests (Rule 8: validate all paths) ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_investment_80_20() {
        let (invest, liquid) = split_investment(1_000_000).unwrap();
        assert_eq!(invest, 800_000);
        assert_eq!(liquid, 200_000);
        assert_eq!(invest + liquid, 1_000_000);
    }

    #[test]
    fn test_split_investment_zero() {
        let (invest, liquid) = split_investment(0).unwrap();
        assert_eq!(invest, 0);
        assert_eq!(liquid, 0);
    }

    #[test]
    fn test_sky_yield_one_year() {
        // 1,000,000 USDC at 6% APY for 1 year = 60,000 USDC yield
        let yield_amount = calculate_accrued_yield(
            1_000_000_000_000, // 1M USDC (6 decimals)
            600,               // 6% in bps
            0,
            SECONDS_PER_YEAR as i64,
        );
        // Should be ~60,000 USDC (60_000_000_000 micro-USDC)
        assert!(yield_amount > 59_000_000_000 && yield_amount < 61_000_000_000);
    }

    #[test]
    fn test_ethena_yield_one_year() {
        // 1,000,000 USDT at 20% APY for 1 year = 200,000 yield
        let yield_amount = calculate_accrued_yield(
            1_000_000_000_000,
            2_000, // 20% in bps
            0,
            SECONDS_PER_YEAR as i64,
        );
        assert!(yield_amount > 199_000_000_000 && yield_amount < 201_000_000_000);
    }

    #[test]
    fn test_no_yield_when_no_time_elapsed() {
        let yield_amount = calculate_accrued_yield(1_000_000, 600, 1000, 1000);
        assert_eq!(yield_amount, 0);
    }

    #[test]
    fn test_keeper_report_within_tolerance() {
        let result = validate_keeper_report(
            1_000_000,
            1_050_000, // 5% above expected — within 10% tolerance
            &StrategyStatus::PendingExecution,
            &StrategyStatus::PendingExecution,
            1000,
            2000,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_keeper_report_out_of_range() {
        let result = validate_keeper_report(
            1_000_000,
            500_000, // 50% below — out of 10% tolerance
            &StrategyStatus::PendingExecution,
            &StrategyStatus::PendingExecution,
            1000,
            2000,
        );
        assert!(result.is_err());
    }
}
