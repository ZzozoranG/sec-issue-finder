import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

const PLATFORM_PACKAGES = new Map([
  ["darwin:arm64", "@zzozorang/sec-issue-finder-darwin-arm64"],
  ["darwin:x64", "@zzozorang/sec-issue-finder-darwin-x64"],
  ["linux:arm64", "@zzozorang/sec-issue-finder-linux-arm64"],
  ["linux:x64", "@zzozorang/sec-issue-finder-linux-x64"],
  ["win32:x64", "@zzozorang/sec-issue-finder-win32-x64"],
]);

export function binaryFileName(platform = process.platform) {
  return platform === "win32" ? "sec-issue-finder.exe" : "sec-issue-finder";
}

export function platformPackageName(platform = process.platform, arch = process.arch) {
  return PLATFORM_PACKAGES.get(`${platform}:${arch}`) ?? null;
}

export function localBinaryCandidates({
  platform = process.platform,
  root = packageRoot,
} = {}) {
  const name = binaryFileName(platform);

  return [
    join(root, "target", "release", name),
    join(root, "target", "debug", name),
  ];
}

export function pathBinaryName(platform = process.platform) {
  return binaryFileName(platform);
}

export function optionalPackageBinaryCandidate({
  platform = process.platform,
  arch = process.arch,
  requireResolve = require.resolve,
} = {}) {
  const packageName = platformPackageName(platform, arch);

  if (!packageName) {
    return {
      packageName: null,
      path: null,
      reason: "unsupported-platform",
    };
  }

  try {
    const packageJsonPath = requireResolve(`${packageName}/package.json`);
    return {
      packageName,
      path: join(dirname(packageJsonPath), "bin", binaryFileName(platform)),
      reason: null,
    };
  } catch {
    return {
      packageName,
      path: null,
      reason: "optional-package-not-installed",
    };
  }
}

export function findBinary({
  binaryPath,
  env = process.env,
  platform = process.platform,
  arch = process.arch,
  exists = existsSync,
  requireResolve = require.resolve,
  root = packageRoot,
} = {}) {
  if (binaryPath) {
    return binaryPath;
  }

  if (env.SEC_ISSUE_FINDER_BINARY_PATH) {
    return env.SEC_ISSUE_FINDER_BINARY_PATH;
  }

  const optionalCandidate = optionalPackageBinaryCandidate({
    platform,
    arch,
    requireResolve,
  });

  if (optionalCandidate.path && exists(optionalCandidate.path)) {
    return optionalCandidate.path;
  }

  const localCandidate = localBinaryCandidates({ platform, root }).find((candidate) =>
    exists(candidate),
  );

  return localCandidate ?? pathBinaryName(platform);
}

export function missingBinaryMessage({
  platform = process.platform,
  arch = process.arch,
  requireResolve = require.resolve,
  root = packageRoot,
} = {}) {
  const optionalCandidate = optionalPackageBinaryCandidate({
    platform,
    arch,
    requireResolve,
  });
  const expectedPackage = optionalCandidate.packageName ?? "(unsupported platform)";
  const optionalPath = optionalCandidate.path ?? `(not resolved from ${expectedPackage})`;
  const searched = [
    optionalPath,
    ...localBinaryCandidates({ platform, root }),
    pathBinaryName(platform),
  ]
    .map((candidate) => `  - ${candidate}`)
    .join("\n");

  return `scif could not find the sec-issue-finder Rust binary.

Detected platform: ${platform}
Detected arch: ${arch}
Expected optional package: ${expectedPackage}

How to fix:
  - Run npm install again so optional dependencies can be installed.
  - Confirm this platform is supported by sec-issue-finder.
  - If you are developing locally, run cargo build or cargo build --release.
  - Set SEC_ISSUE_FINDER_BINARY_PATH to a sec-issue-finder binary path.

scif searched:
${searched}`;
}

