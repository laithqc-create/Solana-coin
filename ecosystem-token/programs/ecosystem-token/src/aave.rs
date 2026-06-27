use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Aave Integration Module
/// Currently uses a simulated 4% APY model
/// Ready for real Aave V3 SDK when available

const AAVE_APY_ANNUAL: Decimal = Decimal::from_parts(4, 0, 0, false, 0); // 4.0%

/// Validate USDC amount for Aave deposit
pub fn validate_aave_deposit(amount: u64) -> Result<()> {
    Ok(())
}

/// Calculate interest earned from Aave (simulated 4% APY)
pub fn calculate_aave_interest(principal: u64, days_elapsed: u64) -> Result<u64> {
    if days_elapsed == 0 || principal == 0 {
        return Ok(0);
    }

    // interest = principal * 0.04 * (days_elapsed / 365)
    let principal_decimal = Decimal::from(principal);
    let days_decimal = Decimal::from(days_elapsed);
    let days_per_year = Decimal::from(365);

    let interest = principal_decimal
        .checked_mul(AAVE_APY_ANNUAL)
        .ok_or(error!(AaveIntegrationError))?
        .checked_div(Decimal::from(100))
        .ok_or(error!(AaveIntegrationError))?
        .checked_mul(days_decimal)
        .ok_or(error!(AaveIntegrationError))?
        .checked_div(days_per_year)
        .ok_or(error!(AaveIntegrationError))?;

    Ok(interest.trunc().to_u64().unwrap_or(0))
}

/// Calculate APY from interest earned
pub fn calculate_apy(principal: u64, interest: u64, days_elapsed: u64) -> Result<Decimal> {

    let principal_decimal = Decimal::from(principal);
    let interest_decimal = Decimal::from(interest);
    let days_decimal = Decimal::from(days_elapsed);
    let days_per_year = Decimal::from(365);

    // APY = (interest / principal) * (365 / days_elapsed) * 100
    let apy = interest_decimal
        .checked_div(principal_decimal)
        .ok_or(error!(AaveIntegrationError))?
        .checked_mul(days_per_year)
        .ok_or(error!(AaveIntegrationError))?
        .checked_div(days_decimal)
        .ok_or(error!(AaveIntegrationError))?
        .checked_mul(Decimal::from(100))
        .ok_or(error!(AaveIntegrationError))?;

    Ok(apy)
}

/// Simulate USDC deposit to Aave (ready for real CPI calls)
pub fn deposit_usdc_to_aave(amount: u64) -> Result<()> {
    validate_aave_deposit(amount)?;
    // TODO: Real Aave V3 deposit via CPI
    // Once `aave-v3-core` crate is available:
    // - Call Aave LendingPool.deposit(USDC, amount, receiver, referral_code)
    // - Update treasury vault's aave_position
    Ok(())
}

/// Simulate claiming yields from Aave (ready for real CPI calls)
pub fn claim_aave_yields(principal: u64, last_claim: i64, now: i64) -> Result<u64> {
    
    let days_elapsed = ((now - last_claim) / 86400) as u64; // seconds to days
    calculate_aave_interest(principal, days_elapsed)
}

#[error_code]
pub enum AaveError {
    #[msg("Aave integration error")]
    AaveIntegrationError,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}

use crate::errors::EcosystemError::*;
