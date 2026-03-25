// Verify contracts on BscScan
// Usage: BSCSCAN_API_KEY=xxx npx tsx scripts/verify.ts [testnet|mainnet] <tokenAddr> <marketAddr>

import { readFileSync } from "fs";
import { resolve } from "path";

const API_URLS: Record<string, string> = {
  testnet: "https://api.etherscan.io/v2/api?chainid=97",
  mainnet: "https://api.etherscan.io/v2/api?chainid=56",
};

async function verifyContract(
  apiUrl: string,
  apiKey: string,
  address: string,
  contractName: string,
  sourceCode: string,
  constructorArgs: string,
) {
  console.log(`\nVerifying ${contractName} at ${address}...`);

  const params = new URLSearchParams({
    apikey: apiKey,
    module: "contract",
    action: "verifysourcecode",
    contractaddress: address,
    sourceCode: sourceCode,
    codeformat: "solidity-single-file",
    contractname: contractName,
    compilerversion: "v0.8.24+commit.e11b9ed9",
    optimizationUsed: "0",
    runs: "200",
    constructorArguements: constructorArgs,
    evmversion: "shanghai",
    licenseType: "3", // MIT
  });

  const res = await fetch(apiUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: params.toString(),
  });

  const data = await res.json();
  console.log(`  Status: ${data.status}, Message: ${data.message}`);
  if (data.result) console.log(`  GUID: ${data.result}`);

  if (data.status === "1") {
    // Poll for verification result
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 5000));
      const sep = apiUrl.includes("?") ? "&" : "?";
      const checkRes = await fetch(
        `${apiUrl}${sep}apikey=${apiKey}&module=contract&action=checkverifystatus&guid=${data.result}`
      );
      const checkData = await checkRes.json();
      console.log(`  Check: ${checkData.result}`);
      if (checkData.result === "Pass - Verified" || checkData.result?.includes("Already Verified")) {
        return true;
      }
      if (checkData.result?.includes("Fail")) {
        console.error(`  Verification failed: ${checkData.result}`);
        return false;
      }
    }
  }
  return false;
}

// Flatten imports by reading OpenZeppelin sources
function flattenSource(contractPath: string): string {
  const src = readFileSync(contractPath, "utf-8");
  // For BscScan, we need to use Standard JSON Input instead of flattened
  return src;
}

async function main() {
  const network = process.argv[2] || "testnet";
  const tokenAddr = process.argv[3];
  const marketAddr = process.argv[4];
  const apiKey = process.env.BSCSCAN_API_KEY;

  if (!apiKey || !tokenAddr || !marketAddr) {
    console.error("Usage: BSCSCAN_API_KEY=xxx npx tsx scripts/verify.ts [testnet|mainnet] <tokenAddr> <marketAddr>");
    process.exit(1);
  }

  const apiUrl = API_URLS[network];
  if (!apiUrl) {
    console.error("Invalid network. Use 'testnet' or 'mainnet'");
    process.exit(1);
  }

  // Use Standard JSON Input format for verification (handles imports properly)
  const artifactsDir = resolve(__dirname, "../artifacts");

  const tokenBuildInfo = findBuildInfo(artifactsDir, "ChaosToken");
  const marketBuildInfo = findBuildInfo(artifactsDir, "ChaosPredictionMarket");

  if (!tokenBuildInfo || !marketBuildInfo) {
    console.error("Build info not found. Run 'npx hardhat compile' first.");
    process.exit(1);
  }

  // Constructor args encoding
  const { ethers } = await import("ethers");
  const tokenConstructorArgs = ethers.AbiCoder.defaultAbiCoder()
    .encode(["uint256"], [ethers.parseUnits("1000000000", 18)])
    .slice(2); // remove 0x prefix

  const marketConstructorArgs = ethers.AbiCoder.defaultAbiCoder()
    .encode(["address"], [tokenAddr])
    .slice(2);

  // Submit verification using Standard JSON Input
  await verifyWithJsonInput(apiUrl, apiKey, tokenAddr, "contracts/ChaosToken.sol:ChaosToken", tokenBuildInfo, tokenConstructorArgs);
  await verifyWithJsonInput(apiUrl, apiKey, marketAddr, "contracts/ChaosPredictionMarket.sol:ChaosPredictionMarket", marketBuildInfo, marketConstructorArgs);
}

function findBuildInfo(artifactsDir: string, contractName: string): any {
  const buildInfoDir = resolve(artifactsDir, "build-info");
  try {
    const { readdirSync } = require("fs");
    const files = readdirSync(buildInfoDir);
    for (const f of files) {
      if (f.endsWith(".json")) {
        const info = JSON.parse(readFileSync(resolve(buildInfoDir, f), "utf-8"));
        if (info.output?.contracts) {
          for (const source of Object.keys(info.output.contracts)) {
            if (info.output.contracts[source][contractName]) {
              return { input: info.input, solcVersion: info.solcVersion };
            }
          }
        }
      }
    }
  } catch {}
  return null;
}

async function verifyWithJsonInput(
  apiUrl: string,
  apiKey: string,
  address: string,
  contractPath: string,
  buildInfo: any,
  constructorArgs: string,
) {
  console.log(`\nVerifying ${contractPath} at ${address}...`);

  const params = new URLSearchParams({
    apikey: apiKey,
    module: "contract",
    action: "verifysourcecode",
    contractaddress: address,
    sourceCode: JSON.stringify(buildInfo.input),
    codeformat: "solidity-standard-json-input",
    contractname: contractPath,
    compilerversion: `v${buildInfo.solcVersion}`,
    constructorArguements: constructorArgs,
  });

  const res = await fetch(apiUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: params.toString(),
  });

  const data = await res.json();
  console.log(`  Submit: status=${data.status}, message=${data.message}`);
  if (data.result && data.status === "1") {
    console.log(`  GUID: ${data.result}`);
    for (let i = 0; i < 12; i++) {
      await new Promise((r) => setTimeout(r, 5000));
      const sep = apiUrl.includes("?") ? "&" : "?";
      const checkRes = await fetch(
        `${apiUrl}${sep}apikey=${apiKey}&module=contract&action=checkverifystatus&guid=${data.result}`
      );
      const checkData = await checkRes.json();
      console.log(`  Status: ${checkData.result}`);
      if (checkData.result?.includes("Pass") || checkData.result?.includes("Already")) {
        console.log("  VERIFIED!");
        return;
      }
      if (checkData.result?.includes("Fail")) {
        console.error(`  Failed: ${checkData.result}`);
        return;
      }
    }
  } else {
    console.error(`  Error: ${data.result}`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
