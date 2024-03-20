import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStakingProgram } from "../target/types/nft_staking_program";
import { Keypair, SystemProgram, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  TOKEN_PROGRAM_ID, 
  createRecoverNestedInstruction, 
  getAccount, 
  getAssociatedTokenAddressSync, 
  getOrCreateAssociatedTokenAccount } 
from "@solana/spl-token";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";

describe("nft-staking-program", async () => {
  // Configure the client to use the local cluster and establish connection
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const connection = provider.connection;
  const program = anchor.workspace.NftStakingProgram as Program<NftStakingProgram>;
  const wallet = provider.wallet as anchor.Wallet;

  // Derive PDA of token mint and mint authority using the seeds
  const tokenMint = PublicKey.findProgramAddressSync([Buffer.from("token-mint")], program.programId);
  const tokenmintAuthority = PublicKey.findProgramAddressSync([Buffer.from("token-mint-authority")], program.programId);

  // Derive PDA of NFT mint and its mint authority using the seeds
  const nftMint = PublicKey.findProgramAddressSync([Buffer.from("nft-mint")], program.programId);
  const nftmintAuthority = PublicKey.findProgramAddressSync([Buffer.from("nft-mint-authority")], program.programId);

  // Derive PDA oof the vault account using the seeds
  let [vaultAccount] = PublicKey.findProgramAddressSync([Buffer.from("token-vault")], program.programId);

  // Generate a random user keypair for user transactions
  let userAccount = await Keypair.generate();
  let userAccountPk = userAccount.publicKey;

  // Airdrop 2 SOL to User Wallet
  it("Airdrops 2 SOL to user account", async () => {
    const tx = await connection.requestAirdrop(userAccountPk, 2*LAMPORTS_PER_SOL);

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Your transaction signature", tx);
  });
  
  // Creates a customized NFT mint program using SPL standard thru smart contract
  it("Creates NFT Mint", async () => {
    const tx = await program.methods.initializeNftMint()
    .accounts({
      payer: wallet.publicKey,
      nftMint: nftMint[0],
      nftMintAuthority: nftmintAuthority[0]
    })
    .rpc()

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Initialize NFT mint tx: ", tx);
  })

  // Airdrops NFT to the user
  it ("Airdrops NFT", async () => {
    let usernftATA = await getOrCreateAssociatedTokenAccount(
      connection,
      userAccount,
      nftMint[0],
      userAccountPk
    );

    const tx = await program.methods.airdropNft()
    .accounts({
      payer: wallet.publicKey,
      nftMint: nftMint[0],
      nftMintAuthority: nftmintAuthority[0],
      associatedTokenAccount: usernftATA.address
    })
    .rpc()

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Airdrop NFT tx: ", tx);
  })

  // Creates a customized token mint program using smart contract
  it("Creates Token Mint", async () => {
    const tx = await program.methods.initializeTokenMint(10)
    .accounts({
      tokenMint: tokenMint[0],
      tokenMintAuthority: tokenmintAuthority[0],
      payer: wallet.publicKey
    })
    .rpc()

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Initialize token mint tx:", tx);
  })

  // Initialize the token vault of the staking program
  it("Initialize Staking Token Vault", async () => {
    const tx = await program.methods.initializeVault()
    .accounts({
      payer: wallet.publicKey,
      tokenVaultAccount: vaultAccount,
      mint: tokenMint[0],
    })
    .rpc()

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Initialize Token Vault tx: ", tx);
  })

  // Airdrops 500 token to the token vault of the staking program
  it("Airdrops 500 token to the Staking Token Vault", async () => {
    const tx = await program.methods.airdropToken(new anchor.BN(500))
    .accounts({
      payer: wallet.publicKey,
      tokenMint: tokenMint[0],
      mintAuthority: tokenmintAuthority[0],
      associatedTokenAccount: vaultAccount
    })
    .rpc()

    let token_vault_balance = await connection.getTokenAccountBalance(vaultAccount);

    console.log("Vault Token Balance: ", parseInt(token_vault_balance.value.amount)/1e10);

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Airdrop 500 tokens to the staking token vault tx:", tx);
  })

  // Stakes NFT
  it("Stakes NFT successfully", async () => {
    let usernftATA = await getAssociatedTokenAddressSync(
      nftMint[0],
      userAccountPk
    );

    console.log("NFT ATA: ", usernftATA);

    let [nftStakeInfo] = PublicKey.findProgramAddressSync([
      Buffer.from("stake-details"),
      userAccountPk.toBuffer(),
      nftMint[0].toBuffer()],
      program.programId
    );

    let [nftPda] = PublicKey.findProgramAddressSync([
      Buffer.from("nft-staked"),
      nftStakeInfo.toBuffer(),
      usernftATA.toBuffer()],
      program.programId
    );
    
    const tx = await program.methods.stakeNft()
    .accounts({
      payer: userAccountPk,
      nftMint: nftMint[0],
      nftMintAuthority: nftmintAuthority[0],
      associatedUserNftAccount: usernftATA,
      nftStakeInfoAccount: nftStakeInfo,
      nftPdaAccount: nftPda,
    })
    .signers([userAccount])
    .rpc()

    const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx,
      });

    console.log("Stake tx: ", tx);
  })

  it("Destakes NFT successfully", async () => {
    let usernftATA = await getAssociatedTokenAddressSync(
      nftMint[0],
      userAccountPk
    );

    let userTokenATA = await getOrCreateAssociatedTokenAccount(
      connection,
      userAccount,
      tokenMint[0],
      userAccountPk
    )

    let [nftStakeInfo] = PublicKey.findProgramAddressSync([
      Buffer.from("stake-details"),
      userAccountPk.toBuffer(),
      nftMint[0].toBuffer()],
      program.programId
    );

    let [nftPda] = PublicKey.findProgramAddressSync([
      Buffer.from("nft-staked"),
      nftStakeInfo.toBuffer(),
      usernftATA.toBuffer()],
      program.programId
    );

    const tx = await program.methods.destakeNft()
    .accounts({
      payer: userAccountPk,
      nftStakeInfoAccount: nftStakeInfo,
      nftPdaAccount: nftPda,
      nftMint: nftMint[0],
      tokenMint: tokenMint[0],
      tokenVaultAccount: vaultAccount,
      associatedUserNftAccount: usernftATA,
      associatedUserTokenAccount: userTokenATA.address,
    })
    .signers([userAccount])
    .rpc()

    console.log("Destake tx: ", tx);

  })

});
