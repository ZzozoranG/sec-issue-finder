export class ScifScanError extends Error {
  constructor(message, { exitCode = null, stdout = "", stderr = "", cause } = {}) {
    super(message, cause ? { cause } : undefined);
    this.name = "ScifScanError";
    this.exitCode = exitCode;
    this.stdout = stdout;
    this.stderr = stderr;
  }
}
