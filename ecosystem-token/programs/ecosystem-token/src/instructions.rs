use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Burn, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::errors::EcosystemError::*;
use crate::morpho::*;
use crate::state::*;

// ============================================================================
// INITIALIZATION
// ============================================================================

pub fn initialize_launchpad(
    ctx: Context<InitializeLaunchpad>,
    _bump: u8,
) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad_state;
    launchpad.authority = ctx.accounts.authority.key();
    launchpad.token_mint = ctx.accounts.token_mint.key();
    launchpad.usdc_mint = ctx.accounts.usdc_mint.key();
    launchpad.usdc_vault = ctx.accounts.usdc_vault.key();
    launchpad.vault = ctx.accounts.usdc_vault.key();
    launchpad.tax_vault = ctx.accounts.tax_vault.key();
    launchpad.mint_authority = ctx.accounts.vault_pda.key();
    launchpad.total_tokens_minted = 0;
    launchpad.total_usdc_raised = 0;
    launchpad.paused = false;
    launchpad.investment_pending_approval = true; // Blocked until admin approves a pool
    launchpad.bump = ctx.bumps.launchpad_state;
    Ok(())
}

pub fn initialize_treasury(
    ctx: Context<InitializeTreasury>,
    marketing_address: Pubkey,
    asset_manager_address: Pubkey,
    protocol_address: Pubkey,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    let treasury = &mut ctx.accounts.treasury_vault;
    treasury.authority = ctx.accounts.authority.key();
    treasury.total_usdc = 0;
    treasury.total_deposited = 0;
    treasury.total_yields_earned = 0;
    treasury.total_yields_distributed = 0;
    treasury.morpho_balance = 0;
    treasury.morpho_yield_earned = 0;
    treasury.liquid_usdc = 0;
    treasury.last_update = now;
    treasury.bump = ctx.bumps.treasury_vault;

    // Default revenue split: 25% / 25% / 25% / 25% (all equal, admin changes later)
    let dist = &mut ctx.accounts.revenue_distribution;
    dist.authority = ctx.accounts.authority.key();
    dist.holder_share_bps = 2_500;
    dist.marketing_share_bps = 2_500;
    dist.asset_manager_share_bps = 2_500;
    dist.protocol_share_bps = 2_500;
    dist.marketing_address = marketing_address;
    dist.asset_manager_address = asset_manager_address;
    dist.protocol_address = protocol_address;
    dist.investment_ratio_bps = INVESTMENT_RATIO_BPS; // 80%
    dist.bump = ctx.bumps.revenue_distribution;

    // Yield config — all holders auto-earn, no staking required
    let yc = &mut ctx.accounts.yield_config;
    yc.global_yield_index = YIELD_INDEX_PRECISION; // Start at 1.0
    yc.normal_yield_bps = NORMAL_YIELD_BPS;        // 45%
    yc.campaign_yield_bps = CAMPAIGN_YIELD_BPS;    // 70%
    yc.campaign_duration_seconds = CAMPAIGN_DURATION_SECONDS;
    yc.morpho_invested_usdc = 0;
    yc.morpho_yield_earned = 0;
    yc.last_index_update = now;
    yc.investment_ratio_bps = INVESTMENT_RATIO_BPS;
    yc.bump = ctx.bumps.yield_config;

    Ok(())
}

// ============================================================================
// MINT — Fixed 1:1 peg, fee from protocol treasury
// ============================================================================

