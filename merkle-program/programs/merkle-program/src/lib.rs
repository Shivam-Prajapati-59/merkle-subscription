use anchor_lang::prelude::*;

declare_id!("FzA5z58t4RWgmDEEoHTrCnfXeZ4rgtnhU3P684J8rPdx");

#[program]
pub mod merkle_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
