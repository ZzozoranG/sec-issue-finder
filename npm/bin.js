#!/usr/bin/env node

import { spawnSync } from "node:child_process";

import { findBinary, missingBinaryMessage } from "./lib/binary.js";

const binary = findBinary();
const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  if (result.error.code === "ENOENT") {
    console.error(missingBinaryMessage());
  } else {
    console.error(`scif failed to start sec-issue-finder: ${result.error.message}`);
  }
  process.exit(1);
}

if (result.signal) {
  console.error(`scif terminated because sec-issue-finder received signal ${result.signal}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
