// Type-check Markdown and source TSDoc examples as standalone consumer modules.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const docsRoot = fileURLToPath(new URL("../../docs/javascript/", import.meta.url));
const sourceRoot = join(packageRoot, "src/swmmrs");

function filesWithExtension(directory, extension) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesWithExtension(path, extension) : entry.name.endsWith(extension) ? [path] : [];
  }).sort();
}

// Keep temporary modules in the package so its self-reference resolves normally.
const temporary = mkdtempSync(join(packageRoot, ".docs-types-"));
try {
  const sources = [];
  const pages = [
    ...filesWithExtension(docsRoot, ".md"),
    ...filesWithExtension(sourceRoot, ".ts"),
  ];
  for (const page of pages) {
    const contents = readFileSync(page, "utf8");
    const blocks = page.endsWith(".md")
      ? [{ text: contents, offset: 0 }]
      : [...contents.matchAll(/\/\*\*([\s\S]*?)\*\//g)].map(match => ({
        text: match[1].replace(/^[ \t]*\* ?/gm, ""),
        offset: contents.slice(0, match.index).split("\n").length - 1,
      }));
    for (const { text, offset } of blocks) {
      for (const match of text.matchAll(/^```(?:typescript|ts)[^\n]*\n([\s\S]*?)^```[ \t]*\r?$/gm)) {
        const line = offset + text.slice(0, match.index).split("\n").length;
        const name = relative(packageRoot, page).replaceAll(/[\\/]/g, "-").replace(/\.(?:md|ts)$/, "");
        const source = join(temporary, `${name}-line-${line}.ts`);
        // The getting-started page serves index.js directly; use the same public
        // declarations through the package self-reference when compiling examples.
        const code = match[1].replaceAll('"./index.js"', '"@swmmrs/swmmrs"');
        writeFileSync(source, code);
        sources.push(source);
      }
    }
  }
  if (sources.length === 0) throw new Error("No TypeScript documentation examples found");
  const compiler = join(packageRoot, "node_modules/typescript/bin/tsc");
  try {
    execFileSync(process.execPath, [
      compiler, "--ignoreConfig", "--noEmit", "--strict", "--target", "ES2022",
      "--module", "NodeNext", "--moduleResolution", "NodeNext",
      "--lib", "ES2022,DOM,ESNext.Disposable", "--skipLibCheck",
      "--noUncheckedIndexedAccess", "--exactOptionalPropertyTypes", ...sources,
    ], { cwd: packageRoot, stdio: "inherit" });
  } catch (error) {
    process.exitCode = error.status || 1;
  }
  if (!process.exitCode) console.log(`Checked ${sources.length} TypeScript documentation examples.`);
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
