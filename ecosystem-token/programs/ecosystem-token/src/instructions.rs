use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::errors::EcosystemError::*;
use crate::state::*;

// ============================================================================
// INITIALIZATION INSTRUCTIONS
// ============================================================================

pub fn initialize_launchpad(
    ctx: Context<InitializeLaunchpad>,
    token_bump: u8,
    vault_bump: u8,
) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad_state;
    
    launchpad.token_mint = ctx.accounts.token_mint.key();
    launchpad.usdc_mint = ctx.accounts.usdc_mint.key();
    launchpad.vault = ctx.accounts.vault.key();
    launchpad.tax_vault = ctx.accounts.tax_vault.key();
    launchpad.mint_authority = ctx.accounts.vault_pda.key();
    launchpad.authority = ctx.accounts.authority.key();
    launchpad.total_usdc_raised = 0;
    launchpad.total_tokens_minted = 0;
    launchpad.tier1_price = 1_000_000; // 1 USDC = 1e6 tokens
    launchpad.current_discount = 50;   // Start at 50% discount
    launchpad.discount_threshold_1 = 1_000_000_000_000; // 1M USDC
    launchpad.discount_threshold_2 = 2_000_000_000_000; // 2M USDC
    launchpad.paused = false;
    launchpad.bump = token_bump;

    let yield_config = &mut ctx.accounts.yield_config;
    let now = Clock::get()?.unix_timestamp;
    yield_config.snapshot_frequency = 604_800; // 7 days
    yield_config.last_snapshot = now;
    yield_config.next_snapshot = now + 604_800;
    yield_config.bump = ctx.bumps.yield_config;

    Ok(())
}

pub fn initialize_treasury(
    ctx: Context<InitializeTreasury>,
    marketing_address: Pubkey,
    asset_manager_address: Pubkey,
    owner_address: Pubkey,
) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury_vault;
    treasury.aave_position = 0;
    treasury.rwa_balance = 0;
    treasury.total_deposited = 0;
    treasury.total_usdc = 0;
    treasury.total_yields_earned = 0;
    treasury.total_yields_distributed = 0;
    treasury.last_aave_claim = Clock::get()?.unix_timestamp;
    treasury.last_rwa_sync = Clock::get()?.unix_timestamp;
    treasury.authority = ctx.accounts.authority.key();
    treasury.rwa_yield_earned = 0;
    treasury.bump = ctx.bumps.treasury_vault;

    let revenue_dist = &mut ctx.accounts.revenue_distribution;
    revenue_dist.user_percentage = 40;
    revenue_dist.marketing_percentage = 20;
    revenue_dist.asset_manager_percentage = 20;
    revenue_dist.owner_percentage = 20;
    revenue_dist.marketing_address = marketing_address;
    revenue_dist.marketing_pct = 20;
    revenue_dist.asset_manager_address = asset_manager_address;
    revenue_dist.manager_pct = 20;
    revenue_dist.owner_address = owner_address;
    revenue_dist.owner_pct = 20;
    revenue_dist.total_distributed = 0;
    revenue_dist.user_pct = 40;
    revenue_dist.authority = ctx.accounts.authority.key();
    revenue_dist.user_percentage = 40;
    revenue_dist.marketing_percentage = 20;
    revenue_dist.asset_manager_percentage = 20;
    revenue_dist.owner_percentage = 20;
    revenue_dist.bump = ctx.bumps.revenue_distribution;

    Ok(())
}

// ============================================================================
// MINTING & REDEMPTION
// ============================================================================

