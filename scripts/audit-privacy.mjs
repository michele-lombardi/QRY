import { readFile, readdir } from "node:fs/promises";
import { extname, resolve } from "node:path";

const projectRoot = resolve(import.meta.dirname, "..");
const appRoot = resolve(projectRoot, "QRY");
const sourceRoots = [
  resolve(appRoot, "crates"),
  resolve(appRoot, "src-tauri", "src"),
  resolve(appRoot, "src"),
];
const sourceExtensions = new Set([".js", ".rs", ".ts"]);
const loggingPattern =
  /(?:\b(?:print|println|eprint|eprintln|dbg)!\s*\(|\b(?:tracing|log)::|\bconsole\.)/;
const allowedReference = "typing callback hot-path reference";

async function sourceFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await sourceFiles(path)));
    else if (entry.isFile() && sourceExtensions.has(extname(entry.name)))
      files.push(path);
  }
  return files;
}

const unexpectedLogs = [];
for (const root of sourceRoots) {
  for (const path of await sourceFiles(root)) {
    const lines = (await readFile(path, "utf8")).split(/\r?\n/u);
    lines.forEach((line, index) => {
      if (loggingPattern.test(line) && !line.includes(allowedReference)) {
        unexpectedLogs.push(`${path}:${index + 1}:${line.trim()}`);
      }
    });
  }
}
if (unexpectedLogs.length > 0) {
  console.error("Unexpected runtime logging found:");
  unexpectedLogs.forEach((line) => console.error(line));
  process.exit(1);
}

const capabilityPath = resolve(
  appRoot,
  "src-tauri",
  "capabilities",
  "default.json",
);
const capability = JSON.parse(await readFile(capabilityPath, "utf8"));
if (
  JSON.stringify(capability.permissions) !== JSON.stringify(["core:default"])
) {
  throw new Error(`unexpected Tauri permissions: ${capability.permissions}`);
}

console.log(
  "Static privacy audit passed: no runtime logs and a narrow Tauri capability.",
);
