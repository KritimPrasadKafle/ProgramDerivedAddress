# Solana PDA CRUD — Anchor Program

A beginner-friendly Solana program built with the **Anchor framework** that demonstrates how to **Create, Update, and Delete** a PDA (Program Derived Address) account storing a user message on-chain.

---

## Table of Contents

- [Overview](#overview)
- [What is a PDA?](#what-is-a-pda)
- [Project Structure](#project-structure)
- [Program: lib.rs](#program-librs)
  - [Account Data Layout](#account-data-layout)
  - [Instructions](#instructions)
  - [Account Constraints Explained](#account-constraints-explained)
- [Tests: pda.test.ts](#tests-pdatestts)
  - [Setup](#setup)
  - [Test Cases](#test-cases)
- [Space Calculation](#space-calculation)
- [How PDA Derivation Works](#how-pda-derivation-works)
- [Prerequisites](#prerequisites)
- [Running the Tests](#running-the-tests)
- [Key Concepts Summary](#key-concepts-summary)

---

## Overview

This project demonstrates the core CRUD pattern on Solana using PDAs:

| Operation | Instruction | What It Does |
|-----------|-------------|--------------|
| **Create** | `create(message)` | Derives a PDA, allocates space, stores message on-chain |
| **Update** | `update(message)` | Re-derives PDA, resizes account, overwrites message |
| **Delete** | `delete()` | Re-derives PDA, closes account, returns lamports to user |

Each user gets their **own unique PDA** — derived from their wallet address — so no two users share the same account.

---

## What is a PDA?

A **Program Derived Address (PDA)** is a special account address that:

- Is **deterministically derived** from a set of seeds + the program ID
- Has **no private key** — no one can sign for it from outside
- Can only be written to by **the program that owns it**
- Is perfect for storing per-user or per-entity data on-chain

```
PDA Address = hash(seeds + program_id + bump)

In this project:
seeds = ["message", user_public_key]
```

Because the user's public key is part of the seeds, every user derives a **different PDA** — giving each user their own isolated message account.

---

## Project Structure

```
project/
├── src/
│   └── lib.rs          # Anchor program (Rust)
└── tests/
    └── pda.test.ts     # Integration tests (TypeScript)
```

---

## Program: lib.rs

### Account Data Layout

```rust
#[account]
pub struct MessageAccount {
    pub user: Pubkey,    // Who owns this account (32 bytes)
    pub message: String, // The stored message (4 + n bytes)
    pub bump: u8,        // Saved PDA bump seed (1 byte)
}
```

The `#[account]` macro automatically prepends an **8-byte discriminator** — a unique fingerprint Anchor uses to identify and validate this account type.

---

### Instructions

#### `create(message: String)`

Creates a new PDA account for the calling user and stores a message.

```rust
pub fn create(_ctx: Context<Create>, message: String) -> Result<()>
```

- Allocates a new on-chain account at the derived PDA address
- Saves `user`, `message`, and `bump` into the account
- The **user pays** the rent-exempt SOL deposit

---

#### `update(message: String)`

Updates the message in an existing PDA account.

```rust
pub fn update(_ctx: Context<Update>, message: String) -> Result<()>
```

- Re-derives and verifies the PDA using the stored bump
- **Reallocates** the account size if the new message is longer or shorter
- Only overwrites the `message` field — `user` and `bump` are unchanged

---

#### `delete()`

Closes the PDA account and returns all lamports to the user.

```rust
pub fn delete(_ctx: Context<Delete>, message: String) -> Result<()>
```

- The function body only logs a message
- The actual account closure is handled entirely by the `close = user` constraint in the `Delete` accounts struct — Anchor zeroes the data and transfers lamports automatically

---

### Account Constraints Explained

#### `Create` Struct

```rust
#[account(
    init,                                        // Create the account on-chain
    seeds = [b"message", user.key().as_ref()],  // PDA seeds
    bump,                                        // Auto-find the valid bump
    payer = user,                                // User funds the rent deposit
    space = 8 + 32 + 4 + message.len() + 1      // Allocate exact byte size
)]
pub message_account: Account<'info, MessageAccount>,
```

#### `Update` Struct

```rust
#[account(
    mut,                                         // Account data will change
    seeds = [b"message", user.key().as_ref()],  // Re-derive to verify ownership
    bump = message_account.bump,                 // Use saved bump (no re-search)
    realloc = 8 + 32 + 4 + message.len() + 1,  // Resize to fit new message
    realloc::payer = user,                       // User pays/receives lamport diff
    realloc::zero = true,                        // Zero new memory (security)
)]
pub message_account: Account<'info, MessageAccount>,
```

#### `Delete` Struct

```rust
#[account(
    mut,
    seeds = [b"message", user.key().as_ref()],  // Re-derive to verify ownership
    bump = message_account.bump,                 // Verify with saved bump
    close = user,                                // Transfer lamports → user
                                                 // Zero all data
                                                 // Remove account from chain
)]
pub message_account: Account<'info, MessageAccount>,
```

---

## Tests: pda.test.ts

### Setup

```typescript
const program = pg.program;           // The deployed Anchor program
const wallet = pg.wallet;             // The connected wallet (payer + signer)

const [messagePda, messageBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("message"), wallet.publicKey.toBuffer()],
  program.programId
);
```

- `pg.program` and `pg.wallet` are provided by the **Solana Playground** environment
- `PublicKey.findProgramAddressSync` derives the PDA **client-side** using the same seeds as the program
- The derived `messagePda` is the exact address where the user's message account will live on-chain
- This derivation is **deterministic** — running it again with the same wallet always returns the same address

> **Why derive it on the client?**  
> Anchor only requires you to pass the account address — it re-derives and verifies it internally using the seeds defined in the accounts struct. The client derives it to know *which address to pass*.

---

### Test Cases

#### Test 1: Create Message Account

```typescript
it("Create Message Account", async () => {
  const message = "Hello, World!";

  const transactionSignature = await program.methods
    .create(message)        // Call the `create` instruction
    .accounts({
      messageAccount: messagePda   // Pass the derived PDA address
    })
    .rpc({ commitment: "confirmed" });  // Wait until tx is confirmed

  const messageAccount = await program.account.messageAccount.fetch(
    messagePda,
    "confirmed"
  );

  console.log(JSON.stringify(messageAccount, null, 2));
  // Expected output:
  // {
  //   "user": "<wallet_pubkey>",
  //   "message": "Hello, World!",
  //   "bump": <bump_number>
  // }
});
```

- Calls the `create` instruction with `"Hello, World!"`
- Fetches and logs the on-chain account data to verify it was stored correctly
- Logs the transaction URL on Solana FM (devnet)

---

#### Test 2: Update Message Account

```typescript
it("Update Message Account", async () => {
  const message = "Hello, Solana!";

  const transactionSignature = await program.methods
    .update(message)        // Call the `update` instruction
    .accounts({
      messageAccount: messagePda
    })
    .rpc({ commitment: "confirmed" });

  const messageAccount = await program.account.messageAccount.fetch(
    messagePda,
    "confirmed"
  );

  console.log(JSON.stringify(messageAccount, null, 2));
  // Expected: message is now "Hello, Solana!" — user and bump unchanged
});
```

- Updates the stored message from `"Hello, World!"` to `"Hello, Solana!"`
- The account is reallocated since the new message has a different byte length
- Fetches and logs the updated account to confirm the change

---

#### Test 3: Delete Message Account

```typescript
it("Delete Message Account", async () => {
  const transactionSignature = await program.methods
    .delete()               // Call the `delete` instruction
    .accounts({
      messageAccount: messagePda
    })
    .rpc({ commitment: "confirmed" });

  const messageAccount = await program.account.messageAccount.fetchNullable(
    messagePda,
    "confirmed"
  );

  console.log("Expect Null:", JSON.stringify(messageAccount, null, 2));
  // Expected output: null — account no longer exists
});
```

- Calls the `delete` instruction to close the PDA account
- Uses `fetchNullable` instead of `fetch` — because after deletion the account no longer exists, and `fetch` would throw an error while `fetchNullable` safely returns `null`
- Logs `null` to confirm the account was successfully removed

---

## Space Calculation

When creating an account, Solana requires you to pre-allocate the exact number of bytes needed:

```
space = 8 + 32 + 4 + message.len() + 1
```

| Bytes | Field | Reason |
|-------|-------|--------|
| `8` | Anchor discriminator | Unique account type identifier prepended by `#[account]` |
| `32` | `user: Pubkey` | Fixed-size public key |
| `4` | String length prefix | Borsh encodes strings as `u32` length + bytes |
| `message.len()` | `message: String` | Variable — actual UTF-8 message content |
| `1` | `bump: u8` | Single byte for the PDA bump seed |

> **Why does this matter?**  
> Solana charges rent proportional to account size. You must allocate exactly what you need upfront. The `realloc` constraint in `Update` handles resizing if the message length changes.

---

## How PDA Derivation Works

```
Client Side (TypeScript):
PublicKey.findProgramAddressSync(
  [Buffer.from("message"), wallet.publicKey.toBuffer()],
  program.programId
)

Program Side (Rust):
seeds = [b"message", user.key().as_ref()]
```

Both use **identical seeds** — Anchor verifies they match. The `bump` is the small number (0–255) subtracted from the hash until a valid off-curve address is found. This is why bumps are saved: finding them costs compute, re-using them is free.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) (v18+)

Or simply use **[Solana Playground](https://beta.solpg.io/)** — no local setup required.

---

## Running the Tests

### Via Solana Playground (Recommended for beginners)

1. Go to [beta.solpg.io](https://beta.solpg.io/)
2. Paste `lib.rs` into the program editor
3. Paste `pda.test.ts` into the test file
4. Click **Build** → **Deploy** → **Test**

### Via Local Anchor CLI

```bash
# Install dependencies
yarn install

# Build the program
anchor build

# Run tests against devnet
anchor test --provider.cluster devnet
```

---

## Key Concepts Summary

| Concept | Description |
|--------|-------------|
| **PDA** | Deterministic account address derived from seeds, owned by the program |
| **Bump seed** | A nonce that ensures the derived address is off the ed25519 curve (no private key) |
| **`init`** | Creates a new account on-chain; fails if account already exists |
| **`realloc`** | Resizes an existing account to fit new data |
| **`close`** | Closes an account, zeroes its data, and returns lamports to a target wallet |
| **Discriminator** | 8-byte prefix Anchor adds to every account to identify its type |
| **`Signer`** | An account that must have signed the transaction |
| **`commitment`** | How finalized a transaction must be before returning (`confirmed` = safe for testing) |
| **`fetchNullable`** | Like `fetch` but returns `null` instead of throwing if the account doesn't exist |