pub fn mint_tokens(ctx: Context<MintTokens>, usdc_amount: u64, is_tier2: bool) -> Result<()> {
    require!(!ctx.accounts.launchpad_state.paused, LaunchpadPaused);
    require!(usdc_amount > 0, InsufficientUsdc);

    let launchpad = &mut ctx.accounts.launchpad_state;

    // Calculate discount based on USDC raised
    let discount = if launchpad.total_usdc_raised < launchpad.discount_threshold_1 {
        50
    } else if launchpad.total_usdc_raised < launchpad.discount_threshold_2 {
        40
    } else {
        50
    };

    // Calculate tokens to mint
    let base_tokens = usdc_amount
        .checked_mul(launchpad.tier1_price)
        .ok_or(MathOverflow)?;
    let tokens_to_mint = if is_tier2 {
        base_tokens
            .checked_mul(100 + discount as u64)
            .ok_or(MathOverflow)?
            .checked_div(100)
            .ok_or(MathOverflow)?
    } else {
        base_tokens
    };

    // Check supply cap (100M tokens)
    let new_total = launchpad
        .total_tokens_minted
        .checked_add(tokens_to_mint)
        .ok_or(MathOverflow)?;
    // No supply cap - unlimited minting

    // Transfer USDC from user to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_usdc_ata.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, usdc_amount)?;

    // Mint tokens to user
    let seeds = &[b"vault".as_ref(), &[launchpad.bump]];
    let signer_seeds = &[&seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.user_token_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer_seeds,
    );
    mint_to(mint_ctx, tokens_to_mint)?;

    // Update user tier info
    let user_tier = &mut ctx.accounts.user_tier_info;
    user_tier.user = ctx.accounts.user.key();
    user_tier.is_tier2 = is_tier2;
    user_tier.tokens_purchased = tokens_to_mint;
    user_tier.total_tokens_minted = tokens_to_mint;

    if is_tier2 {
        let now = Clock::get()?.unix_timestamp;
        user_tier.vesting_start = now;
    }

    user_tier.bump = ctx.bumps.user_tier_info;

    // Update launchpad state
    launchpad.total_usdc_raised = launchpad
        .total_usdc_raised
        .checked_add(usdc_amount)
        .ok_or(MathOverflow)?;
    launchpad.total_tokens_minted = new_total;
    launchpad.current_discount = discount as u8;

    Ok(())
}

pub fn redeem_tokens(ctx: Context<RedeemTokens>, token_amount: u64) -> Result<()> {
    require!(!ctx.accounts.launchpad_state.paused, LaunchpadPaused);

    let user_tier = &ctx.accounts.user_tier_info;
    require!(!user_tier.is_tier2, CannotRedeemTier2);
    require!(token_amount > 0, InvalidRedeemAmount);

    // Transfer tokens to tax vault (burn simulation)
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_ata.to_account_info(),
            to: ctx.accounts.tax_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, token_amount)?;

    // Calculate USDC redemption (1 token = 1 USDC for Tier 1)
    let usdc_amount = token_amount.checked_div(1_000_000).ok_or(MathOverflow)?;

    // Transfer USDC from vault to user
    let seeds = &[b"vault".as_ref(), &[ctx.accounts.launchpad_state.bump]];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, usdc_amount)?;

    Ok(())
}

// ============================================================================
// TRANSFER WITH TAX
// ============================================================================

pub fn transfer_with_tax(ctx: Context<TransferWithTax>, amount: u64) -> Result<()> {
    require!(amount > 0, InsufficientTokens);

    let user_tier = &ctx.accounts.user_tier_info;

    // Check vesting lock for Tier 2
    if user_tier.is_tier2 {
        let now = Clock::get()?.unix_timestamp;
        let vesting_start = user_tier.vesting_start;
        let vesting_duration: i64 = 365 * 24 * 3600;
        let elapsed = (now - vesting_start).max(0);
        let vested = user_tier.tokens_purchased
            .checked_mul(elapsed as u64)
            .unwrap_or(0)
            .checked_div(vesting_duration as u64)
            .unwrap_or(0);
        require!(vested >= amount, CannotTransferLocked);
    }

    // Calculate 0.1% tax
    let tax = amount
        .checked_mul(1)
        .ok_or(MathOverflow)?
        .checked_div(1000)
        .ok_or(MathOverflow)?;
    let net_amount = amount.checked_sub(tax).ok_or(MathOverflow)?;

    // Split tax: 70% to treasury, 30% to yield vault
    let treasury_tax = tax
        .checked_mul(70)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;
    let yield_tax = tax.checked_sub(treasury_tax).ok_or(MathOverflow)?;

    // Transfer net amount to recipient
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, net_amount)?;

    // Transfer treasury tax
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.treasury_vault_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, treasury_tax)?;

    // Transfer yield tax
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.yield_vault_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, yield_tax)?;

    Ok(())
}

