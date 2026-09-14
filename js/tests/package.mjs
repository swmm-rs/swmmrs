import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";

const manifest = JSON.parse(execFileSync("npm", ["pack", "--dry-run", "--json"], { encoding: "utf8" }));
const [packageInfo] = Object.values(manifest);
const paths = new Set(packageInfo.files.map(file => file.path));
for (const path of ["index.js", "index.d.ts", "worker.js", "worker-node.js", "lib/swmmrs/index.js", "lib/swmmrs/index.d.ts", "lib/swmmrs/worker.js", "dist/swmmrs.js", "dist/swmmrs_bg.wasm", "dist/serial/swmmrs.js", "dist/serial/swmmrs.d.ts", "dist/serial/swmmrs_bg.wasm"]) {
  assert(paths.has(path), `Package is missing ${path}. Run npm run build first.`);
}
assert([...paths].some(path => path.startsWith("dist/snippets/") && path.endsWith("browser.js")), "Package is missing the thread-pool worker helper");
assert(![...paths].some(path => path.startsWith("src/native/")), "Package must not include native adapter sources");
console.log(`Package verified: ${paths.size} files, including WASM and worker assets.`);
