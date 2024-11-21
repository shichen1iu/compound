import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Compound } from "../target/types/compound";
import { PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";
import base58 from "bs58";

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
  const staker = require("./staker.json");
  const stakerKeypair = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(base58.decode(staker.staker))
  );

  const assetA = new PublicKey("oicUPh1gVBav6bDrU8eYnLscb9MfTMxGhC849JcDG47");
  const assetB = new PublicKey("ET7SJakh3cjb2hAmkqRUdmDPAeCQHeV1TB5XmPJm6RAp");

  const compoundCollection = Keypair.generate();
  const compoundAsset = Keypair.generate();
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

    console.log("reward Mint Address", rewardMintPDA.toString());

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

  it("stake asset", async () => {
    const stakeAssetTx = await program.methods
      .stakeAsset(
        "Gilgamesh",
        "https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmSkBvu5k5EbEVMTe9MPjRyDS1PPeW83VFBJ9pPPKG8hQV"
      )
      .accounts({
        assetA: assetA,
        assetB: assetB,
        staker: stakerKeypair.publicKey,
        compoundAsset: compoundAsset.publicKey,
      })
      .signers([stakerKeypair, compoundAsset])
      .rpc();
    console.log("stake asset tx signature", stakeAssetTx);
    console.log("compound asset address", compoundAsset.publicKey.toString());

    const [stakeDetialsPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("stake_details"),
        stakerKeypair.publicKey.toBuffer(),
        assetA.toBuffer(),
        assetB.toBuffer(),
      ],
      program.programId
    );

    const stakeDetialsInfo = await program.account.stakeDetails.fetch(
      stakeDetialsPDA
    );

    let assert_a_currency = stakeDetialsInfo.assetACurrency;
    let assert_b_currency = stakeDetialsInfo.assetBCurrency;
    console.log("assert_a_currency", assert_a_currency);
    console.log("assert_b_currency", assert_b_currency);
    console.log("stake start time", stakeDetialsInfo.startTime);

    try {
      assert.equal(
        stakeDetialsInfo.compoundAsset.toString(),
        compoundAsset.publicKey.toString()
      );
    } catch (error) {
      console.error("Error comparing compound asset addresses:", error);
      throw error;
    }

    assert.equal(stakeDetialsInfo.assetA.toString(), assetA.toString());
    assert.equal(stakeDetialsInfo.assetB.toString(), assetB.toString());
  });
});