/// Mint tokens at fixed 1 USDC = 1 token peg regardless of market price.
/// Fee tiers:
///   - Launchpad investor (whitelisted): 0.1% on ≤ original purchase, 0.5% on excess
///   - All others: 0.5%
/// ALL fees go 100% to protocol treasury.
/// Buying immediately enrolls user in yield — no staking required.
/// Campaign buyers (flagged via is_campaign) earn 70% APY for 6 months.
pub fn mint_tokens(
    ctx: Context<MintTokens>,
    usdc_amount: u64,
    is_campaign: bool,
) -> Result<()> {
    require!(!ctx.accounts.launchpad_state.paused, LaunchpadPaused);
    require!(usdc_amount > 0, InsufficientUsdc);

    let now = Clock::get()?.unix_timestamp;
    let launchpad = &mut ctx.accounts.launchpad_state;
    let yc = &mut ctx.accounts.yield_config;

    // ── Accrue global yield index before any balance change ──────────
    let elapsed = now.saturating_sub(yc.last_index_update).max(0);
    yc.global_yield_index = accrue_yield_index(
        yc.global_yield_index,
        yc.normal_yield_bps,
        elapsed,
    );
    yc.last_index_update = now;

    // ── Fixed 1:1 peg ────────────────────────────────────────────────
    // 1 USDC micro-unit = 1 token micro-unit (both 6 decimals)
    let tokens_to_mint = usdc_amount;

    // ── Fee calculation ───────────────────────────────────────────────
    let holder = &mut ctx.accounts.holder_info;
    let fee_result = calculate_fee(
        usdc_amount,
        holder.is_launchpad_investor,
        holder.launchpad_purchased_amount,
    );
    // Fee comes from the amount — user receives tokens_to_mint,
    // protocol treasury receives fee from its own reserves
    let protocol_fee = fee_result.total_fee;

    // ── Transfer USDC from user to vault ────────────────────────────
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_usdc_ata.to_account_info(),
            to: ctx.accounts.usdc_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, usdc_amount)?;

    // ── Mint tokens to user (full 1:1 amount) ───────────────────────
    let seeds = &[b"vault".as_ref(), &[launchpad.bump]];
    let signer = &[&seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.user_token_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer,
    );
    mint_to(mint_ctx, tokens_to_mint)?;

    // ── Mint protocol fee to treasury (100% of fee) ─────────────────
    if protocol_fee > 0 {
        let fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.protocol_treasury_ata.to_account_info(),
                authority: ctx.accounts.vault_pda.to_account_info(),
            },
            signer,
        );
        mint_to(fee_ctx, protocol_fee)?;
    }

    // ── Update holder info — snapshot yield index at purchase ────────
    // Accrue any pending yield before updating balance
    let pending_yield = calculate_holder_yield(
        yc.global_yield_index,
        holder.yield_index_snapshot,
        holder.token_balance,
    );
    holder.unclaimed_yield = holder.unclaimed_yield.saturating_add(pending_yield);
    holder.yield_index_snapshot = yc.global_yield_index;
    holder.token_balance = holder.token_balance.saturating_add(tokens_to_mint);

    // Campaign yield: 70% APY for 6 months from first purchase
    if is_campaign && holder.campaign_expires_at == 0 {
        holder.campaign_expires_at = now.saturating_add(yc.campaign_duration_seconds);
    }

    // Update launchpad totals
    launchpad.total_tokens_minted = launchpad.total_tokens_minted.saturating_add(tokens_to_mint);
    launchpad.total_usdc_raised = launchpad.total_usdc_raised.saturating_add(usdc_amount);

    msg!(
        "MINT: {} USDC → {} tokens at 1:1 peg. Fee: {} ({}). Campaign: {}",
        usdc_amount,
        tokens_to_mint,
        protocol_fee,
        if holder.is_launchpad_investor { "launchpad rate" } else { "standard rate" },
        is_campaign
    );

    Ok(())
}

// ============================================================================
// BURN — Fixed 1:1 peg, fee to protocol treasury
// ============================================================================

