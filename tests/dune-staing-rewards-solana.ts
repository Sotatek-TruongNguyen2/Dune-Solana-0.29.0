import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DuneStaingRewardsSolana } from "../target/types/dune_staing_rewards_solana";

describe("dune-staking-rewards-solana", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DuneStaingRewardsSolana as Program<DuneStaingRewardsSolana>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
