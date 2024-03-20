// Import necessary crates
use anchor_lang::prelude::*;
use solana_program::clock::Clock;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        self, 
        Mint, 
        MintTo, 
        Token, 
        TokenAccount, 
        Transfer, 
        transfer,
        spl_token::instruction::AuthorityType,
        FreezeAccount,
        freeze_account,
        ThawAccount,
        thaw_account, 
        SetAuthority,
        set_authority},
};

// Program ID
declare_id!("862wU3Nyrf4H3mxo514hrqvgDhRL7DVfUaCpHbifUYUi");

// Define constant seeds for the stake program account derivations
pub mod constants {
    pub const TOKEN_VAULT_SEED: &[u8] = b"token-vault";
    pub const TOKEN_MINT_SEED: &[u8] = b"token-mint";
    pub const TOKEN_MINT_AUTHORITY_SEED: &[u8] = b"token-mint-authority";
    pub const NFT_STAKE_INFO_SEED: &[u8] = b"stake-details";
    pub const NFT_MINT_SEED: &[u8] = b"nft-mint";
    pub const NFT_MINT_AUTHORITY_SEED: &[u8] = b"nft-mint-authority";
    pub const NFT_STAKE_SEED: &[u8] = b"nft-staked";
    
}

// Define the main program module
#[program]
pub mod nft_staking_program {
    use super::*;

