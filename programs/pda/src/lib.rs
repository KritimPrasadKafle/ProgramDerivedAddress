use anchor_lang::prelude::*;

declare_id!("J313EyE2LugwZnBZcdrckaaGxduGpLR3Nqtkar7PqBMc");

#[program]
pub mod pda {
    use super::*;

    pub fn create(ctx: Context<Create>) -> Result<()> {
        Ok(())
    }

    pub fn update(ctx: Context<Update>) -> Result<()> {
        Ok(())
    }

    pub fn delete(ctx: Context<Delete>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Create {}

#[derive(Accounts)]
pub struct Update {}

#[derive(Accounts)]
pub struct Delete {}
