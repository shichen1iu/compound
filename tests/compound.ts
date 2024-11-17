import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Compound } from "../target/types/compound";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";
describe("compound", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Compound as Program<Compound>;

  let payer = provider.wallet;
  const collectionA = new PublicKey(
    "BKQFp7a4XuJ1vNvLxgynEC9UTRA74V4xCyhsQEq8TQUr"
  );
  const collectionB = new PublicKey(
    "AgzmJaNWshppdAhig9MkkakzLdvz46WorJ3m7EQPCAb9"
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initVault()
      .accounts({
        payer: payer.publicKey,
        collectionA,
        collectionB,
      })
      .rpc();
    console.log("Your transaction signature", tx);

    // 获取 stake_valut 的 PDA
    const [stakeValutAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_valut")],
      program.programId
    );

    // 获取 reward_mint 的 PDA
    const [rewardMintPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_mint")], // 假设 REWARD_MINT_SEED = "reward_mint"
      program.programId
    );

    const stakeValutInfo = await program.account.stakeValut.fetch(
      stakeValutAddress
    );
    console.log(stakeValutInfo);
    assert.equal(
      stakeValutInfo.rewardMint.toString(),
      rewardMintPDA.toString()
    );
    assert.equal(stakeValutInfo.collectionA.toString(), collectionA.toString());
    assert.equal(stakeValutInfo.collectionB.toString(), collectionB.toString());
  });
});
