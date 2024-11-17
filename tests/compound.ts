import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Compound } from "../target/types/compound";

describe("compound", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Compound as Program<Compound>;

  let payer = provider.wallet;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initVault()
      .accounts({
        payer: payer.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
