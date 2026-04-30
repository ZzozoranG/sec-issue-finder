import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { scan } from "../index.js";
import {
  binaryFileName,
  findBinary,
  missingBinaryMessage,
  optionalPackageBinaryCandidate,
  platformPackageName,
} from "../lib/resolve-binary.js";

const testDir = dirname(fileURLToPath(import.meta.url));

test("maps darwin arm64 to the darwin arm64 package", () => {
  assert.equal(platformPackageName("darwin", "arm64"), "@sec-issue-finder/darwin-arm64");
});

test("maps darwin x64 to the darwin x64 package", () => {
  assert.equal(platformPackageName("darwin", "x64"), "@sec-issue-finder/darwin-x64");
});

test("maps linux x64 to the linux x64 package", () => {
  assert.equal(platformPackageName("linux", "x64"), "@sec-issue-finder/linux-x64");
});

test("maps linux arm64 to the linux arm64 package", () => {
  assert.equal(platformPackageName("linux", "arm64"), "@sec-issue-finder/linux-arm64");
});

test("maps win32 x64 to the win32 x64 package and exe binary", () => {
  assert.equal(platformPackageName("win32", "x64"), "@sec-issue-finder/win32-x64");
  assert.equal(binaryFileName("win32"), "sec-issue-finder.exe");
});

test("unsupported platform and arch have no optional package", () => {
  assert.equal(platformPackageName("freebsd", "x64"), null);

  assert.deepEqual(
    optionalPackageBinaryCandidate({
      platform: "freebsd",
      arch: "x64",
      requireResolve: () => {
        throw new Error("should not resolve unsupported packages");
      },
    }),
    {
      packageName: null,
      path: null,
      reason: "unsupported-platform",
    },
  );
});

test("resolves optional package binary when package is installed", () => {
  const packageJsonPath = join("/virtual", "node_modules", "@sec-issue-finder", "linux-x64", "package.json");
  const binaryPath = join(
    "/virtual",
    "node_modules",
    "@sec-issue-finder",
    "linux-x64",
    "bin",
    "sec-issue-finder",
  );

  assert.equal(
    findBinary({
      platform: "linux",
      arch: "x64",
      env: {},
      exists: (candidate) => candidate === binaryPath,
      requireResolve: () => packageJsonPath,
      root: "/repo",
    }),
    binaryPath,
  );
});

test("SEC_ISSUE_FINDER_BINARY_PATH overrides optional package and local fallback", () => {
  assert.equal(
    findBinary({
      platform: "linux",
      arch: "x64",
      env: {
        SEC_ISSUE_FINDER_BINARY_PATH: "/custom/sec-issue-finder",
      },
      exists: () => true,
      requireResolve: () => "/virtual/package.json",
      root: "/repo",
    }),
    "/custom/sec-issue-finder",
  );
});

test("scan binaryPath option overrides SEC_ISSUE_FINDER_BINARY_PATH", async () => {
  const result = await scan({
    binaryPath: join(testDir, "fixtures", "fake-cli.js"),
    env: {
      SEC_ISSUE_FINDER_BINARY_PATH: "/does/not/exist",
      SCIF_FAKE_MODE: "record",
    },
  });

  assert.deepEqual(result.args, ["scan", "--format", "json"]);
});

test("missing binary message includes platform arch package and fixes", () => {
  const message = missingBinaryMessage({
    platform: "linux",
    arch: "x64",
    requireResolve: () => {
      throw new Error("not installed");
    },
    root: "/repo",
  });

  assert.match(message, /Detected platform: linux/);
  assert.match(message, /Detected arch: x64/);
  assert.match(message, /Expected optional package: @sec-issue-finder\/linux-x64/);
  assert.match(message, /Run npm install again/);
  assert.match(message, /cargo build/);
  assert.match(message, /SEC_ISSUE_FINDER_BINARY_PATH/);
});
