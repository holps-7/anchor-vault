use anchor_lang::prelude::*;

declare_id!("2GKEFVyeUV7v2PN9NWSJp6aE4RnowhQS6sr5y4ranuMw");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
