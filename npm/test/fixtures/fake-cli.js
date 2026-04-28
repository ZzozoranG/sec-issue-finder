#!/usr/bin/env node

const args = process.argv.slice(2);
const mode = process.env.SCIF_FAKE_MODE ?? "success";

if (mode === "record") {
  console.log(
    JSON.stringify({
      args,
      cwd: process.cwd(),
      lockfile: "fake-lockfile",
      ecosystem: "npm",
      dependencies: [],
      findings: [],
    }),
  );
  process.exit(0);
}

if (mode === "failure") {
  process.stdout.write("partial stdout");
  process.stderr.write("simulated stderr");
  process.exit(7);
}

if (mode === "invalid-json") {
  process.stdout.write("not json");
  process.stderr.write("invalid json stderr");
  process.exit(0);
}

console.log(
  JSON.stringify({
    lockfile: "fake-lockfile",
    ecosystem: "npm",
    dependencies: [],
    findings: [],
  }),
);
