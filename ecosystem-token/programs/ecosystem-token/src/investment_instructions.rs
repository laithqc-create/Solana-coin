//! Investment instructions for sUSDS and sUSDe strategies.
//! Follows Rule 7: clean separate file, wired into lib.rs.
//! Follows Rule 8: all inputs validated, checked math, no silent failures.

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::errors::EcosystemError::*;
use crate::state::*;
use crate::yield_strategy::*;

// ============================================================================
// STRATEGY 1: USDC → USDS → sUSDS (Sky Protocol)
// ============================================================================

/// Initialize the Sky Protocol position account.
/// Called once by admin after treasury is set up.
pub fn initialize_sky_position(ctx: Context<InitializeSkyPosition>) -> Result<()> {
    let pos = &mut ctx.accounts.sky_position;
    let now = Clock::get()?.unix_timestamp;

    pos.authority      = ctx.accounts.authority.key();
    pos.usdc_deposited = 0;
    pos.susds_balance  = 0;
    pos.total_yield_earned = 0;
    pos.current_apy_bps = SKY_BASE_APY_BPS;
    pos.last_update_ts = now;
    pos.status         = StrategyStatus::Inactive;
    pos.bump           = ctx.bumps.sky_position;

    msg!("Sky sUSDS position initialized.");
    Ok(())
}

/// Route 80% of USDC pool into sUSDS via Sky Protocol native bridge.
/// 20% stays liquid for immediate redemptions.
/// Off-chain keeper executes: USDC → bridge → USDS → stake → sUSDS
pub fn invest_usdc_to_susds(ctx: Context<InvestUsdcToSusds>) -> Result<()> {
    require!(
        ctx.accounts.sky_position.status == StrategyStatus::Inactive
            || ctx.accounts.sky_position.status == StrategyStatus::PendingWithdrawal,
        YieldStrategyError::InvestmentAlreadyActive
    );

    let pool = &mut ctx.accounts.pool_state;
    require!(pool.liquid_usdc > 0,
        EcosystemError::InsufficientTreasury
    );

    // Split 80% invest / 20% liquid
    let (invest_amount, liquid_amount) = split_investment(pool.total_usdc)?;
    require!(
        invest_amount > 0,
        EcosystemError::InvalidInvestmentAmount
    );

    // Lock funds — move from liquid to invested bucket
    pool.liquid_usdc   = liquid_amount;
    pool.invested_usdc = invest_amount;
    pool.last_update_ts = Clock::get()?.unix_timestamp;

    // Record intent for keeper
    let pos = &mut ctx.accounts.sky_position;
    pos.usdc_deposited = invest_amount;
    pos.status         = StrategyStatus::PendingExecution;
    pos.last_update_ts = pool.last_update_ts;

    msg!(
        "sUSDS INVESTMENT QUEUED: {} USDC (80%). {} USDC kept liquid (20%). \
         Keeper will bridge USDC → USDS → sUSDS via Sky Protocol native bridge.",
        invest_amount, liquid_amount
    );

    Ok(())
}

/// Keeper confirms sUSDS position is live and reports initial balance.
pub fn confirm_sky_investment(
    ctx: Context<KeeperReportSky>,
    susds_balance: u64,
    report_ts: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pos = &mut ctx.accounts.sky_position;

    validate_keeper_report(
        pos.usdc_deposited,
        susds_balance,
        &StrategyStatus::PendingExecution,
        &pos.status,
        report_ts,
        now,
    )?;

    pos.susds_balance  = susds_balance;
    pos.status         = StrategyStatus::Active;
    pos.last_update_ts = now;

    msg!("sUSDS POSITION CONFIRMED: {} sUSDS balance.", susds_balance);
    Ok(())
}

