import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function binaryNames(platform = process.platform) {
  return platform === "win32"
    ? ["sec-issue-finder.exe", "sec-issue-finder"]
    : ["sec-issue-finder"];
}

export function localBinaryCandidates(platform = process.platform) {
  return [
    ...binaryNames(platform).map((name) => join(packageRoot, "target", "release", name)),
    ...binaryNames(platform).map((name) => join(packageRoot, "target", "debug", name)),
  ];
}

export function pathBinaryName(platform = process.platform) {
  return platform === "win32" ? "sec-issue-finder.exe" : "sec-issue-finder";
}

export function findBinary({ binaryPath, platform = process.platform } = {}) {
  if (binaryPath) {
    return binaryPath;
  }

  return localBinaryCandidates(platform).find((candidate) => existsSync(candidate)) ?? pathBinaryName(platform);
}

export function missingBinaryMessage(platform = process.platform) {
  const searched = [...localBinaryCandidates(platform), pathBinaryName(platform)]
    .map((candidate) => `  - ${candidate}`)
    .join("\n");

  return `scif could not find the sec-issue-finder Rust binary.

Build it locally first:

  cargo build

For a release build:

  cargo build --release

scif searched:
${searched}`;
}
