import { lstat, readdir } from "node:fs/promises";
import { basename, extname, resolve } from "node:path";

const forbiddenExtensions = new Set([
  ".csv",
  ".db",
  ".log",
  ".sqlite",
  ".sqlite3",
]);

function isForbidden(path) {
  const name = basename(path).toLowerCase();
  return (
    name === ".env" ||
    name.startsWith(".env.") ||
    forbiddenExtensions.has(extname(name))
  );
}

async function collect(path, findings) {
  const metadata = await lstat(path);
  if (metadata.isSymbolicLink()) {
    throw new Error(`release content must not contain symbolic links: ${path}`);
  }
  if (metadata.isDirectory()) {
    for (const entry of await readdir(path)) {
      await collect(resolve(path, entry), findings);
    }
    return;
  }
  if (isForbidden(path)) findings.push(path);
}

const roots = process.argv.slice(2).map((path) => resolve(path));
if (roots.length === 0) {
  console.error(
    "usage: node scripts/audit-release-content.mjs <file-or-directory> [...]",
  );
  process.exit(2);
}

const findings = [];
for (const root of roots) await collect(root, findings);
if (findings.length > 0) {
  console.error(
    "release content contains forbidden local-data or development files:",
  );
  for (const finding of findings) console.error(`- ${finding}`);
  process.exit(1);
}

console.log(`Release content audit passed for ${roots.length} path(s).`);
