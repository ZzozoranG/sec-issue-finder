#!/usr/bin/env node

import { chmod, copyFile, mkdir, readFile, stat } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const defaultRoot = resolve(scriptDir, "..");

const PLATFORM_PACKAGES = new Map([
  [
    "darwin:arm64",
    {
      directory: "darwin-arm64",
      name: "@sec-issue-finder/darwin-arm64",
      binaryName: "sec-issue-finder",
    },
  ],
  [
    "darwin:x64",
    {
      directory: "darwin-x64",
      name: "@sec-issue-finder/darwin-x64",
      binaryName: "sec-issue-finder",
    },
  ],
  [
    "linux:x64",
    {
      directory: "linux-x64",
      name: "@sec-issue-finder/linux-x64",
      binaryName: "sec-issue-finder",
    },
  ],
  [
    "linux:arm64",
    {
      directory: "linux-arm64",
      name: "@sec-issue-finder/linux-arm64",
      binaryName: "sec-issue-finder",
    },
  ],
  [
    "win32:x64",
    {
      directory: "win32-x64",
      name: "@sec-issue-finder/win32-x64",
      binaryName: "sec-issue-finder.exe",
    },
  ],
]);

export function platformPackageInfo(platform, arch) {
  return PLATFORM_PACKAGES.get(`${platform}:${arch}`) ?? null;
}

export function parseArgs(argv) {
  const options = {
    dryRun: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === "--dry-run") {
      options.dryRun = true;
      continue;
    }

    if (!arg.startsWith("--")) {
      throw new Error(`Unexpected argument: ${arg}`);
    }

    const key = arg.slice(2);
    const value = argv[index + 1];

    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for ${arg}`);
    }

    options[key] = value;
    index += 1;
  }

  for (const required of ["platform", "arch", "binary"]) {
    if (!options[required]) {
      throw new Error(`Missing required argument: --${required}`);
    }
  }

  return options;
}

export async function preparePlatformPackage({
  platform,
  arch,
  binary,
  dryRun = false,
  root = defaultRoot,
} = {}) {
  const info = platformPackageInfo(platform, arch);

  if (!info) {
    throw new Error(`Unsupported platform/arch: ${platform}/${arch}`);
  }

  const mainPackageJsonPath = join(root, "package.json");
  const packageRoot = join(root, "packages", info.directory);
  const platformPackageJsonPath = join(packageRoot, "package.json");
  const binDir = join(packageRoot, "bin");
  const targetBinary = join(binDir, info.binaryName);
  const sourceBinary = resolve(root, binary);

  const [mainPackageJson, platformPackageJson] = await Promise.all([
    readJson(mainPackageJsonPath),
    readJson(platformPackageJsonPath),
  ]);

  if (platformPackageJson.name !== info.name) {
    throw new Error(
      `Unexpected package name in ${platformPackageJsonPath}: expected ${info.name}, got ${platformPackageJson.name}`,
    );
  }

  if (platformPackageJson.version !== mainPackageJson.version) {
    throw new Error(
      `Version mismatch for ${info.name}: main package is ${mainPackageJson.version}, platform package is ${platformPackageJson.version}`,
    );
  }

  if (!dryRun) {
    const sourceStats = await stat(sourceBinary);
    if (!sourceStats.isFile()) {
      throw new Error(`Binary path is not a file: ${sourceBinary}`);
    }

    await mkdir(binDir, { recursive: true });
    await copyFile(sourceBinary, targetBinary);

    if (platform !== "win32") {
      await chmod(targetBinary, 0o755);
    }
  }

  return {
    packageName: info.name,
    packageDirectory: packageRoot,
    sourceBinary,
    targetBinary,
    dryRun,
  };
}

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

async function main() {
  try {
    const options = parseArgs(process.argv.slice(2));
    const result = await preparePlatformPackage(options);

    console.log(
      `${result.dryRun ? "Would prepare" : "Prepared"} ${result.packageName}: ${result.sourceBinary} -> ${result.targetBinary}`,
    );
  } catch (error) {
    console.error(`prepare-platform-package failed: ${error.message}`);
    process.exitCode = 1;
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}