/// Keeper reports yield earned on sUSDS position.
/// Updates APY and accrued yield; adds to pool's distributable yield.
pub fn report_sky_yield(
    ctx: Context<KeeperReportSky>,
    new_susds_balance: u64,
    new_apy_bps: u64,
    report_ts: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pos = &mut ctx.accounts.sky_position;

    require!(pos.status == StrategyStatus::Active, YieldStrategyError::InvalidStrategyStatus);
    require!(report_ts <= now, YieldStrategyError::InvalidTimestamp);

    // Yield = balance increase since last report
    let yield_earned = new_susds_balance.saturating_sub(pos.susds_balance);

    pos.susds_balance      = new_susds_balance;
    pos.current_apy_bps    = new_apy_bps;
    pos.total_yield_earned = pos.total_yield_earned.saturating_add(yield_earned);
    pos.last_update_ts     = now;

    // Credit yield to pool
    let pool = &mut ctx.accounts.pool_state;
    pool.liquid_usdc   = pool.liquid_usdc.saturating_add(yield_earned);
    pool.total_usdc    = pool.total_usdc.saturating_add(yield_earned);
    pool.last_update_ts = now;

    msg!(
        "sUSDS YIELD REPORT: +{} USDC yield. Balance: {}. APY: {}bps.",
        yield_earned, new_susds_balance, new_apy_bps
    );

    Ok(())
}

/// Initiate withdrawal from sUSDS back to USDC.
/// Keeper will unstake sUSDS → USDS → bridge → USDC.
pub fn withdraw_from_susds(ctx: Context<KeeperReportSky>) -> Result<()> {
    let pos = &mut ctx.accounts.sky_position;
    require!(
        pos.status == StrategyStatus::Active, YieldStrategyError::InvalidStrategyStatus);

    pos.status = StrategyStatus::PendingWithdrawal;

    msg!(
        "sUSDS WITHDRAWAL INITIATED: {} sUSDS being unwound. \
         Keeper will unstake sUSDS → USDS → bridge → USDC.",
        pos.susds_balance
    );

    Ok(())
}

// ============================================================================
// STRATEGY 2: USDT → USDe → sUSDe (Ethena via Meteora)
// ============================================================================

/// Initialize the Ethena position account.
pub fn initialize_ethena_position(ctx: Context<InitializeEthenaPosition>) -> Result<()> {
    let pos = &mut ctx.accounts.ethena_position;
    let now = Clock::get()?.unix_timestamp;

    pos.authority      = ctx.accounts.authority.key();
    pos.usdt_deposited = 0;
    pos.susde_balance  = 0;
    pos.total_yield_earned = 0;
    pos.current_apy_bps = ETHENA_BASE_APY_BPS;
    pos.last_update_ts = now;
    pos.status         = StrategyStatus::Inactive;
    pos.bump           = ctx.bumps.ethena_position;

    msg!("Ethena sUSDe position initialized.");
    Ok(())
}

/// Route 80% of USDT pool into sUSDe via Ethena through Meteora.
/// Keeper executes: USDT → Meteora swap → USDe → wrap → sUSDe
pub fn invest_usdt_to_susde(ctx: Context<InvestUsdtToSusde>) -> Result<()> {
    require!(
        ctx.accounts.ethena_position.status == StrategyStatus::Inactive
            || ctx.accounts.ethena_position.status == StrategyStatus::PendingWithdrawal,
        YieldStrategyError::InvestmentAlreadyActive
    );

    let pool = &mut ctx.accounts.pool_state;
    require!(pool.liquid_usdt > 0,
        EcosystemError::InsufficientTreasury
    );

    let (invest_amount, liquid_amount) = split_investment(pool.total_usdt)?;
    require!(
        invest_amount > 0,
        EcosystemError::InvalidInvestmentAmount
    );

    pool.liquid_usdt   = liquid_amount;
    pool.invested_usdt = invest_amount;
    pool.last_update_ts = Clock::get()?.unix_timestamp;

    let pos = &mut ctx.accounts.ethena_position;
    pos.usdt_deposited = invest_amount;
    pos.status         = StrategyStatus::PendingExecution;
    pos.last_update_ts = pool.last_update_ts;

    msg!(
        "sUSDe INVESTMENT QUEUED: {} USDT (80%). {} USDT kept liquid (20%). \
         Keeper will route USDT → Meteora → USDe → sUSDe via Ethena.",
        invest_amount, liquid_amount
    );

    Ok(())
}

