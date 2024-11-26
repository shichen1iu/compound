import * as anchor from "@coral-xyz/anchor";
import { Program, setProvider } from "@coral-xyz/anchor";
import { Compound } from "../target/types/compound";
import {
  PublicKey,
  Keypair,
  Connection,
  Transaction,
  ComputeBudgetProgram,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { assert } from "chai";
import { BankrunProvider } from "anchor-bankrun";
import base58 from "bs58";
import {
  AddedAccount,
  AddedProgram,
  BanksClient,
  Clock,
  ProgramTestContext,
  startAnchor,
} from "solana-bankrun";

describe("compound", () => {
  let context: ProgramTestContext;
  let client: BanksClient;
  let payer: Keypair;
  let provider: BankrunProvider;
  let program: Program<Compound>;
  let connection: Connection;

  const IDL = require("../target/idl/compound.json");

  const collectionAPublicKey = new PublicKey(
    "2BmLWt3kos1cqcakhoyEXXET5rywA3TmCU3nPDysoSj7"
  );
  const collectionBPublicKey = new PublicKey(
    "CQRnfDn2iLnEf8oiA8Nr9HkNNwufSoNH7f4LhQtCmQwn"
  );
  const staker = require("./staker.json");
  const stakerKeypair = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(base58.decode(staker.staker))
  );

  const assetAPublicKey = new PublicKey(
    "7tvjascXmrMQ5CUHHqVSKYkXwTxhykUSREf1teZVPehP"
  );
  const assetBPublicKey = new PublicKey(
    "BFkZuRx1oE82hVgWHfNYgK54UYVtXFvGbebWwLGnVzjH"
  );

  const mplTokenMetadataProgramId = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );

  const mplCoreProgramID = new PublicKey(
    "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
  );

  const compoundProgramID = new PublicKey(
    "ECSZFw8CgLxMmaNHaDJnywcHbbZhE8TRCXnxQA3LGYio"
  );

  let compoundCollection: Keypair;
  let compoundAsset: Keypair;
  before(async () => {
    connection = new Connection(
      "https://devnet.helius-rpc.com/?api-key=47fcd2c1-bfb0-4224-8257-ce200078152a"
    );
    const collectionAInfo = await connection.getAccountInfo(
      collectionAPublicKey
    );
    const collectionBInfo = await connection.getAccountInfo(
      collectionBPublicKey
    );
    const assetAInfo = await connection.getAccountInfo(assetAPublicKey);
    const assetBInfo = await connection.getAccountInfo(assetBPublicKey);
    const stakerInfo = await connection.getAccountInfo(stakerKeypair.publicKey);

    const collectionA: AddedAccount = {
      address: collectionAPublicKey,
      info: collectionAInfo,
    };
    const collectionB: AddedAccount = {
      address: collectionBPublicKey,
      info: collectionBInfo,
    };
    const assetA: AddedAccount = {
      address: assetAPublicKey,
      info: assetAInfo,
    };
    const assetB: AddedAccount = {
      address: assetBPublicKey,
      info: assetBInfo,
    };
    const stakerAccount: AddedAccount = {
      address: stakerKeypair.publicKey,
      info: stakerInfo,
    };

    const mplCoreProgram: AddedProgram = {
      name: "mpl-core",
      programId: mplCoreProgramID,
    };

    const mplTokenMetadataProgram: AddedProgram = {
      name: "mpl-token-metadata",
      programId: mplTokenMetadataProgramId,
    };

    const compoundProgram: AddedProgram = {
      name: "compound",
      programId: compoundProgramID,
    };

    context = await startAnchor(
      "",
      [mplCoreProgram, mplTokenMetadataProgram, compoundProgram],
      [collectionA, collectionB, assetA, assetB, stakerAccount]
    );
    client = context.banksClient;
    payer = context.payer;
    provider = new BankrunProvider(context);
    setProvider(provider);
    program = new Program<Compound>(IDL, provider);

    compoundCollection = Keypair.generate();
    compoundAsset = Keypair.generate();
  });

  it("init vault", async () => {
    const tx = await program.methods
      .initVault(
        "Gilgamesh",
        "https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmSkBvu5k5EbEVMTe9MPjRyDS1PPeW83VFBJ9pPPKG8hQV",
        500
      )
      .accounts({
        payer: payer.publicKey,
        collectionA: collectionAPublicKey,
        collectionB: collectionBPublicKey,
        compoundCollection: compoundCollection.publicKey,
      })
      .signers([compoundCollection])
      .rpc();
    console.log("init vault transaction signature", tx);

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
    assert.equal(
      stakeVaultInfo.collectionA.toString(),
      collectionAPublicKey.toString()
    );
    assert.equal(
      stakeVaultInfo.collectionB.toString(),
      collectionBPublicKey.toString()
    );
  });

  it("stake asset", async () => {
    console.log("staker keypair", stakerKeypair.publicKey.toString());
    const stakeAssetTx = await program.methods
      .stakeAsset(
        "Gilgamesh",
        "https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmSkBvu5k5EbEVMTe9MPjRyDS1PPeW83VFBJ9pPPKG8hQV",
        1000,
        1500
      )
      .accounts({
        assetA: assetAPublicKey,
        assetB: assetBPublicKey,
        staker: stakerKeypair.publicKey,
        compoundAsset: compoundAsset.publicKey,
      })
      .signers([stakerKeypair, compoundAsset])
      .rpc();
    console.log("stake asset tx signature", stakeAssetTx);
    console.log("compound asset address", compoundAsset.publicKey.toString());

    const [stakeDetailsPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("stake_details"),
        stakerKeypair.publicKey.toBuffer(),
        assetAPublicKey.toBuffer(),
        assetBPublicKey.toBuffer(),
      ],
      program.programId
    );

    console.log("stake_details address", stakeDetailsPDA.toString());

    const stakeDetialsInfo = await program.account.stakeDetails.fetch(
      stakeDetailsPDA
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

    assert.equal(
      stakeDetialsInfo.assetA.toString(),
      assetAPublicKey.toString()
    );
    console.log("stakeDetialsInfo.assetA", stakeDetialsInfo.assetA.toString());
    assert.equal(
      stakeDetialsInfo.assetB.toString(),
      assetBPublicKey.toString()
    );
    console.log("stakeDetialsInfo.assetB", stakeDetialsInfo.assetB.toString());
  });

  describe("unstake asset", () => {
    // 第一种情况：质押时间少于7天
    describe("when unstake time is less than 7 days", () => {
      before(async () => {
        let currentClock = await client.getClock();
        let startEndTime =
          currentClock.epochStartTimestamp + BigInt(5 * 24 * 60 * 60);
        context.setClock(
          new Clock(
            currentClock.slot,
            currentClock.epochStartTimestamp,
            currentClock.epoch,
            currentClock.leaderScheduleEpoch,
            startEndTime
          )
        );
      });

      it("should fail to unstake after 5 days", async () => {
        const unstakeAssetTx = await program.methods
          .unstakeAsset()
          .accounts({
            staker: stakerKeypair.publicKey,
            assetA: assetAPublicKey,
            assetB: assetBPublicKey,
          })
          .signers([stakerKeypair])
          .rpc();
        console.log("unstake asset tx signature", unstakeAssetTx);
      });
    });

    // 第二种情况：质押时间超过7天
    describe("when unstake time is more than 7 days", () => {
      before(async () => {
        let currentClock = await client.getClock();
        let startEndTime =
          currentClock.epochStartTimestamp + BigInt(8 * 24 * 60 * 60);
        context.setClock(
          new Clock(
            currentClock.slot,
            currentClock.epochStartTimestamp,
            currentClock.epoch,
            currentClock.leaderScheduleEpoch,
            startEndTime
          )
        );
      });

      it("should successfully unstake after 8 days", async () => {
        const [stakeDetailsPDA, stakeDetailsPDA_Bump] =
          PublicKey.findProgramAddressSync(
            [
              Buffer.from("stake_details"),
              stakerKeypair.publicKey.toBuffer(),
              assetAPublicKey.toBuffer(),
              assetBPublicKey.toBuffer(),
            ],
            program.programId
          );

        console.log("stake_details address", stakeDetailsPDA.toString());
        console.log("stake_details bump", stakeDetailsPDA_Bump);

        try {
          // 检查账户是否存在
          const accountInfo = await program.provider.connection.getAccountInfo(
            stakeDetailsPDA
          );
          console.log("Stake Details Account exists:", !!accountInfo);

          if (accountInfo) {
            // 如果账户存在，获取账户数据
            const stakeDetails = await program.account.stakeDetails.fetch(
              stakeDetailsPDA
            );
            console.log("Stake Details data:", {
              bump: stakeDetails.bump,
              startTime: stakeDetails.startTime.toString(),
              assetA: stakeDetails.assetA.toString(),
              assetB: stakeDetails.assetB.toString(),
              compoundAsset: stakeDetails.compoundAsset.toString(),
              isStaked: stakeDetails.isStaked,
            });
          }
        } catch (e) {
          console.error("Error checking stake details account:", e);
        }

        const unstakeAssetIx = await program.methods
          .unstakeAsset()
          .accounts({
            staker: stakerKeypair.publicKey,
            assetA: assetAPublicKey,
            assetB: assetBPublicKey,
          })
          .signers([stakerKeypair])
          .instruction();
        const blockhashContext = await connection.getLatestBlockhash();

        let unstakeAssetTx = new Transaction({
          blockhash: blockhashContext.blockhash,
          lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
        })
          .add(unstakeAssetIx)
          .add(ComputeBudgetProgram.setComputeUnitLimit({ units: 230_000 }));

        await sendAndConfirmTransaction(
          connection,
          unstakeAssetTx,
          [stakerKeypair],
          {
            commitment: "confirmed",
          }
        );
        console.log("unstake asset tx signature", unstakeAssetTx);
      });
    });
  });
});
