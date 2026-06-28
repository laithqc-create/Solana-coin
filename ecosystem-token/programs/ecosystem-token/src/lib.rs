use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

mod rwa;
mod errors;
mod instructions;
mod state;

// Re-export ErrorCode at crate root so Anchor's require! macro can find it
pub use anchor_lang::error::ErrorCode;

use instructions::*;

#[program]
pub mod ecosystem_token {
    use super::*;

    // ============================================================================
    // INITIALIZATION
    // ============================================================================

    pub fn initialize_launchpad(
        ctx: Context<InitializeLaunchpad>,
        token_bump: u8,
        vault_bump: u8,
    ) -> Result<()> {
        instructions::initialize_launchpad(ctx, token_bump, vault_bump)
    }

    pub fn initialize_treasury(
        ctx: Context<InitializeTreasury>,
        marketing_address: Pubkey,
        asset_manager_address: Pubkey,
        owner_address: Pubkey,
    ) -> Result<()> {
        instructions::initialize_treasury(ctx, marketing_address, asset_manager_address, owner_address)
    }

    // ============================================================================
    // MINTING & REDEMPTION
    // ============================================================================

    pub fn mint_tokens(ctx: Context<MintTokens>, usdc_amount: u64, is_tier2: bool) -> Result<()> {
        instructions::mint_tokens(ctx, usdc_amount, is_tier2)
    }

    pub fn burn_tokens(ctx: Context<BurnTokens>, token_amount: u64) -> Result<()> {
        instructions::burn_tokens(ctx, token_amount)
    }

    // ============================================================================
    // TRANSFER WITH TAX
    // ============================================================================



    // ============================================================================
    // STAKING & YIELD
    // ============================================================================

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        instructions::stake_tokens(ctx, amount)
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
        instructions::request_unstake(ctx, amount)
    }

    pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
        instructions::complete_unstake(ctx)
    }

    pub fn emergency_redeem_defi(ctx: Context<EmergencyRedeemDefi>) -> Result<()> {
        instructions::emergency_redeem_defi(ctx)
    }

    pub fn create_yield_snapshot(ctx: Context<CreateYieldSnapshot>) -> Result<()> {
        instructions::create_yield_snapshot(ctx)
    }

    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        instructions::claim_yield(ctx)
    }

    // ============================================================================
    // ADMIN INSTRUCTIONS
    // ============================================================================

    pub fn set_tier2_whitelist(
        ctx: Context<SetTier2Whitelist>,
        is_whitelisted: bool,
    ) -> Result<()> {
        instructions::set_tier2_whitelist(ctx, is_whitelisted)
    }

    pub fn invest_in_rwa(ctx: Context<InvestInRwa>, amount: u64) -> Result<()> {
        instructions::invest_in_rwa(ctx, amount)
    }

    pub fn claim_rwa_yields(ctx: Context<ClaimRwaYields>) -> Result<()> {
        instructions::claim_rwa_yields(ctx)
    }

    pub fn distribute_revenue(ctx: Context<DistributeRevenue>) -> Result<()> {
        instructions::distribute_revenue(ctx)
    }

    pub fn update_allocation_percentages(
        ctx: Context<UpdateAllocationPercentages>,
        user_pct: u8,
        marketing_pct: u8,
        manager_pct: u8,
        owner_pct: u8,
    ) -> Result<()> {
        instructions::update_allocation_percentages(
            ctx,
            user_pct,
            marketing_pct,
            manager_pct,
            owner_pct,
        )
    }

    pub fn pause_launchpad(ctx: Context<PauseLaunchpad>) -> Result<()> {
        instructions::pause_launchpad(ctx)
    }

    pub fn resume_launchpad(ctx: Context<ResumeLaunchpad>) -> Result<()> {
        instructions::resume_launchpad(ctx)
    }
}
