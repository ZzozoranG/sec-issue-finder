import assert from "node:assert/strict";
import { constants } from "node:fs";
import { access, mkdir, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import test from "node:test";

import {
  parseArgs,
  platformPackageInfo,
  preparePlatformPackage,
} from "../../scripts/prepare-platform-package.mjs";

test("maps supported platform packages", () => {
  assert.deepEqual(platformPackageInfo("linux", "x64"), {
    directory: "linux-x64",
    name: "@sec-issue-finder/linux-x64",
    binaryName: "sec-issue-finder",
  });
  assert.deepEqual(platformPackageInfo("win32", "x64"), {
    directory: "win32-x64",
    name: "@sec-issue-finder/win32-x64",
    binaryName: "sec-issue-finder.exe",
  });
});

test("rejects unsupported platform packages", () => {
  assert.equal(platformPackageInfo("freebsd", "x64"), null);
});

test("parses CLI args including dry-run", () => {
  assert.deepEqual(
    parseArgs([
      "--platform",
      "linux",
      "--arch",
      "x64",
      "--binary",
      "target/release/sec-issue-finder",
      "--dry-run",
    ]),
    {
      platform: "linux",
      arch: "x64",
      binary: "target/release/sec-issue-finder",
      dryRun: true,
    },
  );
});

test("copies a Unix binary and makes it executable", async () => {
  const root = await createFixtureRoot("linux-x64", "@sec-issue-finder/linux-x64");
  const source = join(root, "target", "release", "sec-issue-finder");
  await mkdir(join(root, "target", "release"), { recursive: true });
  await writeFile(source, "#!/bin/sh\nexit 0\n", { mode: 0o644 });

  const result = await preparePlatformPackage({
    platform: "linux",
    arch: "x64",
    binary: "target/release/sec-issue-finder",
    root,
  });

  assert.equal(result.packageName, "@sec-issue-finder/linux-x64");
  assert.equal(result.targetBinary, join(root, "packages", "linux-x64", "bin", "sec-issue-finder"));
  assert.equal(await readFile(result.targetBinary, "utf8"), "#!/bin/sh\nexit 0\n");
  await access(result.targetBinary, constants.X_OK);
});

test("copies a Windows binary with exe extension", async () => {
  const root = await createFixtureRoot("win32-x64", "@sec-issue-finder/win32-x64");
  const source = join(root, "target", "release", "sec-issue-finder.exe");
  await mkdir(join(root, "target", "release"), { recursive: true });
  await writeFile(source, "fake exe");

  const result = await preparePlatformPackage({
    platform: "win32",
    arch: "x64",
    binary: "target/release/sec-issue-finder.exe",
    root,
  });

  assert.equal(result.targetBinary, join(root, "packages", "win32-x64", "bin", "sec-issue-finder.exe"));
  assert.equal(await readFile(result.targetBinary, "utf8"), "fake exe");
});

test("dry-run validates metadata without copying binary", async () => {
  const root = await createFixtureRoot("darwin-arm64", "@sec-issue-finder/darwin-arm64");

  const result = await preparePlatformPackage({
    platform: "darwin",
    arch: "arm64",
    binary: "target/release/sec-issue-finder",
    dryRun: true,
    root,
  });

  assert.equal(result.dryRun, true);
  await assert.rejects(stat(result.targetBinary), /ENOENT/);
});

test("fails when platform package version differs from main package", async () => {
  const root = await createFixtureRoot("linux-arm64", "@sec-issue-finder/linux-arm64", {
    platformVersion: "0.2.0",
  });

  await assert.rejects(
    preparePlatformPackage({
      platform: "linux",
      arch: "arm64",
      binary: "target/release/sec-issue-finder",
      dryRun: true,
      root,
    }),
    /Version mismatch/,
  );
});

async function createFixtureRoot(directory, packageName, { platformVersion = "0.1.0" } = {}) {
  const root = await mkdtemp(join(tmpdir(), "scif-platform-package-"));
  await writeJson(join(root, "package.json"), {
    name: "sec-issue-finder",
    version: "0.1.0",
  });
  await mkdir(join(root, "packages", directory), { recursive: true });
  await writeJson(join(root, "packages", directory, "package.json"), {
    name: packageName,
    version: platformVersion,
    files: ["bin/"],
  });

  return root;
}

async function writeJson(path, value) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

