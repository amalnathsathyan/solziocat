
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
declare_id!("8GLXTjvLQGTQY5Pwagn6FBQ7QZY42mcDvAktGhmKTZpR");

#[program]
pub mod rebase_token {
/**
 * @title SolzioCat
 * @notice Implementation of the SOLZIO token.
 * @dev SOLZIO is a rebasable token, which means that the total supply can be updated. Its particularity is that
 * every 4 days the supply is divided by 2, and in between the supply is constantly decreasing linearly. The balances
 * stays fixed for 34 minutes. Then every 34 minutes, the supply is updated, and the balances are updated
 * proportionally.
 * At each rebase, 13.37% of the debased supply is sent to the feesCollector.
 */
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, initial_supply: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.total_supply = initial_supply;
        state.last_rebase = Clock::get()?.unix_timestamp;
        state.rebase_interval = 4 * 24 * 60 * 60 + 144; // 4 days + 144 seconds

        // Mint initial supply to the creator
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.creator_token_account.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
                &[&[b"mint_authority", &[*ctx.bumps.get("mint").unwrap()]]],
            ),
            initial_supply,
        )?;

        Ok(())
    }

    pub fn rebase(ctx: Context<Rebase>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let current_time = Clock::get()?.unix_timestamp;

        if current_time - state.last_rebase < state.rebase_interval {
            return Err(ErrorCode::TooEarlyForRebase.into());
        }

        // Calculate new supply (half of current supply)
        let new_supply = state.total_supply / 2;
        let amount_to_burn = state.total_supply - new_supply;

        // Burn tokens to reduce supply
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.burn_account.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
                &[&[b"mint_authority", &[*ctx.bumps.get("mint").unwrap()]]],
            ),
            amount_to_burn,
        )?;

        state.total_supply = new_supply;
        state.last_rebase = current_time;

        Ok(())
    }

    // Add other necessary instructions (transfer, etc.)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = creator, space = 8 + 8 + 8 + 8)]
    pub state: Account<'info, RebaseState>,
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Rebase<'info> {
    #[account(mut)]
    pub state: Account<'info, RebaseState>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = burn_authority,
    )]
    pub burn_account: Account<'info, TokenAccount>,
    pub burn_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct RebaseState {
    pub total_supply: u64,
    pub last_rebase: i64,
    pub rebase_interval: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Too early for rebase")]
    TooEarlyForRebase,
}