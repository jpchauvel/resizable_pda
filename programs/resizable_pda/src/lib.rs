use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("FEMohQcaSFUQ5tQ1povr5iUYB5NngZ5g6vCJy7ae9Nbo");

#[program]
pub mod resizable_pda {
    use super::*;

    pub fn create_account(ctx: Context<CreatePDA>, nonce: u64, message: String) -> Result<()> {
        let pda = &mut ctx.accounts.pda_account;
        pda.authority = ctx.accounts.user.key();
        pda.nonce = nonce;
        pda.data = message.into_bytes(); // Store the message in data

        msg!(
            "Created PDA with message: {}",
            String::from_utf8_lossy(&pda.data)
        );
        Ok(())
    }

    pub fn resize_account(ctx: Context<ResizePDA>, new_size: u64) -> Result<()> {
        let account_info = &mut ctx.accounts.pda_account.to_account_info();
        let old_size = account_info.try_data_len()? as u64;
        let new_data_size = 8 + 32 + 8 + 4 + new_size as u64;

        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(new_data_size as usize);

        if new_data_size > old_size {
            let required_lamports = new_minimum_balance.saturating_sub(account_info.lamports());

            // Log current balances for inspection
            msg!(
                "Current Authority Lamports: {}",
                ctx.accounts.authority.lamports()
            );
            msg!("Current PDA Account Lamports: {}", account_info.lamports());
            msg!("Required for resize: {}", required_lamports);

            if required_lamports > 0 {
                require!(
                    ctx.accounts.authority.lamports() >= required_lamports,
                    ErrorCode::InsufficientFunds
                );

                let cpi_accounts = Transfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to: account_info.clone(),
                };
                let cpi_context =
                    CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
                system_program::transfer(cpi_context, required_lamports)?;

                msg!(
                    "After transfer Authority Lamports: {}",
                    ctx.accounts.authority.lamports()
                );
                msg!(
                    "After transfer PDA Account Lamports: {}",
                    account_info.lamports()
                );
            }
        } else if new_data_size < old_size {
            // Resize smaller, refunding lamports
            let refund = account_info.lamports().saturating_sub(new_minimum_balance);
            **ctx.accounts.authority.try_borrow_mut_lamports()? += refund;
            **account_info.try_borrow_mut_lamports()? -= refund;
            msg!("Decreasing size, refunding lamports: {}", refund);
        }

        account_info.realloc(new_data_size as usize, false)?;

        let pda_data = &mut &mut ctx.accounts.pda_account.data;
        pda_data.resize(new_size as usize, 0);

        msg!("Resized PDA from {} to {} bytes", old_size, new_size);
        Ok(())
    }

    pub fn update_data(ctx: Context<UpdatePDA>, new_message: String) -> Result<()> {
        let pda = &mut ctx.accounts.pda_account;
        let new_data = new_message.into_bytes();

        require!(new_data.len() <= pda.data.len(), ErrorCode::DataTooLarge);

        pda.data.clear(); // Clear the existing data
        pda.data.extend(new_data); // Add new data

        msg!(
            "Updated PDA with new message: {}",
            String::from_utf8_lossy(&pda.data)
        );
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

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
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
    #[msg("Insufficient funds: Not enough lamports to complete account resizing.")]
    InsufficientFunds,
}