// ============================================================================
// STAKING & YIELD
// ============================================================================

pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
    require!(amount > 0, InsufficientTokens);

    let user_tier = &ctx.accounts.user_tier_info;

    // Ensure tokens are not vesting-locked
    if user_tier.is_tier2 {
        {
            let now = Clock::get()?.unix_timestamp;
            let vesting_start = user_tier.vesting_start;
            let vesting_duration: i64 = 365 * 24 * 3600;
            let elapsed = (now - vesting_start).max(0);
            let vested = user_tier.tokens_purchased
                .checked_mul(elapsed as u64)
                .unwrap_or(0)
                .checked_div(vesting_duration as u64)
                .unwrap_or(0);
            require!(vested >= amount, VestingLocked);
        }
    }

    // Transfer tokens to staking vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_ata.to_account_info(),
            to: ctx.accounts.staking_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update staking info
    let staking_info = &mut ctx.accounts.staking_info;
    staking_info.user = ctx.accounts.user.key();
    staking_info.staked_amount = staking_info
        .staked_amount
        .checked_add(amount)
        .ok_or(MathOverflow)?;
    staking_info.staked_at = Clock::get()?.unix_timestamp;
    staking_info.bump = ctx.bumps.staking_info;

    Ok(())
}

/// Step 1 of 2 for unstaking: starts the 48 business-hour countdown.
/// During this period, the off-chain keeper routes funds from RUSDY → USDC via Jupiter.
/// The user sees a countdown timer in the dashboard with weekend-pause logic.
pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
    require!(amount > 0, InsufficientTokens);

    let staking_info = &mut ctx.accounts.staking_info;
    require!(staking_info.staked_amount >= amount, NoStakedTokens);

    let now = Clock::get()?.unix_timestamp;

    // Deduct from staked balance immediately
    staking_info.staked_amount = staking_info
        .staked_amount
        .checked_sub(amount)
        .ok_or(MathOverflow)?;

    // Record the unstaking request with timestamp for countdown
    let request = &mut ctx.accounts.unstaking_request;
    request.user = ctx.accounts.user.key();
    request.amount = amount;
    request.usdc_to_return = amount; // 1:1 peg (USDC equivalent)
    request.requested_at = now;
    request.completed = false;
    request.emergency_redeemed = false;
    request.bump = ctx.bumps.unstaking_request;

    msg!(
        "Unstake requested: {} tokens. Your funds are being routed from RUSDY (real world assets) back to USDC. This may take up to 48 business hours due to regulatory requirements. Timer pauses on weekends.",
        amount
    );

    Ok(())
}

/// Step 2 of 2: Complete the unstake after 48 business hours have elapsed.
/// Weekends do NOT count toward the 48-hour requirement.
pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
    let request = &mut ctx.accounts.unstaking_request;
    require!(!request.completed, UnstakeAlreadyCompleted);
    require!(!request.emergency_redeemed, UnstakeAlreadyCompleted);

    let now = Clock::get()?.unix_timestamp;

    // Calculate business seconds elapsed (Mon–Fri only, weekends excluded)
    let elapsed = crate::rwa::business_seconds_elapsed(request.requested_at, now);

    require!(
        elapsed >= crate::rwa::UNSTAKE_BUSINESS_SECONDS,
        UnstakeCooldownNotMet
    );

    request.completed = true;

    // Transfer tokens back to user
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.user_token_ata.to_account_info(),
            authority: ctx.accounts.staking_vault.to_account_info(),
        },
    );
    transfer(transfer_ctx, request.amount)?;

    msg!(
        "Unstake complete: {} tokens returned to user after 48 business hours.",
        request.amount
    );

    Ok(())
}

