// Import everything from Anchor's prelude:
// macros, traits, types like Account, Signer, Program, Context, Result, etc.
use anchor_lang::prelude::*;

// Declares the on-chain address of this program.
// Solana verifies the deployed binary matches this ID.
// Every program has a unique address — think of it as the program's identity.
declare_id!("J313EyE2LugwZnBZcdrckaaGxduGpLR3Nqtkar7PqBMc");

// The #[program] macro marks this module as containing instruction handlers.
// Each public function inside becomes a callable instruction from the client (e.g. TypeScript/JS).
#[program]
pub mod pda {
    use super::*; // Bring outer scope (account structs, types) into this module

    // -----------------------------------------------------------------------
    // INSTRUCTION: create
    // Creates a new PDA account and stores the user's message inside it.
    // Called once per user — each user gets their own unique PDA.
    // -----------------------------------------------------------------------
    pub fn create(_ctx: Context<Create>, message: String) -> Result<()> {
        // msg!() prints a log to the transaction output (visible in explorer/logs)
        msg!("Creating message: {}", message);

        // Get a mutable reference to the PDA account we're initializing.
        // The account was already created on-chain by Anchor (via the `init` constraint),
        // here we are just filling in its data fields.
        let __account_data__ = &mut _ctx.accounts.message_account;

        // Store the public key of the signer (the user who called this instruction)
        __account_data__.user = _ctx.accounts.user.key();

        // Store the message string passed in from the client
        __account_data__.message = message;

        // Store the bump seed used to derive this PDA.
        // Anchor automatically finds and provides the bump via `_ctx.bumps`.
        // We save it so future instructions can verify the PDA without re-searching.
        __account_data__.bump = _ctx.bumps.message_account;

        Ok(()) // Return success
    }

    // -----------------------------------------------------------------------
    // INSTRUCTION: update
    // Updates the message stored inside an existing PDA account.
    // The account is resized if the new message is longer or shorter.
    // -----------------------------------------------------------------------
    pub fn update(_ctx: Context<Update>, message: String) -> Result<()> {
        msg!("Updating message: {}", message);

        // Get a mutable reference to the existing PDA account
        let __account_data__ = &mut _ctx.accounts.message_account;

        // Overwrite only the message field — user and bump remain unchanged
        __account_data__.message = message;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // INSTRUCTION: delete
    // Deletes the PDA account and returns its lamports (rent) back to the user.
    // The actual closing of the account is handled by the `close = user`
    // constraint in the Delete accounts struct — not here in the function body.
    // -----------------------------------------------------------------------
    pub fn delete(_ctx: Context<Delete>, message: String) -> Result<()> {
        // Just log the deletion — Anchor's `close` constraint does the real work
        msg!("Deleted Successfully: {}", message);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ACCOUNTS STRUCT: Initialize
// Empty struct — not used by any instruction in this program.
// This is a leftover from Anchor's default project template. Can be removed.
// ---------------------------------------------------------------------------
#[derive(Accounts)]
pub struct Initialize {}

// ---------------------------------------------------------------------------
// ACCOUNTS STRUCT: Create
// Defines and validates all accounts required by the `create` instruction.
// Anchor automatically checks constraints before the instruction runs.
// ---------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(message: String)] // Exposes the `message` argument to account constraints (needed for `space`)
pub struct Create<'info> {

    // The wallet that signs and pays for account creation.
    // `mut` is required because lamports will be deducted from this account.
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,                                          // Create this account on-chain (allocate space + assign owner)
        seeds = [b"message", user.key().as_ref()],    // PDA seeds: literal "message" + user's public key
                                                       // This guarantees each user has their own unique PDA
        bump,                                          // Anchor auto-finds the valid bump seed for this PDA
        payer = user,                                  // `user` pays the rent-exempt SOL deposit
        space = 8 + 32 + 4 + message.len() + 1        // Total byte size of the account:
                                                       //   8  = Anchor discriminator (account type identifier)
                                                       //   32 = Pubkey (user field)
                                                       //   4  = String length prefix (u32)
                                                       //   message.len() = actual message bytes
                                                       //   1  = u8 (bump field)
    )]
    pub message_account: Account<'info, MessageAccount>,

    // Required by Solana whenever a new account is being created.
    // Anchor calls system_program::create_account internally.
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// ACCOUNTS STRUCT: Update
// Defines and validates all accounts required by the `update` instruction.
// ---------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(message: String)] // Exposes `message` to constraints (needed for `realloc` size)
pub struct Update<'info> {

    // Must sign the transaction; also pays/receives lamports if account size changes
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,                                           // Account data will be modified
        seeds = [b"message", user.key().as_ref()],    // Re-derive the PDA to verify it's the correct account
        bump = message_account.bump,                   // Use the stored bump instead of searching again (more efficient)
        realloc = 8 + 32 + 4 + message.len() + 1,    // Resize the account to fit the new message length
                                                       // (new message could be longer or shorter than the original)
        realloc::payer = user,                         // If account grows → user pays extra rent
                                                       // If account shrinks → user receives a refund
        realloc::zero = true,                          // Zero out any newly allocated memory
                                                       // (security best practice — prevents garbage/stale data)
    )]
    pub message_account: Account<'info, MessageAccount>,

    // Required because realloc may transfer lamports between accounts
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// ACCOUNTS STRUCT: Delete
// Defines and validates all accounts required by the `delete` instruction.
// ---------------------------------------------------------------------------
#[derive(Accounts)]
pub struct Delete<'info> {

    // The original creator/owner of the PDA — receives the lamports back on close
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,                                           // Account will be modified (zeroed out and closed)
        seeds = [b"message", user.key().as_ref()],    // Re-derive the PDA to confirm it belongs to this user
        bump = message_account.bump,                   // Use the stored bump for verification
        close = user,                                  // THE KEY CONSTRAINT — this single line:
                                                       //   1. Transfers all lamports from message_account → user
                                                       //   2. Zeroes out all account data
                                                       //   3. Removes the account from existence on-chain
    )]
    pub message_account: Account<'info, MessageAccount>,
}

// ---------------------------------------------------------------------------
// ACCOUNT DATA STRUCT: MessageAccount
// Defines the data layout stored inside our PDA account.
// ---------------------------------------------------------------------------
#[account] // Anchor macro that:
           //   - Adds an 8-byte discriminator prefix (unique fingerprint for this account type)
           //   - Implements Borsh serialization/deserialization automatically
pub struct MessageAccount {
    pub user: Pubkey,    // 32 bytes — public key of the user who owns this account
    pub message: String, // 4 + n bytes — the stored message (length-prefixed string)
    pub bump: u8,        // 1 byte — saved PDA bump seed for efficient future verification
}