use anchor_lang::prelude::*;

declare_id!("8GLXTjvLQGTQY5Pwagn6FBQ7QZY42mcDvAktGhmKTZpR");

#[program]
pub mod solziocat {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
