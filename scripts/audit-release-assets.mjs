import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { basename, resolve } from "node:path";

const [assetDirectoryArgument, version] = process.argv.slice(2);
if (
  !assetDirectoryArgument ||
  !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u.test(version ?? "")
) {
  console.error(
    "usage: node scripts/audit-release-assets.mjs <directory> <version>",
  );
  process.exit(2);
}

const assetDirectory = resolve(assetDirectoryArgument);
const payloadNames = [
  `QRY-${version}-aarch64.app.zip`,
  `QRY-${version}-x86_64.app.zip`,
  `QRY_${version}_x64-setup.exe`,
  `QRY_${version}_x64_en-US.msi`,
];
const expectedNames = payloadNames
  .flatMap((name) => [name, `${name}.sha256`])
  .sort();
const actualNames = (await readdir(assetDirectory)).sort();
if (JSON.stringify(actualNames) !== JSON.stringify(expectedNames)) {
  throw new Error(
    `unexpected release assets\nexpected: ${expectedNames.join(", ")}\nactual: ${actualNames.join(", ")}`,
  );
}

for (const payloadName of payloadNames) {
  const payloadPath = resolve(assetDirectory, payloadName);
  const checksumPath = `${payloadPath}.sha256`;
  const checksum = (await readFile(checksumPath, "utf8")).trimEnd();
  const match = checksum.match(/^([a-f0-9]{64})  (.+)$/u);
  if (!match || match[2] !== basename(payloadPath)) {
    throw new Error(`invalid checksum file format: ${checksumPath}`);
  }
  const actualHash = createHash("sha256")
    .update(await readFile(payloadPath))
    .digest("hex");
  if (actualHash !== match[1]) {
    throw new Error(`checksum mismatch: ${payloadName}`);
  }
}

console.log(`Release asset audit passed for QRY ${version}.`);