    pub fn initialize_vault(_ctx: Context<InitializeVault>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_token_mint(_ctx: Context<InitializeTokenMint>, _decimals: u8) -> Result<()> {
        Ok(())
    }

    pub fn initialize_nft_mint(_ctx: Context<InitializeNFTMint>) -> Result<()> {
        Ok(())
    }

    pub fn airdrop_token(ctx: Context<AirdropToken>, amount: u64) -> Result<()> {
        // Getting mint authority bump
        let mint_authority_bump = ctx.bumps.mint_authority;

        // Creating the signer by referencing the seeds and bump
        let mint_authority_seeds = &[constants::TOKEN_MINT_AUTHORITY_SEED, &[mint_authority_bump]];
        let signer = &[&mint_authority_seeds[..]];

        // Convert the amount to proper decimal format of the token mint
        let airdrop_amount = (amount)
            .checked_mul(10u64.pow(ctx.accounts.token_mint.decimals as u32))
            .unwrap();

        msg!("Airdropping {} tokens.....", airdrop_amount);

        let mint_to_ctx = ctx.accounts.mint_to_ctx().with_signer(signer);
        token::mint_to(mint_to_ctx, airdrop_amount)?;

        msg!("Airdrop complete!");

        Ok(())
    }

    pub fn airdrop_nft(ctx: Context<AirdropNFT>) -> Result <()> {
        // Getting the nft mint authority bump
        let nft_mint_authority_bump = ctx.bumps.nft_mint_authority;

        // Creating the signer by referencing the seeds and bump
        let nft_mint_authority_seeds = &[constants::NFT_MINT_AUTHORITY_SEED, &[nft_mint_authority_bump]];
        let signer = &[&nft_mint_authority_seeds[..]];

        msg!("Airdropping NFT! ...... ");

        // Minting 1 token to user
        let nft_mint_to_ctx = ctx.accounts.nft_mint_to_ctx().with_signer(signer);
        token::mint_to(nft_mint_to_ctx, 1)?;
        
        // Setting the MintTokens authority of the mint to None; no more tokens will be minted.
        let set_authority_ctx = ctx.accounts.set_authority_ctx().with_signer(signer);
        token::set_authority(set_authority_ctx, AuthorityType::MintTokens, None)?;

        msg!("Airdrop complete!");

        Ok(())
    }

    pub fn stake_nft(ctx: Context<StakeNFT>) -> Result <()> {
        let nft_stake_info = &mut ctx.accounts.nft_stake_info_account;

        if nft_stake_info.is_staked {
            return Err(ErrorCode::IsStaked.into());
        }

        let clock = Clock::get()?;

        nft_stake_info.stake_at_slot = clock.slot;
        nft_stake_info.is_staked = true;

        // Getting the nft mint authority bump
        let nft_mint_authority_bump = ctx.bumps.nft_mint_authority;

        // Creating the signer by referencing the seeds and bump
        let nft_mint_authority_seeds = &[constants::NFT_MINT_AUTHORITY_SEED, &[nft_mint_authority_bump]];
        let signer = &[&nft_mint_authority_seeds[..]];

        // Transferring the Freeze authority of the mint to NFT PDA
        let set_freeze_authority_ctx = ctx.accounts.set_freeze_authority_ctx().with_signer(signer);
        token::set_authority(set_freeze_authority_ctx, AuthorityType::FreezeAccount, Some(ctx.accounts.nft_pda_account.key()))?;

        // Getting the nft pda authority bump
        let nft_pda_account_bump = ctx.bumps.nft_pda_account;

        let nft_stake_info_account_key = ctx.accounts.nft_stake_info_account.key();
        let associated_user_nft_account_key = ctx.accounts.associated_user_nft_account.key();

        // Construct the signer
        let nft_pda_account_seeds = &[
            constants::NFT_STAKE_SEED, 
            &nft_stake_info_account_key.as_ref(),
            &associated_user_nft_account_key.as_ref(), 
            &[nft_pda_account_bump]];
        let pda_signer = &[&nft_pda_account_seeds[..]];

        // Freeze the NFT account
        let freeze_token_account_ctx = ctx.accounts.freeze_token_account_ctx().with_signer(pda_signer);
        token::freeze_account(freeze_token_account_ctx)?;

        Ok(())
    }

    pub fn destake_nft(ctx: Context<DestakeNFT>) -> Result<()> {
        let nft_stake_info = &mut ctx.accounts.nft_stake_info_account;

        if !nft_stake_info.is_staked {
            return Err(ErrorCode::NotStaked.into());
        }

        let clock = Clock::get()?;
        let slots_passed = clock.slot - nft_stake_info.stake_at_slot;

        let reward = (slots_passed as u64)
            .checked_mul(10u64.pow(ctx.accounts.token_mint.decimals as u32))
            .unwrap();

        // Transferring of rewards from token vault to user token account
        let token_bump = ctx.bumps.token_vault_account;
        let vault_seeds = &[constants::TOKEN_VAULT_SEED, &[token_bump]];
        let vault_signer = &[&vault_seeds[..]];

        let transfer_token_ctx = ctx.accounts.transfer_token_ctx().with_signer(vault_signer);
        token::transfer(transfer_token_ctx, reward);

        // Getting the nft pda authority bump
        let nft_pda_account_bump = ctx.bumps.nft_pda_account;

        let nft_stake_info_account_key = ctx.accounts.nft_stake_info_account.key();
        let associated_user_nft_account_key = ctx.accounts.associated_user_nft_account.key();

        // Construct the signer
        let nft_pda_account_seeds = &[
            constants::NFT_STAKE_SEED, 
            &nft_stake_info_account_key.as_ref(),
            &associated_user_nft_account_key.as_ref(), 
            &[nft_pda_account_bump]];
        let pda_signer = &[&nft_pda_account_seeds[..]];

        // Thaw the user NFT account
        let thaw_account_ctx = ctx.accounts.thaw_account_ctx().with_signer(pda_signer);
        token::thaw_account(thaw_account_ctx)?;

        // Transfer the Freeze Authority back to NFT mint
        let set_freeze_authority_ctx = ctx.accounts.set_freeze_authority_ctx().with_signer(pda_signer);
        token::set_authority(set_freeze_authority_ctx, AuthorityType::FreezeAccount, Some(ctx.accounts.nft_mint.key()))?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct InitializeTokenMint <'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        mint::authority = token_mint_authority,
        mint::decimals = decimals,
        seeds = [constants::TOKEN_MINT_SEED],
        bump,
        payer = payer
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: Signer
    #[account(
        seeds = [constants::TOKEN_MINT_AUTHORITY_SEED],
        bump,
    )]
    pub token_mint_authority: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeNFTMint <'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        mint::authority = nft_mint_authority,
        mint::decimals = 0,
        mint::freeze_authority = nft_mint_authority,
        seeds = [constants::NFT_MINT_SEED],
        bump,
        payer = payer
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: Signer
    #[account(
        seeds = [constants::NFT_MINT_AUTHORITY_SEED],
        bump,
    )]
    pub nft_mint_authority: AccountInfo<'info>,

    pub rent:Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeVault<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::TOKEN_VAULT_SEED],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AirdropToken<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [constants::TOKEN_MINT_SEED],
        bump
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: Signer
    #[account(
        mut,
        seeds = [constants::TOKEN_MINT_AUTHORITY_SEED],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub associated_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AirdropNFT<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [constants::NFT_MINT_SEED],
        bump
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: Signer
    #[account(
        mut,
        seeds = [constants::NFT_MINT_AUTHORITY_SEED],
        bump
    )]
    pub nft_mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub associated_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct StakeNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::NFT_STAKE_INFO_SEED, payer.key.as_ref(), nft_mint.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<NftStakeInfo>()
    )]
    pub nft_stake_info_account: Account<'info, NftStakeInfo>,

    #[account(
        init,
        seeds = [constants::NFT_STAKE_SEED, nft_stake_info_account.key().as_ref(), associated_user_nft_account.key().as_ref()],
        bump,
        payer = payer,
        mint::authority = nft_pda_account,
        mint::decimals = 0
    )]
    pub nft_pda_account: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = payer
    )]
    pub associated_user_nft_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [constants::NFT_MINT_SEED],
        bump
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: Signer
    #[account(
        mut,
        seeds = [constants::NFT_MINT_AUTHORITY_SEED],
        bump
    )]
    pub nft_mint_authority: AccountInfo<'info>,
   
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>
 }

 #[derive(Accounts)]
 pub struct DestakeNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [constants::NFT_STAKE_INFO_SEED, payer.key.as_ref(), nft_mint.key().as_ref()],
        bump
    )]
    pub nft_stake_info_account: Account<'info, NftStakeInfo>,

    #[account(
        mut,
        seeds = [constants::NFT_STAKE_SEED, nft_stake_info_account.key().as_ref(), associated_user_nft_account.key().as_ref()],
        bump
    )]
    pub nft_pda_account: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [constants::TOKEN_VAULT_SEED],
        bump,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [constants::NFT_MINT_SEED],
        bump
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [constants::TOKEN_MINT_SEED],
        bump
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub associated_user_nft_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub associated_user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
 }