/// Burn tokens and receive USDC back at fixed 1:1 peg.
/// Price NEVER changes regardless of external market.
/// Fee tiers same as mint. All fees → 100% protocol treasury.
pub fn burn_tokens(ctx: Context<BurnTokens>, token_amount: u64) -> Result<()> {
    require!(!ctx.accounts.launchpad_state.paused, LaunchpadPaused);
    require!(token_amount > 0, InvalidRedeemAmount);

    let now = Clock::get()?.unix_timestamp;
    let launchpad = &ctx.accounts.launchpad_state;
    let yc = &mut ctx.accounts.yield_config;

    // ── Accrue global yield index before balance change ──────────────
    let elapsed = now.saturating_sub(yc.last_index_update).max(0);
    yc.global_yield_index = accrue_yield_index(
        yc.global_yield_index,
        yc.normal_yield_bps,
        elapsed,
    );
    yc.last_index_update = now;

    // ── Fee calculation ───────────────────────────────────────────────
    let holder = &mut ctx.accounts.holder_info;
    require!(holder.token_balance >= token_amount, InsufficientTokens);

    let fee_result = calculate_fee(
        token_amount,
        holder.is_launchpad_investor,
        holder.launchpad_purchased_amount,
    );
    let protocol_fee = fee_result.total_fee;

    // ── Fixed 1:1 USDC return ────────────────────────────────────────
    let usdc_to_return = token_amount; // 1:1 always

    // ── Burn ALL user's tokens ───────────────────────────────────────
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.user_token_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, token_amount)?;

    // ── Return full 1:1 USDC to user ────────────────────────────────
    let seeds = &[b"vault".as_ref(), &[launchpad.bump]];
    let signer = &[&seeds[..]];

    let usdc_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer,
    );
    transfer(usdc_ctx, usdc_to_return)?;

    // ── Protocol fee: transferred from vault reserves ────────────────
    if protocol_fee > 0 {
        let fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.usdc_vault.to_account_info(),
                to: ctx.accounts.protocol_treasury_ata.to_account_info(),
                authority: ctx.accounts.vault_pda.to_account_info(),
            },
            signer,
        );
        transfer(fee_ctx, protocol_fee)?;
    }

    // ── Update holder yield snapshot before balance change ───────────
    let pending_yield = calculate_holder_yield(
        yc.global_yield_index,
        holder.yield_index_snapshot,
        holder.token_balance,
    );
    holder.unclaimed_yield = holder.unclaimed_yield.saturating_add(pending_yield);
    holder.yield_index_snapshot = yc.global_yield_index;
    holder.token_balance = holder.token_balance.saturating_sub(token_amount);

    msg!(
        "BURN: {} tokens → {} USDC at 1:1 peg. Fee: {} to protocol treasury.",
        token_amount,
        usdc_to_return,
        protocol_fee
    );

    Ok(())
}

// ============================================================================
// YIELD CLAIM — Auto-yield, no staking needed
// ============================================================================

/// Claim accumulated yield. Any holder earns automatically just by holding.
/// Yield is calculated from the global index delta since last snapshot.
pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let yc = &mut ctx.accounts.yield_config;
    let holder = &mut ctx.accounts.holder_info;

    // Accrue index to now
    let elapsed = now.saturating_sub(yc.last_index_update).max(0);

    // Use campaign APY if eligible, else normal
    let apy = holder_apy_bps(holder.campaign_expires_at, now);
    yc.global_yield_index = accrue_yield_index(yc.global_yield_index, apy, elapsed);
    yc.last_index_update = now;

    // Calculate claimable yield
    let new_yield = calculate_holder_yield(
        yc.global_yield_index,
        holder.yield_index_snapshot,
        holder.token_balance,
    );
    let total_claimable = holder.unclaimed_yield.saturating_add(new_yield);
    require!(total_claimable > 0, NoYieldToClaim);

    // Reset holder snapshot
    holder.yield_index_snapshot = yc.global_yield_index;
    holder.unclaimed_yield = 0;

    // Transfer yield USDC from treasury to user
    let seeds = &[b"vault".as_ref(), &[ctx.accounts.launchpad_state.bump]];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.yield_usdc_vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer,
    );
    transfer(transfer_ctx, total_claimable)?;

    msg!(
        "YIELD CLAIM: {} USDC (APY: {}bps, campaign: {})",
        total_claimable,
        apy,
        holder.campaign_expires_at > now
    );

    Ok(())
}

// ============================================================================
// UNSTAKING — 48 business-hour countdown (weekend-paused)
// ============================================================================

/// Step 1: Request to exit. Starts 48 business-hour countdown.
/// During this period, Morpho position unwinds and USDC is routed back.
pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
    require!(amount > 0, InsufficientTokens);

    let holder = &mut ctx.accounts.holder_info;
    require!(holder.token_balance >= amount, InsufficientTokens);

    let now = Clock::get()?.unix_timestamp;

    // Accrue yield before reducing balance
    let yc = &ctx.accounts.yield_config;
    let pending = calculate_holder_yield(
        yc.global_yield_index,
        holder.yield_index_snapshot,
        holder.token_balance,
    );
    holder.unclaimed_yield = holder.unclaimed_yield.saturating_add(pending);
    holder.yield_index_snapshot = yc.global_yield_index;
    holder.token_balance = holder.token_balance.saturating_sub(amount);

    let request = &mut ctx.accounts.unstaking_request;
    request.user = ctx.accounts.user.key();
    request.amount = amount;
    request.usdc_to_return = amount; // 1:1 peg
    request.requested_at = now;
    request.completed = false;
    request.emergency_redeemed = false;
    request.bump = ctx.bumps.unstaking_request;

    msg!(
        "UNSTAKE REQUEST: {} tokens. Your funds are being routed from Morpho back to USDC. \
         This takes up to 48 business hours (Mon-Fri) due to regulatory compliance. \
         Timer pauses on weekends.",
        amount
    );

    Ok(())
}

