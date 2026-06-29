use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

mod morpho;
mod rwa;
mod errors;
mod instructions;
mod state;

pub use anchor_lang::error::ErrorCode;
pub use errors::EcosystemError;

use instructions::*;

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
        instructions::initialize_treasury(ctx, marketing_address, asset_manager_address, protocol_address)
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

    // ── Unstaking flow ─────────────────────────────────────────────────────
    pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
        instructions::request_unstake(ctx, amount)
    }

    pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
        instructions::complete_unstake(ctx)
    }

    pub fn emergency_redeem_defi(ctx: Context<EmergencyRedeemDefi>) -> Result<()> {
        instructions::emergency_redeem_defi(ctx)
    }

    // ── Morpho investment ──────────────────────────────────────────────────
    pub fn propose_morpho_pool(
        ctx: Context<ProposeMorphoPool>,
        pool_name: [u8; 64],
        pool_type: state::MorphoPoolType,
        apy_bps: u64,
    ) -> Result<()> {
        instructions::propose_morpho_pool(ctx, pool_name, pool_type, apy_bps)
    }

    pub fn approve_morpho_pool(ctx: Context<ApproveMorphoPool>) -> Result<()> {
        instructions::approve_morpho_pool(ctx)
    }

    pub fn invest_in_morpho(ctx: Context<InvestInMorpho>, amount: u64) -> Result<()> {
        instructions::invest_in_morpho(ctx, amount)
    }

    pub fn report_morpho_yield(ctx: Context<ReportMorphoYield>, yield_amount: u64) -> Result<()> {
        instructions::report_morpho_yield(ctx, yield_amount)
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
            ctx, holder_bps, marketing_bps, asset_manager_bps, protocol_bps, investment_ratio_bps,
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