/// Emergency DeFi redemption: bypasses the 48hr wait but exposes
/// the user to AMM slippage because RUSDY must be swapped immediately
/// via Jupiter at current market prices rather than via DCA routing.
pub fn emergency_redeem_defi(ctx: Context<EmergencyRedeemDefi>) -> Result<()> {
    let request = &mut ctx.accounts.unstaking_request;
    require!(!request.completed, UnstakeAlreadyCompleted);
    require!(!request.emergency_redeemed, UnstakeAlreadyCompleted);

    let amount = request.amount;
    request.emergency_redeemed = true;
    request.completed = true;

    // In production: trigger immediate Jupiter swap RUSDY → USDC
    // Slippage is NOT protected here — user accepts AMM market risk
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.user_token_ata.to_account_info(),
            authority: ctx.accounts.staking_vault.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    msg!(
        "EMERGENCY DeFi redemption executed: {} tokens. WARNING: Your transaction was subject to potential high slippage due to AMM market conditions. This is a result of bypassing the standard 48-hour DCA routing process.",
        amount
    );

    Ok(())
}

pub fn create_yield_snapshot(ctx: Context<CreateYieldSnapshot>) -> Result<()> {
    let yield_config = &mut ctx.accounts.yield_config;
    let now = Clock::get()?.unix_timestamp;

    require!(now >= yield_config.next_snapshot, SnapshotFrequencyNotMet);

    // Create new snapshot
    let snapshot = &mut ctx.accounts.yield_snapshot;
    snapshot.snapshot_ts = now;
    snapshot.snapshot_time = now;
    snapshot.total_staked = 0; // Would be summed from all StakingInfo in production
    snapshot.yield_amount = ctx.accounts.tax_vault.amount;
    snapshot.tax_collected = ctx.accounts.tax_vault.amount;
    snapshot.is_distributed = true;
    snapshot.bump = ctx.bumps.yield_snapshot;

    // Update yield config
    yield_config.last_snapshot = now;
    yield_config.next_snapshot = now + yield_config.snapshot_frequency;
    yield_config.snapshot_frequency = yield_config.snapshot_frequency;

    Ok(())
}

pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
    let staking_info = &ctx.accounts.staking_info;
    require!(staking_info.staked_amount > 0, NoStakedTokens);

    let snapshot = &ctx.accounts.yield_snapshot;
    require!(snapshot.is_distributed, YieldSnapshotNotReady);
    require!(snapshot.total_staked > 0, NoYieldToClaim);

    // Calculate pro-rata yield
    let user_share = (staking_info.staked_amount as u128)
        .checked_mul(snapshot.yield_amount as u128)
        .ok_or(MathOverflow)?
        .checked_div(snapshot.total_staked as u128)
        .ok_or(MathOverflow)? as u64;

    require!(user_share > 0, NoYieldToClaim);

    // Transfer yield to user
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.tax_vault.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, user_share)?;

    // Update staking info
    let mut staking_mut = ctx.accounts.staking_info.clone();
    staking_mut.last_claim = Clock::get()?.unix_timestamp;
    staking_mut.last_claim_ts = Clock::get()?.unix_timestamp;
    staking_mut.total_yield_claimed = staking_mut
        .total_yield_claimed
        .checked_add(user_share)
        .ok_or(MathOverflow)?;

    Ok(())
}

// ============================================================================
// ADMIN INSTRUCTIONS
// ============================================================================

pub fn set_tier2_whitelist(
    ctx: Context<SetTier2Whitelist>,
    is_whitelisted: bool,
) -> Result<()> {
    let whitelist = &mut ctx.accounts.tier2_whitelist;
    whitelist.user = ctx.accounts.target_user.key();
    whitelist.is_whitelisted = is_whitelisted;
    whitelist.whitelisted_at = if is_whitelisted {
        Clock::get()?.unix_timestamp
    } else {
        0
    };
    whitelist.authority = ctx.accounts.authority.key();
    whitelist.bump = ctx.bumps.tier2_whitelist;

    Ok(())
}