/// Step 2: Complete after 48 business hours. Weekends excluded from timer.
pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
    let request = &mut ctx.accounts.unstaking_request;
    require!(!request.completed, UnstakeAlreadyCompleted);
    require!(!request.emergency_redeemed, UnstakeAlreadyCompleted);

    let now = Clock::get()?.unix_timestamp;
    let elapsed = crate::rwa::business_seconds_elapsed(request.requested_at, now);
    require!(elapsed >= crate::rwa::UNSTAKE_BUSINESS_SECONDS, UnstakeCooldownNotMet);

    request.completed = true;

    // Return tokens to user
    let seeds = &[b"vault".as_ref(), &[ctx.accounts.launchpad_state.bump]];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer,
    );
    transfer(transfer_ctx, request.usdc_to_return)?;

    msg!("UNSTAKE COMPLETE: {} USDC returned after 48 business hours.", request.usdc_to_return);
    Ok(())
}

/// Emergency exit: immediate but exposes user to AMM slippage.
pub fn emergency_redeem_defi(ctx: Context<EmergencyRedeemDefi>) -> Result<()> {
    let request = &mut ctx.accounts.unstaking_request;
    require!(!request.completed, UnstakeAlreadyCompleted);
    require!(!request.emergency_redeemed, UnstakeAlreadyCompleted);

    request.emergency_redeemed = true;
    request.completed = true;

    let seeds = &[b"vault".as_ref(), &[ctx.accounts.launchpad_state.bump]];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer,
    );
    transfer(transfer_ctx, request.amount)?;

    msg!(
        "EMERGENCY DEFI REDEMPTION: {} tokens redeemed immediately. \
         WARNING: Transaction subject to potential high slippage due to AMM \
         market conditions. This bypasses the standard 48-hour Morpho unwinding process.",
        request.amount
    );

    Ok(())
}

// ============================================================================
// MORPHO INVESTMENT — Admin approval required before investing
// ============================================================================

/// Admin proposes a Morpho pool. Must pass eligibility check.
/// Pool is NOT active until admin approves via approve_morpho_pool.
pub fn propose_morpho_pool(
    ctx: Context<ProposeMorphoPool>,
    pool_name: [u8; 64],
    pool_type: MorphoPoolType,
    apy_bps: u64,
) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );
    require!(is_eligible_pool_type(&pool_type), IneligiblePoolType);

    let pool = &mut ctx.accounts.morpho_pool;
    pool.pool_address = ctx.accounts.pool_address.key();
    pool.pool_name = pool_name;
    pool.pool_type = pool_type;
    pool.apy_bps = apy_bps;
    pool.approved = false; // Admin must approve separately
    pool.is_active = false;
    pool.invested_usdc = 0;
    pool.approved_at = 0;
    pool.bump = ctx.bumps.morpho_pool;

    msg!("Morpho pool proposed. Awaiting admin approval before investment can begin.");
    Ok(())
}

/// Admin approves a proposed Morpho pool — investment becomes unblocked.
pub fn approve_morpho_pool(ctx: Context<ApproveMorphoPool>) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );

    let pool = &mut ctx.accounts.morpho_pool;
    require!(is_eligible_pool_type(&pool.pool_type), IneligiblePoolType);

    pool.approved = true;
    pool.is_active = true;
    pool.approved_at = Clock::get()?.unix_timestamp;

    // Unblock investment on launchpad
    ctx.accounts.launchpad_state.investment_pending_approval = false;

    msg!("Morpho pool APPROVED. Investment is now enabled.");
    Ok(())
}

