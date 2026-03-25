// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
import hre from "hardhat";

async function main() {
  const [deployer] = await hre.network.provider.request({ method: "eth_accounts" }) as string[];
  console.log("Deployer:", deployer);

  // 1. Deploy CHAOS Token (initial supply: 1 billion)
  const tokenArtifact = await hre.artifacts.readArtifact("ChaosToken");
  const tokenFactory = new hre.ethers.ContractFactory(
    tokenArtifact.abi,
    tokenArtifact.bytecode
  );
  const initialSupply = 1_000_000_000n;
  const token = await tokenFactory.deploy(initialSupply);
  await token.waitForDeployment();
  const tokenAddress = await token.getAddress();
  console.log("ChaosToken deployed:", tokenAddress);

  // 2. Deploy ChaosPredictionMarket
  const marketArtifact = await hre.artifacts.readArtifact("ChaosPredictionMarket");
  const marketFactory = new hre.ethers.ContractFactory(
    marketArtifact.abi,
    marketArtifact.bytecode
  );
  const market = await marketFactory.deploy(tokenAddress);
  await market.waitForDeployment();
  const marketAddress = await market.getAddress();
  console.log("ChaosPredictionMarket deployed:", marketAddress);

  console.log("\n=== Deployment Summary ===");
  console.log(`CHAOS Token:              ${tokenAddress}`);
  console.log(`ChaosPredictionMarket:    ${marketAddress}`);
  console.log(`\nAdd to .env.local:`);
  console.log(`NEXT_PUBLIC_CHAOS_TOKEN_TESTNET=${tokenAddress}`);
  console.log(`NEXT_PUBLIC_MARKET_CONTRACT_TESTNET=${marketAddress}`);
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
