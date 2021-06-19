use anchor_lang::prelude::*;
use anchor_lang::{AccountDeserialize, AnchorDeserialize};
use anchor_spl::token::{self, TokenAccount, Transfer};

#[program]
mod exchange {
    use super::*;

    pub fn create_excahnge(ctx: Context<CreateExchange>, price: u64) -> Result<(), ProgramError> {
        let exchange = &mut ctx.accounts.exchange;
        exchange.ongoing = true;
        exchange.seller = *ctx.accounts.seller.key;
        exchange.item_holder = *ctx.accounts.item_holder.to_account_info().key;
        exchange.buyer = *ctx.accounts.seller.key;
        exchange.price = price;
        Ok(())
    }

    pub fn process_exchange(ctx: Context<ProgressExchange>) -> Result<(), ProgramError> {
        let exchange = &mut ctx.accounts.exchange;

        // 货币转账
        let (_, seed) = Pubkey::find_program_address(&[&exchange.seller.to_bytes()], &ctx.program_id);
        let seeds = &[exchange.seller.as_ref(), &[seed]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.currency_receiver.to_account_info().clone(),
            authority: ctx.accounts.from_auth.clone(),
        };

        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        // 完成货币转账
        token::transfer(cpi_ctx, exchange.price)?;

        // NFT转账
        let cpi_accounts = Transfer {
            from: ctx.accounts.item_holder.to_account_info().clone(),
            to: ctx.accounts.item_receiver.to_account_info().clone(),
            authority: ctx.accounts.item_holder_auth.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.item_holder.amount)?;

        exchange.ongoing = false;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateExchange<'info> {
    #[account(init)]
    exchange: ProgramAccount<'info, Exchange>,
    seller: AccountInfo<'info>,
    #[account("&item_holder.owner == &Pubkey::find_program_address(&[&seller.key.to_bytes()], &program_id).0")]
    item_holder: CpiAccount<'info, TokenAccount>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ProgressExchange<'info> {
    #[account(mut, "exchange.ongoing")]
    exchange: ProgramAccount<'info, Exchange>,
    #[account(signer)]
    buyer: AccountInfo<'info>,
    #[account(
        mut,
        "from.mint == currency_receiver.mint",
        "&from.owner == from_auth.key",
    )]
    from: CpiAccount<'info, TokenAccount>,
    from_auth: AccountInfo<'info>,
    item_holder: CpiAccount<'info, TokenAccount>,
    item_holder_auth: AccountInfo<'info>,
    item_receiver: CpiAccount<'info, TokenAccount>,
    currency_receiver: CpiAccount<'info, TokenAccount>,
    token_program: AccountInfo<'info>,
}

#[account]
pub struct Exchange {
    ongoing: bool,
    seller: Pubkey,
    item_holder: Pubkey,
    buyer:Pubkey,
    price: u64,
}