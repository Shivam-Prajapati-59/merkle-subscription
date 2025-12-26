import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MerkleProgram } from "../target/types/merkle_program";
import { assert, config, expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createHash } from "crypto";

describe("merkle-program", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.merkleProgram as Program<MerkleProgram>;

  // Derive the config PDA
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  function createLeaf(userPubkey: PublicKey, expiration: number): Buffer {
    const userBytes = userPubkey.toBuffer();
    const expirationBytes = Buffer.alloc(8);
    expirationBytes.writeBigInt64LE(BigInt(expiration));

    return createHash("sha256")
      .update(Buffer.concat([userBytes, expirationBytes]))
      .digest();
  }

  function buildMerkleTree(leaves: Buffer[]): {
    root: Buffer;
    proofs: Map<string, Buffer[]>;
  } {
    if (leaves.length === 0) {
      throw new Error("Cannot build tree with no leaves");
    }

    // Sort leaves to ensure deterministic tree structure
    const sortedLeaves = [...leaves].sort(Buffer.compare);

    // Store proofs for each leaf
    const proofs = new Map<string, Buffer[]>();

    // Build tree level by level
    let currentLevel = sortedLeaves;
    const levels: Buffer[][] = [currentLevel];

    // Build up to root
    while (currentLevel.length > 1) {
      const nextLevel: Buffer[] = [];

      for (let i = 0; i < currentLevel.length; i += 2) {
        if (i + 1 < currentLevel.length) {
          // Pair exists
          const left = currentLevel[i];
          const right = currentLevel[i + 1];
          const combined =
            Buffer.compare(left, right) <= 0
              ? Buffer.concat([left, right])
              : Buffer.concat([right, left]);
          const parent = createHash("sha256").update(combined).digest();
          nextLevel.push(parent);
        } else {
          // Odd node, promote to next level
          nextLevel.push(currentLevel[i]);
        }
      }

      levels.push(nextLevel);
      currentLevel = nextLevel;
    }

    const root = currentLevel[0];

    // Generate proofs for each leaf
    for (const leaf of sortedLeaves) {
      const proof: Buffer[] = [];
      let index = levels[0].findIndex((l) => l.equals(leaf));

      for (let levelIdx = 0; levelIdx < levels.length - 1; levelIdx++) {
        const level = levels[levelIdx];
        const isRightNode = index % 2 === 1;
        const siblingIndex = isRightNode ? index - 1 : index + 1;

        if (siblingIndex < level.length) {
          proof.push(level[siblingIndex]);
        }

        index = Math.floor(index / 2);
      }

      proofs.set(leaf.toString("hex"), proof);
    }

    return { root, proofs };
  }

  it("Should initialize the subscription config", async () => {
    // Create a sample merkle root (32 bytes)
    const initialRoot = new Uint8Array(32);
    // Fill with some test data (or use actual merkle root)
    for (let i = 0; i < 32; i++) {
      initialRoot[i] = i;
    }

    // Initialize the config
    const tx = await program.methods
      .initialize(Array.from(initialRoot))
      .accounts({
        authority: wallet.publicKey,
      })
      .rpc({ skipPreflight: true, commitment: "confirmed" });

    console.log("Initialize transaction signature:", tx);

    // Fetch and verify the account data
    const configAccount = await program.account.subscriptionConfig.fetch(
      configPDA
    );

    assert.equal(
      configAccount.authority.toString(),
      wallet.publicKey.toString(),
      "Authority should match wallet public key"
    );
    assert.deepEqual(
      Buffer.from(configAccount.merkleRoot),
      initialRoot,
      "Merkle root should match initial root"
    );
    assert.isAbove(configAccount.bump, 0, "Bump should be greater than 0");

    console.log("Config initialized successfully:", {
      authority: configAccount.authority.toString(),
      merkleRoot: Buffer.from(configAccount.merkleRoot).toString("hex"),
      bump: configAccount.bump,
    });
  });

  it("Update merkle root by authority", async () => {
    // create new updated Root
    const newRoot = Buffer.alloc(32, 255);
    const tx = await program.methods
      .updateRoot(Array.from(newRoot))
      .accounts({
        config: configPDA,
        authority: wallet.publicKey,
      })
      .rpc({ commitment: "confirmed" });

    console.log("Update root transaction:", tx);

    //verify the update
    const configAccount = await program.account.subscriptionConfig.fetch(
      configPDA
    );

    assert.deepEqual(
      Buffer.from(configAccount.merkleRoot),
      newRoot,
      "Merkle root should be updated"
    );

    console.log(
      "Merkle root updated:",
      Buffer.from(configAccount.merkleRoot).toString("hex")
    );
  });

  it("Verify Subscription", async () => {
    // Initialize keypairs
    const user1 = Keypair.generate();
    const user2 = Keypair.generate();

    // Set expiration times
    const now = Math.floor(Date.now() / 1000);
    const futureExpiration = now + 86400; // 1 day from now
    const pastExpiration = now - 86400; // 1 day ago

    // Airdrop SOL to users for transaction fees
    const airdropSig1 = await connection.requestAirdrop(
      user1.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSig1);

    const airdropSig2 = await connection.requestAirdrop(
      user2.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSig2);

    //create the leaf nodes
    const leaf1 = createLeaf(user1.publicKey, futureExpiration);
    const leaf2 = createLeaf(user2.publicKey, futureExpiration);

    console.log("User1 pubkey:", user1.publicKey.toString());
    console.log("User2 pubkey:", user2.publicKey.toString());

    // Build Merkle Tree
    const { root, proofs } = buildMerkleTree([leaf1, leaf2]);

    // Update root in program
    await program.methods
      .updateRoot(Array.from(root))
      .accounts({
        config: configPDA,
        authority: wallet.publicKey,
      })
      .rpc();

    console.log("Merkle root:", root.toString("hex"));
    console.log("Leaf1:", leaf1.toString("hex"));

    // Get proof for user1
    const proof1 = proofs.get(leaf1.toString("hex"))!;
    const proof1Array = proof1.map((p) => Array.from(p));

    console.log("Proof for user1:", proof1Array);

    // Verify subscription for user1
    const tx = await program.methods
      .verifySubscription(proof1Array, new anchor.BN(futureExpiration))
      .accounts({
        user: user1.publicKey,
      })
      .signers([user1])
      .rpc({ commitment: "confirmed" });

    console.log("Verify subscription transaction:", tx);
    console.log("User1 subscription verified successfully");

    // Test with user2
    const proof2 = proofs.get(leaf2.toString("hex"))!;
    const proof2Array = proof2.map((p) => Array.from(p));

    const tx2 = await program.methods
      .verifySubscription(proof2Array, new anchor.BN(futureExpiration))
      .accounts({
        user: user2.publicKey,
      })
      .signers([user2])
      .rpc({ commitment: "confirmed" });

    console.log("User2 subscription verified successfully");

    // Test expired subscription (should fail)
    try {
      await program.methods
        .verifySubscription(proof1Array, new anchor.BN(pastExpiration))
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc({ commitment: "confirmed" });

      assert.fail("Should have failed with expired subscription");
    } catch (error) {
      console.log("Expired subscription correctly rejected");
      assert.include(error.toString(), "SubscriptionExpired");
    }
  });
});
