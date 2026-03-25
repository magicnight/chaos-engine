import hre from "hardhat";

async function main() {
  const [deployer] = await hre.network.provider.request({ method: "eth_accounts" }) as string[];
  console.log("Deployer:", deployer);

  // 1. Deploy CRUX Token (initial supply: 1 billion)
  const tokenArtifact = await hre.artifacts.readArtifact("CrucixToken");
  const tokenFactory = new hre.ethers.ContractFactory(
    tokenArtifact.abi,
    tokenArtifact.bytecode
  );
  const initialSupply = 1_000_000_000n;
  const token = await tokenFactory.deploy(initialSupply);
  await token.waitForDeployment();
  const tokenAddress = await token.getAddress();
  console.log("CrucixToken deployed:", tokenAddress);

  // 2. Deploy PredictionMarket
  const marketArtifact = await hre.artifacts.readArtifact("PredictionMarket");
  const marketFactory = new hre.ethers.ContractFactory(
    marketArtifact.abi,
    marketArtifact.bytecode
  );
  const market = await marketFactory.deploy(tokenAddress);
  await market.waitForDeployment();
  const marketAddress = await market.getAddress();
  console.log("PredictionMarket deployed:", marketAddress);

  console.log("\n=== Deployment Summary ===");
  console.log(`CRUX Token:        ${tokenAddress}`);
  console.log(`PredictionMarket:  ${marketAddress}`);
  console.log(`\nAdd to .env.local:`);
  console.log(`NEXT_PUBLIC_CRUX_TOKEN_TESTNET=${tokenAddress}`);
  console.log(`NEXT_PUBLIC_MARKET_CONTRACT_TESTNET=${marketAddress}`);
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
