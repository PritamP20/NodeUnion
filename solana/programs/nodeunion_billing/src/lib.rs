use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("zEmSJV8TWmSwHX2f6RdyFvZgvCwQaJ9ZrLfdQtidexo");

#[program]
pub mod nodeunion_billing {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        treasury: Pubkey,
        rate_per_unit: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.token_mint = ctx.accounts.token_mint.key();
        config.treasury = treasury;
        config.rate_per_unit = rate_per_unit;
        config.bump = ctx.bumps.config;
        Ok(())
    }

    // Register a network on-chain
    pub fn register_network(
        ctx: Context<RegisterNetwork>,
        network_id: String,
        name: String,
    ) -> Result<()> {
        require!(!network_id.is_empty(), BillingError::InvalidNetworkId);
        require!(network_id.len() <= NetworkRegistry::MAX_NETWORK_ID_LEN, BillingError::InvalidNetworkId);
        require!(!name.is_empty(), BillingError::InvalidName);

        let registry = &mut ctx.accounts.registry;
        registry.network_id = network_id.clone();
        registry.name = name;
        registry.authority = ctx.accounts.authority.key();
        registry.created_at = Clock::get()?.unix_timestamp;
        registry.bump = ctx.bumps.registry;

        emit!(NetworkRegisteredEvent {
            network_id,
        });

        Ok(())
    }

    // Register a provider (node) in a network
    pub fn register_provider(
        ctx: Context<RegisterProvider>,
        network_id: String,
        provider_id: String,
        provider_wallet: Pubkey,
    ) -> Result<()> {
        require!(!provider_id.is_empty(), BillingError::InvalidProviderId);
        require!(provider_id.len() <= ProviderRegistry::MAX_PROVIDER_ID_LEN, BillingError::InvalidProviderId);

        let provider = &mut ctx.accounts.provider;
        provider.provider_id = provider_id;
        provider.network_id = network_id;
        provider.provider_wallet = provider_wallet;
        provider.authority = ctx.accounts.authority.key();
        provider.created_at = Clock::get()?.unix_timestamp;
        provider.bump = ctx.bumps.provider;

        emit!(ProviderRegisteredEvent {
            provider_id: provider.provider_id.clone(),
            network_id: provider.network_id.clone(),
            wallet: provider_wallet,
        });

        Ok(())
    }

    pub fn open_escrow(
        ctx: Context<OpenEscrow>,
        job_id: String,
        network_id: String,
        provider_id: String,
        max_units: u64,
        deposit_amount: u64,
    ) -> Result<()> {
        require!(!job_id.is_empty(), BillingError::InvalidJobId);
        require!(job_id.len() <= JobEscrow::MAX_JOB_ID_LEN, BillingError::InvalidJobId);
        require!(network_id.len() <= JobEscrow::MAX_NETWORK_ID_LEN, BillingError::InvalidNetworkId);
        require!(deposit_amount > 0, BillingError::InvalidAmount);

        // Validate network exists (via registry account)
        require!(ctx.accounts.network_registry.network_id == network_id, BillingError::InvalidNetwork);

        // Validate provider is in this network
        require!(ctx.accounts.provider_registry.network_id == network_id, BillingError::ProviderNotInNetwork);
        require!(ctx.accounts.provider_registry.provider_id == provider_id, BillingError::InvalidProviderId);

        let escrow = &mut ctx.accounts.escrow;
        escrow.job_id = job_id;
        escrow.network_id = network_id;
        escrow.provider_id = provider_id;
        escrow.user = ctx.accounts.user.key();
        escrow.provider_wallet = ctx.accounts.provider_registry.provider_wallet;
        escrow.config = ctx.accounts.config.key();
        escrow.max_units = max_units;
        escrow.used_units = 0;
        escrow.deposit_amount = deposit_amount;
        escrow.spent_amount = 0;
        escrow.status = EscrowStatus::Open;
        escrow.created_at = Clock::get()?.unix_timestamp;
        escrow.bump = ctx.bumps.escrow;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, deposit_amount)?;

        emit!(EscrowOpenedEvent {
            job_id: escrow.job_id.clone(),
            network_id: escrow.network_id.clone(),
            user: escrow.user,
            provider_wallet: escrow.provider_wallet,
            deposit_amount,
            max_units,
        });

        Ok(())
    }

    pub fn record_usage(ctx: Context<RecordUsage>, units: u64) -> Result<()> {
        let config = &ctx.accounts.config;

        require!(ctx.accounts.escrow.status == EscrowStatus::Open, BillingError::EscrowClosed);
        require!(units > 0, BillingError::InvalidUnits);

        let escrow_bump = ctx.accounts.escrow.bump;
        let escrow_job_id = ctx.accounts.escrow.job_id.clone();

        let new_used = ctx.accounts
            .escrow
            .used_units
            .checked_add(units)
            .ok_or(BillingError::MathOverflow)?;

        require!(new_used <= ctx.accounts.escrow.max_units, BillingError::UsageExceedsLimit);

        let amount = units
            .checked_mul(config.rate_per_unit)
            .ok_or(BillingError::MathOverflow)?;

        let new_spent = ctx.accounts
            .escrow
            .spent_amount
            .checked_add(amount)
            .ok_or(BillingError::MathOverflow)?;

        require!(new_spent <= ctx.accounts.escrow.deposit_amount, BillingError::InsufficientEscrow);

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            escrow_job_id.as_bytes(),
            &[escrow_bump],
        ]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.provider_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, amount)?;

        let escrow = &mut ctx.accounts.escrow;
        escrow.used_units = new_used;
        escrow.spent_amount = new_spent;

        emit!(UsageRecordedEvent {
            job_id: escrow.job_id.clone(),
            network_id: escrow.network_id.clone(),
            units,
            amount,
            total_used_units: escrow.used_units,
            total_spent_amount: escrow.spent_amount,
        });

        Ok(())
    }

    pub fn close_escrow(ctx: Context<CloseEscrow>) -> Result<()> {
        require!(ctx.accounts.escrow.status == EscrowStatus::Open, BillingError::EscrowClosed);

        let escrow_bump = ctx.accounts.escrow.bump;
        let escrow_job_id = ctx.accounts.escrow.job_id.clone();
        let deposit_amount = ctx.accounts.escrow.deposit_amount;
        let spent_amount = ctx.accounts.escrow.spent_amount;

        let refund = deposit_amount
            .checked_sub(spent_amount)
            .ok_or(BillingError::MathOverflow)?;

        if refund > 0 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                b"escrow",
                escrow_job_id.as_bytes(),
                &[escrow_bump],
            ]];

            let cpi_accounts = Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token::transfer(cpi_ctx, refund)?;
        }

        let escrow = &mut ctx.accounts.escrow;
        escrow.status = EscrowStatus::Closed;

        emit!(EscrowClosedEvent {
            job_id: escrow.job_id.clone(),
            network_id: escrow.network_id.clone(),
            total_spent_amount: escrow.spent_amount,
            refund_amount: refund,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = 8 + BillingConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, BillingConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct RegisterNetwork<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + NetworkRegistry::INIT_SPACE,
        seeds = [b"network", network_id.as_bytes()],
        bump
    )]
    pub registry: Account<'info, NetworkRegistry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(network_id: String, provider_id: String)]