/// Invest 80% of liquid USDC into approved Morpho pool.
/// Off-chain keeper executes the actual cross-chain transfer.
pub fn invest_in_morpho(ctx: Context<InvestInMorpho>, amount: u64) -> Result<()> {
    require!(
        !ctx.accounts.launchpad_state.investment_pending_approval,
        InvestmentPendingApproval
    );
    require!(ctx.accounts.morpho_pool.approved, PoolNotApproved);
    require!(ctx.accounts.morpho_pool.is_active, PoolNotActive);
    require!(amount > 0, InvalidInvestmentAmount);

    let treasury = &mut ctx.accounts.treasury_vault;
    require!(treasury.liquid_usdc >= amount, InsufficientTreasury);

    let dist = &ctx.accounts.revenue_distribution;
    let max_investable = calculate_investment_amount(treasury.total_usdc, dist.investment_ratio_bps);
    require!(amount <= max_investable, InvalidInvestmentAmount);

    treasury.liquid_usdc = treasury.liquid_usdc.saturating_sub(amount);
    treasury.morpho_balance = treasury.morpho_balance.saturating_add(amount);

    let pool = &mut ctx.accounts.morpho_pool;
    pool.invested_usdc = pool.invested_usdc.saturating_add(amount);

    let yc = &mut ctx.accounts.yield_config;
    yc.morpho_invested_usdc = yc.morpho_invested_usdc.saturating_add(amount);

    msg!(
        "MORPHO INVESTMENT: {} USDC queued for investment. \
         Keeper will execute cross-chain transfer to Morpho pool.",
        amount
    );
    Ok(())
}

/// Keeper reports Morpho yield back on-chain.
pub fn report_morpho_yield(ctx: Context<ReportMorphoYield>, yield_amount: u64) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );
    require!(yield_amount > 0, NoMorphoYield);

    let treasury = &mut ctx.accounts.treasury_vault;
    treasury.morpho_yield_earned = treasury.morpho_yield_earned.saturating_add(yield_amount);
    treasury.total_yields_earned = treasury.total_yields_earned.saturating_add(yield_amount);
    treasury.liquid_usdc = treasury.liquid_usdc.saturating_add(yield_amount);

    let yc = &mut ctx.accounts.yield_config;
    yc.morpho_yield_earned = yc.morpho_yield_earned.saturating_add(yield_amount);

    msg!("MORPHO YIELD REPORTED: {} USDC earned from Morpho pool.", yield_amount);
    Ok(())
}

// ============================================================================
// ADMIN — Revenue split management
// ============================================================================

/// Update revenue split ratios. Must sum to 10000 bps (100%).
/// Admin can call this from the dashboard at any time.
pub fn update_revenue_split(
    ctx: Context<UpdateRevenueSplit>,
    holder_bps: u64,
    marketing_bps: u64,
    asset_manager_bps: u64,
    protocol_bps: u64,
    investment_ratio_bps: u64,
) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );
    require!(
        holder_bps + marketing_bps + asset_manager_bps + protocol_bps == 10_000,
        InvalidDistributionShares
    );
    require!(
        investment_ratio_bps <= 10_000,
        InvalidInvestmentRatio
    );

    let dist = &mut ctx.accounts.revenue_distribution;
    dist.holder_share_bps = holder_bps;
    dist.marketing_share_bps = marketing_bps;
    dist.asset_manager_share_bps = asset_manager_bps;
    dist.protocol_share_bps = protocol_bps;
    dist.investment_ratio_bps = investment_ratio_bps;

    msg!(
        "Revenue split updated: holders {}bps / marketing {}bps / manager {}bps / protocol {}bps. \
         Investment ratio: {}bps",
        holder_bps, marketing_bps, asset_manager_bps, protocol_bps, investment_ratio_bps
    );
    Ok(())
}

/// Distribute revenue to all parties based on current split ratios.
pub fn distribute_revenue(ctx: Context<DistributeRevenue>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury_vault;
    require!(
        treasury.total_yields_earned > treasury.total_yields_distributed,
        InsufficientYield
    );

    let distributable = treasury.total_yields_earned
        .saturating_sub(treasury.total_yields_distributed);

    let dist = &ctx.accounts.revenue_distribution;

    let holder_share = distributable.saturating_mul(dist.holder_share_bps).checked_div(10_000).unwrap_or(0);
    let marketing_share = distributable.saturating_mul(dist.marketing_share_bps).checked_div(10_000).unwrap_or(0);
    let manager_share = distributable.saturating_mul(dist.asset_manager_share_bps).checked_div(10_000).unwrap_or(0);
    let protocol_share = distributable.saturating_mul(dist.protocol_share_bps).checked_div(10_000).unwrap_or(0);

    treasury.total_yields_distributed = treasury.total_yields_distributed
        .saturating_add(holder_share + marketing_share + manager_share + protocol_share);

    msg!(
        "Revenue distributed: {} holders / {} marketing / {} manager / {} protocol",
        holder_share, marketing_share, manager_share, protocol_share
    );
    Ok(())
}

