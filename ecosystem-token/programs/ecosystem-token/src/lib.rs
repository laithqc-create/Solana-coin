use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

mod morpho;
mod rwa;
mod errors;
mod instructions;
mod investment_instructions;
mod state;
mod yield_strategy;

pub use anchor_lang::error::ErrorCode;
pub use errors::EcosystemError;

use instructions::*;
use investment_instructions::*;

#[program]
pub mod ecosystem_token {
    use super::*;

    // ── Initialization ─────────────────────────────────────────────────────
    pub fn initialize_launchpad(ctx: Context<InitializeLaunchpad>, bump: u8) -> Result<()> {
        instructions::initialize_launchpad(ctx, bump)
    }

    pub fn initialize_treasury(
        ctx: Context<InitializeTreasury>,
        marketing_address: Pubkey,
        asset_manager_address: Pubkey,
        protocol_address: Pubkey,
    ) -> Result<()> {
        instructions::initialize_treasury(
            ctx, marketing_address, asset_manager_address, protocol_address,
        )
    }

    // ── Token operations ───────────────────────────────────────────────────
    pub fn mint_tokens(ctx: Context<MintTokens>, usdc_amount: u64, is_campaign: bool) -> Result<()> {
        instructions::mint_tokens(ctx, usdc_amount, is_campaign)
    }

    pub fn burn_tokens(ctx: Context<BurnTokens>, token_amount: u64) -> Result<()> {
        instructions::burn_tokens(ctx, token_amount)
    }

    // ── Yield (auto — no staking required) ────────────────────────────────
    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        instructions::claim_yield(ctx)
    }

    // ── Unstaking ─────────────────────────────────────────────────────────
    pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
        instructions::request_unstake(ctx, amount)
    }

    pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
        instructions::complete_unstake(ctx)
    }

    pub fn emergency_redeem_defi(ctx: Context<EmergencyRedeemDefi>) -> Result<()> {
        instructions::emergency_redeem_defi(ctx)
    }

    // ── Strategy 1: USDC → sUSDS (Sky Protocol) ───────────────────────────
    pub fn initialize_sky_position(ctx: Context<InitializeSkyPosition>) -> Result<()> {
        investment_instructions::initialize_sky_position(ctx)
    }

    pub fn invest_usdc_to_susds(ctx: Context<InvestUsdcToSusds>) -> Result<()> {
        investment_instructions::invest_usdc_to_susds(ctx)
    }

    pub fn confirm_sky_investment(
        ctx: Context<KeeperReportSky>,
        susds_balance: u64,
        report_ts: i64,
    ) -> Result<()> {
        investment_instructions::confirm_sky_investment(ctx, susds_balance, report_ts)
    }

    pub fn report_sky_yield(
        ctx: Context<KeeperReportSky>,
        new_susds_balance: u64,
        new_apy_bps: u64,
        report_ts: i64,
    ) -> Result<()> {
        investment_instructions::report_sky_yield(ctx, new_susds_balance, new_apy_bps, report_ts)
    }

    pub fn withdraw_from_susds(ctx: Context<KeeperReportSky>) -> Result<()> {
        investment_instructions::withdraw_from_susds(ctx)
    }

    // ── Strategy 2: USDT → sUSDe (Ethena via Meteora) ─────────────────────
    pub fn initialize_ethena_position(ctx: Context<InitializeEthenaPosition>) -> Result<()> {
        investment_instructions::initialize_ethena_position(ctx)
    }

    pub fn invest_usdt_to_susde(ctx: Context<InvestUsdtToSusde>) -> Result<()> {
        investment_instructions::invest_usdt_to_susde(ctx)
    }

    pub fn confirm_ethena_investment(
        ctx: Context<KeeperReportEthena>,
        susde_balance: u64,
        report_ts: i64,
    ) -> Result<()> {
        investment_instructions::confirm_ethena_investment(ctx, susde_balance, report_ts)
    }

    pub fn report_ethena_yield(
        ctx: Context<KeeperReportEthena>,
        new_susde_balance: u64,
        new_apy_bps: u64,
        report_ts: i64,
    ) -> Result<()> {
        investment_instructions::report_ethena_yield(ctx, new_susde_balance, new_apy_bps, report_ts)
    }

    pub fn withdraw_from_susde(ctx: Context<KeeperReportEthena>) -> Result<()> {
        investment_instructions::withdraw_from_susde(ctx)
    }

    // ── Admin ──────────────────────────────────────────────────────────────
    pub fn update_revenue_split(
        ctx: Context<UpdateRevenueSplit>,
        holder_bps: u64,
        marketing_bps: u64,
        asset_manager_bps: u64,
        protocol_bps: u64,
        investment_ratio_bps: u64,
    ) -> Result<()> {
        instructions::update_revenue_split(
            ctx, holder_bps, marketing_bps,
            asset_manager_bps, protocol_bps, investment_ratio_bps,
        )
    }

    pub fn distribute_revenue(ctx: Context<DistributeRevenue>) -> Result<()> {
        instructions::distribute_revenue(ctx)
    }

    pub fn pause_launchpad(ctx: Context<PauseLaunchpad>) -> Result<()> {
        instructions::pause_launchpad(ctx)
    }

    pub fn resume_launchpad(ctx: Context<ResumeLaunchpad>) -> Result<()> {
        instructions::resume_launchpad(ctx)
    }
}
