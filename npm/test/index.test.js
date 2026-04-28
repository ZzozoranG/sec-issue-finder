import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { ScifScanError, scan } from "../index.js";

const fakeBinary = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "fake-cli.js");

test("scan builds CLI arguments and returns parsed JSON", async () => {
  const result = await scan({
    binaryPath: fakeBinary,
    lockfile: "pnpm-lock.yaml",
    noDev: true,
    failOn: "high",
    cwd: process.cwd(),
    env: {
      SCIF_FAKE_MODE: "record",
    },
  });

  assert.deepEqual(result.args, [
    "scan",
    "--format",
    "json",
    "--lockfile",
    "pnpm-lock.yaml",
    "--no-dev",
    "--fail-on",
    "high",
  ]);
});

test("scan supports the moderate failOn alias used by the Rust CLI", async () => {
  const result = await scan({
    binaryPath: fakeBinary,
    failOn: "moderate",
    env: {
      SCIF_FAKE_MODE: "record",
    },
  });

  assert.deepEqual(result.args, ["scan", "--format", "json", "--fail-on", "moderate"]);
});

test("scan returns JSON stdout as an object", async () => {
  const result = await scan({
    binaryPath: fakeBinary,
  });

  assert.equal(result.lockfile, "fake-lockfile");
  assert.equal(result.ecosystem, "npm");
  assert.deepEqual(result.dependencies, []);
  assert.deepEqual(result.findings, []);
});

test("scan throws exitCode stdout and stderr on non-zero exit", async () => {
  await assert.rejects(
    scan({
      binaryPath: fakeBinary,
      env: {
        SCIF_FAKE_MODE: "failure",
      },
    }),
    (error) => {
      assert.ok(error instanceof ScifScanError);
      assert.equal(error.exitCode, 7);
      assert.equal(error.stdout, "partial stdout");
      assert.equal(error.stderr, "simulated stderr");
      return true;
    },
  );
});

test("scan throws a clear error when stdout is not JSON", async () => {
  await assert.rejects(
    scan({
      binaryPath: fakeBinary,
      env: {
        SCIF_FAKE_MODE: "invalid-json",
      },
    }),
    (error) => {
      assert.ok(error instanceof ScifScanError);
      assert.equal(error.exitCode, 0);
      assert.equal(error.stdout, "not json");
      assert.equal(error.stderr, "invalid json stderr");
      assert.match(error.message, /valid JSON/);
      return true;
    },
  );
});