#[account]
pub struct NftStakeInfo {
    pub is_staked: bool,
    pub stake_at_slot: u64
}

impl <'info> AirdropToken <'info> {
    pub fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo{
            mint: self.token_mint.to_account_info(),
            to: self.associated_token_account.to_account_info(),
            authority: self.mint_authority.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl <'info> AirdropNFT <'info> {
    pub fn nft_mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo{
            mint: self.nft_mint.to_account_info(),
            to: self.associated_token_account.to_account_info(),
            authority: self.nft_mint_authority.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn set_authority_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = SetAuthority{
           account_or_mint: self.nft_mint.to_account_info(),
           current_authority: self.nft_mint_authority.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl <'info> StakeNFT <'info> {
    pub fn set_freeze_authority_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = SetAuthority{
            account_or_mint: self.nft_mint.to_account_info(),
            current_authority: self.nft_mint_authority.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn freeze_token_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, FreezeAccount<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = FreezeAccount{
            account: self.associated_user_nft_account.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            authority: self.nft_pda_account.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl <'info> DestakeNFT <'info> {
    pub fn transfer_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>>{
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer{
            from: self.token_vault_account.to_account_info(),
            to: self.associated_user_token_account.to_account_info(),
            authority: self.token_vault_account.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn thaw_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, ThawAccount<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = ThawAccount {
            account: self.associated_user_nft_account.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            authority: self.nft_pda_account.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn set_freeze_authority_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = SetAuthority{
            account_or_mint: self.nft_mint.to_account_info(),
            current_authority: self.nft_pda_account.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("NFT is already staked")]
    IsStaked,

    #[msg("There is no staked NFT found")]
    NotStaked,
}