/// Invest treasury USDC into RUSDY via Jupiter DCA routing.
/// 70% of treasury tax goes here (replacing Aave).
/// Actual Jupiter swap is executed by off-chain keeper to prevent MEV.
pub fn invest_in_rwa(ctx: Context<InvestInRwa>, amount: u64) -> Result<()> {
    require!(amount > 0, InvalidRwaAmount);

    let treasury = &mut ctx.accounts.treasury_vault;
    let yield_config = &mut ctx.accounts.yield_config;
    let now = Clock::get()?.unix_timestamp;

    // Validate we have enough treasury balance
    require!(treasury.total_usdc >= amount, InsufficientTreasury);

    // Calculate USDC to route through Jupiter DCA → RUSDY
    let usdc_to_invest = crate::rwa::record_rwa_investment(amount, 70);

    // Update state: record pending RUSDY investment
    treasury.rwa_balance = treasury
        .rwa_balance
        .checked_add(usdc_to_invest)
        .ok_or(MathOverflow)?;

    treasury.total_usdc = treasury
        .total_usdc
        .checked_sub(usdc_to_invest)
        .ok_or(MathOverflow)?;

    yield_config.rwa_invested_usdc = yield_config
        .rwa_invested_usdc
        .checked_add(usdc_to_invest)
        .ok_or(MathOverflow)?;

    yield_config.last_investment_ts = now;

    msg!(
        "RWA investment queued: {} USDC routed to RUSDY via Jupiter DCA. Keeper will execute swap across multiple blocks to minimize slippage.",
        usdc_to_invest
    );

    Ok(())
}

/// Claim yield earned on RUSDY investment.
/// Yield accrues at ~5% APY on the invested USDC equivalent.
pub fn claim_rwa_yields(ctx: Context<ClaimRwaYields>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury_vault;
    let yield_config = &mut ctx.accounts.yield_config;
    let now = Clock::get()?.unix_timestamp;

    let seconds_elapsed = now
        .checked_sub(yield_config.last_investment_ts)
        .unwrap_or(0) as u64;

    let interest = crate::rwa::calculate_rwa_yield(
        yield_config.rwa_invested_usdc,
        seconds_elapsed,
    )?;

    require!(interest > 0, NoRwaYield);

    treasury.rwa_yield_earned = treasury
        .total_yields_earned
        .checked_add(interest)
        .ok_or(MathOverflow)?;

    treasury.total_yields_earned = treasury
        .total_yields_earned
        .checked_add(interest)
        .ok_or(MathOverflow)?;

    yield_config.last_investment_ts = now;

    msg!(
        "RUSDY yield claimed: {} USDC earned from RWA investment at ~5% APY.",
        interest
    );

    Ok(())
}

pub fn distribute_revenue(ctx: Context<DistributeRevenue>) -> Result<()> {
    let revenue_dist = &ctx.accounts.revenue_distribution;
    let treasury = &ctx.accounts.treasury_vault;

    require!(treasury.total_yields_earned > treasury.total_yields_distributed, InsufficientYield);

    let available = treasury
        .total_yields_earned
        .checked_sub(treasury.total_yields_distributed)
        .ok_or(MathOverflow)?;

    // Calculate distributions
    let user_share = available
        .checked_mul(revenue_dist.user_pct as u64)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;
    
    let marketing_share = available
        .checked_mul(revenue_dist.marketing_pct as u64)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;

    let manager_share = available
        .checked_mul(revenue_dist.manager_pct as u64)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;

    let owner_share = available
        .checked_mul(revenue_dist.owner_pct as u64)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;

    // In production: Transfer shares to respective addresses
    // For now: Just update treasury tracking
    let mut treasury_mut = ctx.accounts.treasury_vault.clone();
    treasury_mut.total_yields_distributed = treasury_mut
        .total_yields_distributed
        .checked_add(user_share + marketing_share + manager_share + owner_share)
        .ok_or(MathOverflow)?;

    Ok(())
}

pub fn update_allocation_percentages(
    ctx: Context<UpdateAllocationPercentages>,
    user_pct: u8,
    marketing_pct: u8,
    manager_pct: u8,
    owner_pct: u8,
) -> Result<()> {
    require!(
        (user_pct + marketing_pct + manager_pct + owner_pct) == 100,
        InvalidDistributionPercentages
    );

    let revenue_dist = &mut ctx.accounts.revenue_distribution;
    revenue_dist.user_pct = user_pct;
    revenue_dist.user_percentage = user_pct;
    revenue_dist.marketing_pct = marketing_pct;
    revenue_dist.marketing_percentage = marketing_pct;
    revenue_dist.manager_pct = manager_pct;
    revenue_dist.asset_manager_percentage = manager_pct;
    revenue_dist.owner_pct = owner_pct;
    revenue_dist.owner_percentage = owner_pct;

    Ok(())
}

