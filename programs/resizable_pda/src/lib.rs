use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, CreateAccount, Transfer};
use anchor_lang::solana_program::system_instruction;

declare_id!("FEMohQcaSFUQ5tQ1povr5iUYB5NngZ5g6vCJy7ae9Nbo");

#[program]
pub mod resizable_pda {
    use super::*;

    /// Creates a new PDA account with an initial message.
    pub fn create_account(ctx: Context<CreatePDA>, nonce: u64, message: String) -> Result<()> {
        let pda = &mut ctx.accounts.pda_account;
        pda.authority = ctx.accounts.user.key();
        pda.nonce = nonce;
        pda.data = message.into_bytes(); // Store the message in data

        msg!("Created PDA with message: {}", String::from_utf8_lossy(&pda.data));
        Ok(())
    }

    /// Resizes the PDA's data buffer.
    pub fn resize_account(ctx: Context<ResizePDA>, new_size: u64) -> Result<()> {
        let pda = &mut ctx.accounts.pda_account;

        let old_size = pda.data.len();
        pda.data.resize(new_size as usize, 0);

        msg!("Resized PDA from {} to {} bytes", old_size, new_size);
        Ok(())
    }

    /// Updates the PDA's stored message.
    pub fn update_data(ctx: Context<UpdatePDA>, new_message: String) -> Result<()> {
        let pda = &mut ctx.accounts.pda_account;
        let new_data = new_message.into_bytes();

        require!(new_data.len() <= pda.data.len(), ErrorCode::DataTooLarge);
        pda.data[..new_data.len()].copy_from_slice(&new_data);

        msg!("Updated PDA with new message: {}", String::from_utf8_lossy(&pda.data));
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(nonce: u64, message: String)]
pub struct CreatePDA<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 4 + message.len(), // Base size + message size
        seeds = [b"my-seed", user.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub pda_account: Account<'info, PDAAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResizePDA<'info> {
    #[account(mut, has_one = authority)]
    pub pda_account: Account<'info, PDAAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePDA<'info> {
    #[account(mut, has_one = authority)]
    pub pda_account: Account<'info, PDAAccount>,

    pub authority: Signer<'info>,
}

#[account]
pub struct PDAAccount {
    pub authority: Pubkey,
    pub nonce: u64,
    pub data: Vec<u8>, // Stores the message as bytes
}

#[error_code]
pub enum ErrorCode {
    #[msg("New message is too large for the allocated space.")]
    DataTooLarge,
}
