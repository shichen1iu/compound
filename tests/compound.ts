import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Compound } from "../target/types/compound";
import { PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";
describe("compound", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Compound as Program<Compound>;

  let payer = provider.wallet;
  const collectionA = new PublicKey(
    "2BmLWt3kos1cqcakhoyEXXET5rywA3TmCU3nPDysoSj7"
  );
  const collectionB = new PublicKey(
    "CQRnfDn2iLnEf8oiA8Nr9HkNNwufSoNH7f4LhQtCmQwn"
  );
  const compoundCollection = Keypair.generate();

  it("init vault", async () => {
    const tx = await program.methods
      .initVault(
        "Gilgamesh",
        "https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmSkBvu5k5EbEVMTe9MPjRyDS1PPeW83VFBJ9pPPKG8hQV",
        500
      )
      .accounts({
        payer: payer.publicKey,
        collectionA,
        collectionB,
        compoundCollection: compoundCollection.publicKey,
      })
      .signers([compoundCollection])
      .rpc();
    console.log("Your transaction signature", tx);

    // 获取 stake_valut 的 PDA
    const [stakeVaultAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_vault")],
      program.programId
    );

    // 获取 reward_mint 的 PDA
    const [rewardMintPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_mint")], // 假设 REWARD_MINT_SEED = "reward_mint"
      program.programId
    );

    const stakeVaultInfo = await program.account.stakeVault.fetch(
      stakeVaultAddress
    );
    // console.log(stakeValutInfo);

    console.log(
      "compound collection address : ",
      stakeVaultInfo.compoundCollection.toString()
    );
    assert.equal(
      stakeVaultInfo.rewardMint.toString(),
      rewardMintPDA.toString()
    );
    assert.equal(stakeVaultInfo.collectionA.toString(), collectionA.toString());
    assert.equal(stakeVaultInfo.collectionB.toString(), collectionB.toString());
  });
});