pub fn pause_launchpad(ctx: Context<PauseLaunchpad>) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad_state;
    launchpad.paused = true;
    Ok(())
}

pub fn resume_launchpad(ctx: Context<ResumeLaunchpad>) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad_state;
    launchpad.paused = false;
    Ok(())
}

// ============================================================================
// CONTEXT DEFINITIONS
// ============================================================================

#[derive(Accounts)]
#[instruction(token_bump: u8, vault_bump: u8)]
pub struct InitializeLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 * 6 + 8 * 4 + 1 * 5 + 1,
        seeds = [b"launchpad"],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 * 2 + 8 + 1,
        seeds = [b"yield-config"],
        bump
    )]
    pub yield_config: Account<'info, YieldConfig>,

    pub token_mint: Account<'info, Mint>,
    pub usdc_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_pda
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = vault_pda
    )]
    pub tax_vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA
    #[account(seeds = [b"vault"], bump = vault_bump)]
    pub vault_pda: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 * 4 + 8 + 32 + 1,
        seeds = [b"treasury-vault"],
        bump
    )]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 32 * 3 + 1 * 4,
        seeds = [b"revenue-dist"],
        bump
    )]
    pub revenue_distribution: Account<'info, RevenueDistribution>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA
    pub vault_pda: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 1 + 50 + 8 + 1,
        seeds = [b"user-tier", user.key().as_ref()],
        bump
    )]
    pub user_tier_info: Account<'info, UserTierInfo>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RedeemTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub tax_vault: Account<'info, TokenAccount>,

    /// CHECK: Vault PDA
    pub vault_pda: UncheckedAccount<'info>,

    #[account(seeds = [b"user-tier", user.key().as_ref()], bump = user_tier_info.bump)]
    pub user_tier_info: Account<'info, UserTierInfo>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferWithTax<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_vault_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub yield_vault_ata: Account<'info, TokenAccount>,

    #[account(seeds = [b"user-tier", user.key().as_ref()], bump = user_tier_info.bump)]
    pub user_tier_info: Account<'info, UserTierInfo>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(seeds = [b"user-tier", user.key().as_ref()], bump = user_tier_info.bump)]
    pub user_tier_info: Account<'info, UserTierInfo>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 * 4 + 1,
        seeds = [b"staking", user.key().as_ref()],
        bump
    )]
    pub staking_info: Account<'info, StakingInfo>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump = staking_info.bump
    )]
    pub staking_info: Account<'info, StakingInfo>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 1,
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

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,

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

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,

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
pub struct CreateYieldSnapshot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    #[account(mut)]
    pub tax_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 * 2 + 8 + 1 + 1,
        seeds = [b"yield-snapshot".as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub yield_snapshot: Account<'info, YieldSnapshot>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub tax_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump = staking_info.bump
    )]
    pub staking_info: Account<'info, StakingInfo>,

    pub yield_snapshot: Account<'info, YieldSnapshot>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SetTier2Whitelist<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Target user to whitelist
    pub target_user: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 8 + 32 + 1,
        seeds = [b"whitelist", target_user.key().as_ref()],
        bump
    )]
    pub tier2_whitelist: Account<'info, Tier2Whitelist>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InvestInRwa<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRwaYields<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(mut, seeds = [b"yield-config"], bump = yield_config.bump)]
    pub yield_config: Account<'info, YieldConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeRevenue<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"treasury-vault"], bump = treasury_vault.bump)]
    pub treasury_vault: Account<'info, TreasuryVault>,

    #[account(seeds = [b"revenue-dist"], bump = revenue_distribution.bump)]
    pub revenue_distribution: Account<'info, RevenueDistribution>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAllocationPercentages<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"revenue-dist"], bump = revenue_distribution.bump)]
    pub revenue_distribution: Account<'info, RevenueDistribution>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PauseLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResumeLaunchpad<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad_state.bump)]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub system_program: Program<'info, System>,
}
