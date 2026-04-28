import { spawn } from "node:child_process";

import { findBinary, missingBinaryMessage } from "./lib/binary.js";
import { ScifScanError } from "./lib/errors.js";

const VALID_FAIL_ON = new Set(["low", "moderate", "medium", "high", "critical"]);

export { ScifScanError };

export async function scan(options = {}) {
  const args = buildScanArgs(options);
  const binary = findBinary({ binaryPath: options.binaryPath });
  const { exitCode, stdout, stderr } = await runCli(binary, args, options);

  if (exitCode !== 0) {
    throw new ScifScanError(`sec-issue-finder exited with code ${exitCode}`, {
      exitCode,
      stdout,
      stderr,
    });
  }

  try {
    return JSON.parse(stdout);
  } catch (cause) {
    throw new ScifScanError("sec-issue-finder did not return valid JSON on stdout", {
      exitCode,
      stdout,
      stderr,
      cause,
    });
  }
}

function buildScanArgs(options) {
  const args = ["scan", "--format", "json"];

  if (options.lockfile) {
    args.push("--lockfile", options.lockfile);
  }

  if (options.noDev) {
    args.push("--no-dev");
  }

  if (options.failOn) {
    if (!VALID_FAIL_ON.has(options.failOn)) {
      throw new TypeError(`Invalid failOn value: ${options.failOn}`);
    }
    args.push("--fail-on", options.failOn);
  }

  return args;
}

function runCli(binary, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(binary, args, {
      cwd: options.cwd,
      env: {
        ...process.env,
        ...(options.env ?? {}),
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });

    child.on("error", (error) => {
      if (error.code === "ENOENT" && !options.binaryPath) {
        reject(new ScifScanError(missingBinaryMessage(), { stdout, stderr, cause: error }));
        return;
      }

      reject(
        new ScifScanError(`Failed to start sec-issue-finder: ${error.message}`, {
          stdout,
          stderr,
          cause: error,
        }),
      );
    });

    child.on("close", (exitCode) => {
      resolve({ exitCode, stdout, stderr });
    });
  });
}

export default {
  scan,
};