pub struct RegisterProvider<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + ProviderRegistry::INIT_SPACE,
        seeds = [b"provider", network_id.as_bytes(), provider_id.as_bytes()],
        bump
    )]
    pub provider: Account<'info, ProviderRegistry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: String, network_id: String, provider_id: String)]
pub struct OpenEscrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ BillingError::InvalidTokenAccount,
        constraint = user_token_account.mint == config.token_mint @ BillingError::InvalidMint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + JobEscrow::INIT_SPACE,
        seeds = [b"escrow", job_id.as_bytes()],
        bump
    )]
    pub escrow: Account<'info, JobEscrow>,

    #[account(
        mut,
        constraint = escrow_token_account.owner == escrow.key() @ BillingError::InvalidTokenAccount,
        constraint = escrow_token_account.mint == config.token_mint @ BillingError::InvalidMint
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, BillingConfig>,

    #[account(
        seeds = [b"network", network_id.as_bytes()],
        bump = network_registry.bump
    )]
    pub network_registry: Account<'info, NetworkRegistry>,

    #[account(
        seeds = [b"provider", network_id.as_bytes(), provider_id.as_bytes()],
        bump = provider_registry.bump
    )]
    pub provider_registry: Account<'info, ProviderRegistry>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordUsage<'info> {
    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, JobEscrow>,

    #[account(
        mut,
        constraint = escrow_token_account.owner == escrow.key() @ BillingError::InvalidTokenAccount,
        constraint = escrow_token_account.mint == config.token_mint @ BillingError::InvalidMint
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = provider_token_account.owner == escrow.provider_wallet @ BillingError::InvalidProvider
    )]
    pub provider_token_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, BillingConfig>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CloseEscrow<'info> {
    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, JobEscrow>,

    #[account(
        mut,
        constraint = escrow_token_account.owner == escrow.key() @ BillingError::InvalidTokenAccount,
        constraint = escrow_token_account.mint == config.token_mint @ BillingError::InvalidMint
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.owner == escrow.user @ BillingError::InvalidUser
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, BillingConfig>,

    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct BillingConfig {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub rate_per_unit: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct NetworkRegistry {
    #[max_len(32)]
    pub network_id: String,
    #[max_len(64)]
    pub name: String,
    pub authority: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

impl NetworkRegistry {
    pub const MAX_NETWORK_ID_LEN: usize = 32;
}

#[account]
#[derive(InitSpace)]
pub struct ProviderRegistry {
    #[max_len(64)]
    pub provider_id: String,
    #[max_len(32)]
    pub network_id: String,
    pub provider_wallet: Pubkey,
    pub authority: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

impl ProviderRegistry {
    pub const MAX_PROVIDER_ID_LEN: usize = 64;
}

#[account]
#[derive(InitSpace)]
pub struct JobEscrow {
    #[max_len(64)]
    pub job_id: String,
    #[max_len(32)]
    pub network_id: String,
    #[max_len(64)]
    pub provider_id: String,
    pub user: Pubkey,
    pub provider_wallet: Pubkey,
    pub config: Pubkey,
    pub max_units: u64,
    pub used_units: u64,
    pub deposit_amount: u64,
    pub spent_amount: u64,
    pub status: EscrowStatus,
    pub created_at: i64,
    pub bump: u8,
}

impl JobEscrow {
    pub const MAX_JOB_ID_LEN: usize = 64;
    pub const MAX_NETWORK_ID_LEN: usize = 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum EscrowStatus {
    Open,
    Closed,
}

#[event]
pub struct NetworkRegisteredEvent {
    pub network_id: String,
}

#[event]
pub struct ProviderRegisteredEvent {
    pub provider_id: String,
    pub network_id: String,
    pub wallet: Pubkey,
}

#[event]
pub struct EscrowOpenedEvent {
    pub job_id: String,
    pub network_id: String,
    pub user: Pubkey,
    pub provider_wallet: Pubkey,
    pub deposit_amount: u64,
    pub max_units: u64,
}

#[event]
pub struct UsageRecordedEvent {
    pub job_id: String,
    pub network_id: String,
    pub units: u64,
    pub amount: u64,
    pub total_used_units: u64,
    pub total_spent_amount: u64,
}

#[event]
pub struct EscrowClosedEvent {
    pub job_id: String,
    pub network_id: String,
    pub total_spent_amount: u64,
    pub refund_amount: u64,
}

#[error_code]
pub enum BillingError {
    #[msg("Invalid job id")]
    InvalidJobId,
    #[msg("Invalid network id")]
    InvalidNetworkId,
    #[msg("Invalid provider id")]
    InvalidProviderId,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid name")]
    InvalidName,
    #[msg("Invalid units")]
    InvalidUnits,
    #[msg("Usage exceeds max limit")]
    UsageExceedsLimit,
    #[msg("Escrow has insufficient funds")]
    InsufficientEscrow,
    #[msg("Escrow already closed")]
    EscrowClosed,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid provider")]
    InvalidProvider,
    #[msg("Invalid user")]
    InvalidUser,
    #[msg("Invalid network")]
    InvalidNetwork,
    #[msg("Provider not in network")]
    ProviderNotInNetwork,
}