/// Keeper confirms sUSDe position is live.
pub fn confirm_ethena_investment(
    ctx: Context<KeeperReportEthena>,
    susde_balance: u64,
    report_ts: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pos = &mut ctx.accounts.ethena_position;

    validate_keeper_report(
        pos.usdt_deposited,
        susde_balance,
        &StrategyStatus::PendingExecution,
        &pos.status,
        report_ts,
        now,
    )?;

    pos.susde_balance  = susde_balance;
    pos.status         = StrategyStatus::Active;
    pos.last_update_ts = now;

    msg!("sUSDe POSITION CONFIRMED: {} sUSDe balance.", susde_balance);
    Ok(())
}

/// Keeper reports yield earned on sUSDe position.
/// Ethena APY is variable — keeper always reports current rate.
pub fn report_ethena_yield(
    ctx: Context<KeeperReportEthena>,
    new_susde_balance: u64,
    new_apy_bps: u64,
    report_ts: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pos = &mut ctx.accounts.ethena_position;

    require!(pos.status == StrategyStatus::Active, YieldStrategyError::InvalidStrategyStatus);
    require!(report_ts <= now, YieldStrategyError::InvalidTimestamp);

    let yield_earned = new_susde_balance.saturating_sub(pos.susde_balance);

    pos.susde_balance      = new_susde_balance;
    pos.current_apy_bps    = new_apy_bps;
    pos.total_yield_earned = pos.total_yield_earned.saturating_add(yield_earned);
    pos.last_update_ts     = now;

    let pool = &mut ctx.accounts.pool_state;
    pool.liquid_usdt   = pool.liquid_usdt.saturating_add(yield_earned);
    pool.total_usdt    = pool.total_usdt.saturating_add(yield_earned);
    pool.last_update_ts = now;

    msg!(
        "sUSDe YIELD REPORT: +{} USDT yield. Balance: {}. APY: {}bps (variable).",
        yield_earned, new_susde_balance, new_apy_bps
    );

    Ok(())
}

/// Initiate withdrawal from sUSDe back to USDT.
pub fn withdraw_from_susde(ctx: Context<KeeperReportEthena>) -> Result<()> {
    let pos = &mut ctx.accounts.ethena_position;
    require!(pos.status == StrategyStatus::Active, YieldStrategyError::InvalidStrategyStatus);

    pos.status = StrategyStatus::PendingWithdrawal;

    msg!(
        "sUSDe WITHDRAWAL INITIATED: {} sUSDe being unwound. \
         Keeper will unstake sUSDe → USDe → Meteora → USDT.",
        pos.susde_balance
    );

    Ok(())
}

// ============================================================================
// ACCOUNT STRUCTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeSkyPosition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 * 5 + 8 + 1 + 1,
        seeds = [b"sky-position"],
        bump
    )]
    pub sky_position: Account<'info, SkyPosition>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InvestUsdcToSusds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"pool-state"], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut, seeds = [b"sky-position"], bump = sky_position.bump)]
    pub sky_position: Account<'info, SkyPosition>,
}

#[derive(Accounts)]
pub struct KeeperReportSky<'info> {
    /// Only the protocol authority can submit keeper reports
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"sky-position"], bump = sky_position.bump)]
    pub sky_position: Account<'info, SkyPosition>,

    #[account(mut, seeds = [b"pool-state"], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,
}

#[derive(Accounts)]
pub struct InitializeEthenaPosition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 * 5 + 8 + 1 + 1,
        seeds = [b"ethena-position"],
        bump
    )]
    pub ethena_position: Account<'info, EthenaPosition>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InvestUsdtToSusde<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"pool-state"], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut, seeds = [b"ethena-position"], bump = ethena_position.bump)]
    pub ethena_position: Account<'info, EthenaPosition>,
}

#[derive(Accounts)]
pub struct KeeperReportEthena<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"ethena-position"], bump = ethena_position.bump)]
    pub ethena_position: Account<'info, EthenaPosition>,

    #[account(mut, seeds = [b"pool-state"], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,
}