/// Pause launchpad (emergency use only)
pub fn pause_launchpad(ctx: Context<PauseLaunchpad>) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );
    ctx.accounts.launchpad_state.paused = true;
    msg!("Launchpad PAUSED by authority.");
    Ok(())
}

/// Resume launchpad
pub fn resume_launchpad(ctx: Context<ResumeLaunchpad>) -> Result<()> {
    require!(
        ctx.accounts.launchpad_state.authority == ctx.accounts.authority.key(),
        Unauthorized
    );
    ctx.accounts.launchpad_state.paused = false;
    msg!("Launchpad RESUMED by authority.");
    Ok(())
}

// ============================================================================
// ACCOUNT STRUCTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 * 7 + 8 * 2 + 1 * 3,
        seeds = [b"launchpad"],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub token_mint: Account<'info, Mint>,

    /// CHECK: USDC mint address
    pub usdc_mint: UncheckedAccount<'info>,

    /// CHECK: USDC vault
    pub usdc_vault: UncheckedAccount<'info>,

    /// CHECK: Tax vault
    pub tax_vault: UncheckedAccount<'info>,

    /// CHECK: Vault PDA
    pub vault_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 * 8 + 8,
        seeds = [b"treasury-vault"],
        bump
    )]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 * 4 + 8 * 5,
        seeds = [b"revenue-dist"],
        bump
    )]
    pub revenue_distribution: Account<'info, RevenueDistribution>,

    #[account(
        init,
        payer = authority,
        space = 8 + 16 + 8 * 7 + 8,
        seeds = [b"yield-config"],
        bump
    )]
    pub yield_config: Account<'info, YieldConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    /// Protocol treasury receives 100% of fees
    #[account(mut)]
    pub protocol_treasury_ata: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA authority
    pub vault_pda: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 16 + 8 * 4 + 1 + 1,
        seeds = [b"holder", user.key().as_ref()],
        bump
    )]
    pub holder_info: Account<'info, HolderInfo>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    /// Protocol treasury receives 100% of fees
    #[account(mut)]
    pub protocol_treasury_ata: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA authority
    pub vault_pda: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"holder", user.key().as_ref()], bump = holder_info.bump)]
    pub holder_info: Account<'info, HolderInfo>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub yield_usdc_vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA authority
    pub vault_pda: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"holder", user.key().as_ref()], bump = holder_info.bump)]
    pub holder_info: Account<'info, HolderInfo>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut, seeds = [b"holder", user.key().as_ref()], bump = holder_info.bump)]
    pub holder_info: Account<'info, HolderInfo>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 * 3 + 1 * 3,
        seeds = [b"unstaking", user.key().as_ref()],
        bump
    )]
    pub unstaking_request: Account<'info, UnstakingRequest>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompleteUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA authority
    pub vault_pda: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"unstaking", user.key().as_ref()],
        bump = unstaking_request.bump,
        has_one = user
    )]
    pub unstaking_request: Account<'info, UnstakingRequest>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyRedeemDefi<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA authority
    pub vault_pda: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"unstaking", user.key().as_ref()],
        bump = unstaking_request.bump,
        has_one = user
    )]
    pub unstaking_request: Account<'info, UnstakingRequest>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProposeMorphoPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 64 + 1 + 8 * 3 + 1 * 2 + 8,
        seeds = [b"morpho-pool", pool_address.key().as_ref()],
        bump
    )]
    pub morpho_pool: Account<'info, MorphoPool>,

    /// CHECK: The Morpho pool contract address
    pub pool_address: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveMorphoPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut)]
    pub morpho_pool: Account<'info, MorphoPool>,
}

#[derive(Accounts)]
pub struct InvestInMorpho<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(seeds = [b"revenue-dist"], bump = revenue_distribution.bump)]
    pub revenue_distribution: Account<'info, RevenueDistribution>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut)]
    pub morpho_pool: Account<'info, MorphoPool>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReportMorphoYield<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,
}

#[derive(Accounts)]
pub struct UpdateRevenueSplit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut, seeds = [b"revenue-dist"], bump = revenue_distribution.bump)]
    pub revenue_distribution: Account<'info, RevenueDistribution>,
}

#[derive(Accounts)]
pub struct DistributeRevenue<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(seeds = [b"revenue-dist"], bump = revenue_distribution.bump)]
    pub revenue_distribution: Account<'info, RevenueDistribution>,
}

#[derive(Accounts)]
pub struct PauseLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,
}

#[derive(Accounts)]
pub struct ResumeLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,
}
