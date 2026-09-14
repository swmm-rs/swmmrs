import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// Scan bytes too: WASM can retain export names, panic strings, and source paths.
const root = fileURLToPath(new URL("../dist/", import.meta.url));
// Derive the private module name from source rather than hard-coding it here.
const source = fileURLToPath(new URL("../../crates/solver/thirdparty/swmmrs-parallel/rust/", import.meta.url));
const backends = readdirSync(source, { withFileTypes: true })
  .filter(entry => entry.isDirectory() && existsSync(join(source, entry.name, "browser.rs")));
assert.equal(backends.length, 1, "Expected exactly one parallel browser backend");
const internalName = backends[0].name.toLowerCase();
function check(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    assert(!entry.name.toLowerCase().includes(internalName), `Unexpected release asset name: ${path}`);
    if (entry.isDirectory()) check(path);
    else assert(!readFileSync(path).toString("latin1").toLowerCase().includes(internalName), `Internal backend name in ${path}`);
  }
}
check(root);
console.log("Release asset names and contents verified, including WASM.");
