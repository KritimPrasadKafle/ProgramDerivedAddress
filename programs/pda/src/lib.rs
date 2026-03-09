use anchor_lang::prelude::*;

declare_id!("J313EyE2LugwZnBZcdrckaaGxduGpLR3Nqtkar7PqBMc");

#[program]
pub mod pda {
    use super::*;

    pub fn create(_ctx: Context<Create>, message: String) -> Result<()> {
        msg!("Creating message: {}", message);
        let account_data = &mut _ctx.accounts.message_account;
        account_data.user = _ctx.accounts.user.key();
        account_data.message = message;
        account_data.bump = _ctx.bumps.message_account;
        Ok(())
    }

    pub fn update(_ctx: Context<Update>) -> Result<()> {
        Ok(())
    }

    pub fn delete(_ctx: Context<Delete>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(message: String)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        seeds = [b"message", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 4 + message.len() + 1
    )]
    pub message_account: Account<'info, MessageAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update {}

#[derive(Accounts)]
pub struct Delete {}

#[account]
pub struct MessageAccount{
    pub user: Pubkey,
    pub message: String,
    pub bump: u8,
}


