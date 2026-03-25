// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
//
// Deploy ChaosToken + ChaosPredictionMarket to BSC
// Usage: DEPLOYER_PRIVATE_KEY=0x... npx tsx scripts/deploy.ts [testnet|mainnet]

import { ethers } from "ethers";
import { readFileSync } from "fs";
import { resolve } from "path";

const NETWORKS = {
  testnet: {
    url: "https://data-seed-prebsc-1-s1.binance.org:8545",
    chainId: 97,
    explorer: "https://testnet.bscscan.com",
    envPrefix: "TESTNET",
  },
  mainnet: {
    url: "https://bsc-dataseed.binance.org/",
    chainId: 56,
    explorer: "https://bscscan.com",
    envPrefix: "MAINNET",
  },
};

async function main() {
  const network = (process.argv[2] || "testnet") as keyof typeof NETWORKS;
  if (!NETWORKS[network]) {
    console.error("Usage: npx tsx scripts/deploy.ts [testnet|mainnet]");
    process.exit(1);
  }

  const pk = process.env.DEPLOYER_PRIVATE_KEY;
  if (!pk) {
    console.error("Set DEPLOYER_PRIVATE_KEY env var");
    process.exit(1);
  }

  const net = NETWORKS[network];
  const provider = new ethers.JsonRpcProvider(net.url, net.chainId);
  const wallet = new ethers.Wallet(pk, provider);

  console.log(`Network:  ${network} (chainId: ${net.chainId})`);
  console.log(`Deployer: ${wallet.address}`);

  const balance = await provider.getBalance(wallet.address);
  console.log(`Balance:  ${ethers.formatEther(balance)} BNB`);

  if (balance < ethers.parseEther("0.01")) {
    console.error("Need at least 0.01 BNB for gas");
    process.exit(1);
  }

  // Read artifacts
  const artifactsDir = resolve(__dirname, "../artifacts/contracts");

  const tokenArtifact = JSON.parse(
    readFileSync(resolve(artifactsDir, "ChaosToken.sol/ChaosToken.json"), "utf-8")
  );
  const marketArtifact = JSON.parse(
    readFileSync(resolve(artifactsDir, "ChaosPredictionMarket.sol/ChaosPredictionMarket.json"), "utf-8")
  );

  // 1. Deploy ChaosToken (1 billion tokens)
  const initialSupply = ethers.parseUnits("1000000000", 18); // 1B * 1e18
  console.log(`\nDeploying ChaosToken (1B CHAOS)...`);

  const tokenFactory = new ethers.ContractFactory(tokenArtifact.abi, tokenArtifact.bytecode, wallet);
  const tokenContract = await tokenFactory.deploy(initialSupply);
  console.log(`  tx: ${tokenContract.deploymentTransaction()?.hash}`);
  await tokenContract.waitForDeployment();
  const tokenAddress = await tokenContract.getAddress();
  console.log(`  ChaosToken deployed: ${tokenAddress}`);

  // 2. Deploy ChaosPredictionMarket
  console.log(`\nDeploying ChaosPredictionMarket...`);

  const marketFactory = new ethers.ContractFactory(marketArtifact.abi, marketArtifact.bytecode, wallet);
  const marketContract = await marketFactory.deploy(tokenAddress);
  console.log(`  tx: ${marketContract.deploymentTransaction()?.hash}`);
  await marketContract.waitForDeployment();
  const marketAddress = await marketContract.getAddress();
  console.log(`  ChaosPredictionMarket deployed: ${marketAddress}`);

  // Summary
  console.log("\n══════════════════════════════════════════════");
  console.log("  DEPLOYMENT SUCCESSFUL");
  console.log("══════════════════════════════════════════════");
  console.log(`  Network:              ${network}`);
  console.log(`  Deployer:             ${wallet.address}`);
  console.log(`  ChaosToken:           ${tokenAddress}`);
  console.log(`  ChaosPredictionMarket:${marketAddress}`);
  console.log(`  Token Explorer:       ${net.explorer}/address/${tokenAddress}`);
  console.log(`  Market Explorer:      ${net.explorer}/address/${marketAddress}`);
  console.log("══════════════════════════════════════════════");
  console.log(`\n.env.local:`);
  console.log(`NEXT_PUBLIC_CHAOS_TOKEN_${net.envPrefix}=${tokenAddress}`);
  console.log(`NEXT_PUBLIC_MARKET_CONTRACT_${net.envPrefix}=${marketAddress}`);
}

main().catch((err) => {
  console.error("Deployment failed:", err.message || err);
  process.exitCode = 1;
});
