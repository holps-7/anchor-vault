import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVault } from "../target/types/anchor_vault";

describe("anchor-vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.getProvider()
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.anchorVault as Program<AnchorVault>;

  let vaultStatePda: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let vaultStateBump: number;
  let vaultBump: number;

  before(async () => {
    //// Airdrop SOL to user
    //await provider.connection.confirmTransaction(
    //  await provider.connection.requestAirdrop(provider.wallet.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL),
    //  "confirmed"
    //);

    // Derive PDAs
    [vaultStatePda, vaultStateBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );
    [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultStatePda.toBuffer()],
      program.programId
    );
  });

  it("Initializes vault for user", async () => {
    const txn = await program.methods
      .initialize()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log(`init txnHash: ${txn}`);

    // Fetch and check vault state
    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    if (!vaultState) throw new Error("Vault state not initialized");
  });

  it("Deposits into the vault", async () => {
    const amount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
    const txn = await program.methods.deposit(amount)
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();
    
    console.log(`deposit ${amount} txnHash: ${txn}`);
  });

  it("Withdraws from the vault", async () => {
    const amount = new anchor.BN(0.2 * anchor.web3.LAMPORTS_PER_SOL);
    const txn = await program.methods.withdraw(amount)
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId
      }).rpc();

    console.log(`withdraw ${amount} txnHash: ${txn}`);
  });

  it("Fails to withdraw 0 balance", async () => {
    let threw = false;
    try {
      const amount = new anchor.BN(0);
      const txn = await program.methods.withdraw(amount)
        .accountsPartial({
          user: provider.wallet.publicKey,
          vaultState: vaultStatePda,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        }).rpc();

      console.log(`withdraw ${amount} txnHash: ${txn}`);
    } catch (error) {
      threw = true;
      console.log(`Error: ${error}`);
    }
    if (!threw) throw new Error("Should not allow overdraw");
  });

  it("Fails to withdraw more than vault balance", async () => {
    let threw = false;
    try {
      const amount = new anchor.BN(10 * anchor.web3.LAMPORTS_PER_SOL);
      const txn = await program.methods.withdraw(amount)
        .accountsPartial({
          user: provider.wallet.publicKey,
          vaultState: vaultStatePda,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        }).rpc();

      console.log(`over withdraw ${amount} txnHash: ${txn}`);
    } catch (error) {
      threw = true;
      console.log(`Error: ${error}`);
    }
    if (!threw) throw new Error("Should not allow overdraw");
  });

  it("Withdraws all and closes the vault", async () => {
    const txn = await program.methods.withdrawAndClose()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();
    
      console.log(`withdraw_and_close txnHash: ${txn}`);

    // Vault state account should be closed
    let closed = false;
    try {
      await program.account.vaultState.fetch(vaultStatePda);
    } catch (error) {
      closed = true;
      console.log(`Successfully closed`);
    }
    if (!closed) throw new Error("Vault state should be closed");
  });
});
