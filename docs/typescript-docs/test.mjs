import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { ReflectionKind } from "typedoc";
import ts from "typescript";

const read = path => readFileSync(new URL(path, import.meta.url), "utf8");
const config = JSON.parse(read("typedoc.json"));
const families = config.entryPoints.map(path => path.replace(/\.ts$/, ""));
const project = JSON.parse(read(".generated/reflection.json"));
const modules = new Map(project.children.map(child => [child.name, child]));
const pages = new Map(families.map(name => [name, read(`.generated/${name}.md`)]));
const declaration = (family, name) => modules.get(family).children.find(child => child.name === name);
const node = declaration("node", "Node");
const markdown = pages.get("node");
const summary = reflection => reflection.comment?.summary?.map(part => part.text ?? "").join("");
const anchors = markdown => [...markdown.matchAll(/\{#([^}]+)\}|<a id="([^"]+)"><\/a>/g)]
  .map(match => match[1] ?? match[2]);

test("every package-root export is selected exactly once and rendered in its family", () => {
  const configPath = fileURLToPath(new URL("tsconfig.json", import.meta.url));
  const parsed = ts.getParsedCommandLineOfConfigFile(configPath, {}, {
    ...ts.sys, onUnRecoverableConfigFileDiagnostic: diagnostic => assert.fail(ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
  });
  const program = ts.createProgram(parsed.fileNames, parsed.options);
  assert.deepEqual(ts.getPreEmitDiagnostics(program), []);
  const checker = program.getTypeChecker();
  const exportsAt = path => checker.getExportsOfModule(checker.getSymbolAtLocation(program.getSourceFile(fileURLToPath(new URL(path, import.meta.url)))));
  const resolve = symbol => symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
  const publicExports = exportsAt("../../js/src/swmmrs/index.ts");
  // The default export is the same function, not a second API declaration.
  assert.equal(resolve(publicExports.find(s => s.name === "default")), resolve(publicExports.find(s => s.name === "runSwmm")));
  assert.match(summary(declaration("simulation", "runSwmm").signatures[0]), /package default/);
  const selected = [];
  for (const family of families) {
    const symbols = exportsAt(`${family}.ts`);
    assert.deepEqual(symbols.map(s => s.name).sort(), modules.get(family).children.map(s => s.name).sort());
    for (const symbol of symbols) {
      const exported = publicExports.find(s => s.name === symbol.name);
      assert.ok(exported, `${family}: non-public export ${symbol.name}`);
      assert.equal(resolve(symbol), resolve(exported), `${symbol.name}: wrapper must re-export the source declaration`);
      selected.push(symbol.name);
    }
  }
  assert.equal(new Set(selected).size, selected.length, "duplicate public export across families");
  assert.deepEqual(selected.sort(), publicExports.map(s => s.name).filter(name => name !== "default").sort());
  assert.deepEqual([...modules.keys()].sort(), [...families].sort());
});

test("all declared public contracts have summaries and method parameter descriptions", () => {
  let checked = 0;
  const containerKinds = new Set([
    ReflectionKind.Project, ReflectionKind.Module, ReflectionKind.CallSignature,
    ReflectionKind.TypeParameter, ReflectionKind.Reference,
  ]);
  function visit(reflection, path) {
    if (!reflection || typeof reflection !== "object") return;
    path += reflection.name ? `.${reflection.name}` : "";
    // Anonymous callback types have their contract on the containing parameter.
    // Type parameters are explained by their containing declaration as needed.
    const callable = reflection.signatures ?? [reflection.getSignature, reflection.setSignature].filter(Boolean);
    if (reflection.kind && !containerKinds.has(reflection.kind)
        && !reflection.name.startsWith("__") && !reflection.inheritedFrom) {
      if (callable.length === 0) assert.ok(summary(reflection), `${path}: missing source summary`);
      checked++;
    }
    if (reflection.kind !== ReflectionKind.TypeLiteral) { // Anonymous function-type contracts belong on their parameter.
      for (const signature of callable) {
        if (signature.inheritedFrom) continue;
        assert.ok(summary(signature), `${path}: missing callable summary`);
        for (const parameter of signature.parameters ?? []) {
          assert.ok(summary(parameter), `${path}.${parameter.name}: missing parameter contract`);
        }
      }
    }
    for (const key of ["children", "signatures", "getSignature", "setSignature", "type", "declaration", "types"]) {
      const value = reflection[key];
      if (Array.isArray(value)) value.forEach(child => visit(child, path));
      else visit(value, path);
    }
  }
  for (const [name, module] of modules) visit(module, name);
  assert.ok(checked > 800, `unexpectedly small reference: ${checked} declarations`);
});

test("Node exposes documented public members, not its constructor or worker plumbing", () => {
  const names = node.children.map(child => child.name);
  assert.ok(names.includes("configure"));
  assert.ok(names.includes("totalInflowVolume"));
  assert.ok(!names.includes("constructor"));
  assert.ok(!names.includes("create"));
  assert.ok(!names.some(name => name.includes("call")));
  for (const member of node.children) {
    for (const signature of member.signatures ?? []) {
      assert.ok(signature.comment?.blockTags.some(tag => tag.tag === "@returns"),
        `${member.name}: missing return contract`);
    }
  }
  for (const page of pages.values()) {
    assert.doesNotMatch(page, /(?:NodeOperations|LinkOperations|OutputOperations|WorkerClient|#call|__namedParameters)/);
  }
});

test("source comments render parameters, defaults, errors, examples, and inherited field docs", () => {
  assert.match(markdown, /configure\(patch: NodePatch\): Promise<void>/);
  assert.match(markdown, /Defaults to `false`: merge supplied IDs/);
  assert.match(markdown, /##### Throws/);
  assert.match(markdown, /await node\.configure\(\{ fullDepth: 4\.5, initialDepth: 0\.25 \}\)/);
  assert.match(markdown, /Cumulative lateral inflow volume in project volume units, not a flow rate/);
  const patch = declaration("node", "NodePatch");
  assert.match(summary(patch.children.find(child => child.name === "fullDepth")), /project length units/);
  assert.match(pages.get("output"), /Half-open range and I\/O strategy/);
  assert.match(pages.get("output"), /await reader\.readBulkSeries/);
  assert.match(pages.get("lid"), /Initial saturation percentage, 0–100/);
  assert.match(pages.get("lid"), /equivalent depth in in or mm/);
  assert.match(pages.get("subcatchment"), /Surface slope as a fraction, not percent/);
  assert.match(pages.get("subcatchment"), /False by default merges; true clears omitted pollutants to zero/);
  assert.match(pages.get("definitions"), /Sparse aquifer update/);
  assert.match(pages.get("rdii"), /12|twelve/);
});

test("TypeScript type punctuation survives Markdown rendering", () => {
  assert.match(markdown, /readonly \(readonly `number`\[\]\)\[\]/);
  for (const page of pages.values()) {
    assert.doesNotMatch(page, /readonly readonly/);
    assert.doesNotMatch(page, /\\[<>]/);
  }
  assert.match(markdown, /`Promise`&lt;`void`&gt;/);
});

test("all generated cross-references resolve across family pages with unique explicit anchors", () => {
  let links = 0;
  const ids = new Map([...pages].map(([family, page]) => [family, anchors(page)]));
  for (const [family, page] of pages) {
    assert.equal(new Set(ids.get(family)).size, ids.get(family).length, `${family}: duplicate anchor`);
    for (const [, href] of page.matchAll(/\]\(([^)]+)\)/g)) {
      if (/^https?:/.test(href)) continue;
      const target = new URL(href, `https://docs.invalid/${family}.md`);
      const name = target.pathname.slice(1).replace(/\.md$/, "");
      assert.ok(pages.has(name), `${family}: unknown page ${href}`);
      if (target.hash) assert.ok(ids.get(name).includes(decodeURIComponent(target.hash.slice(1))), `${family}: missing target ${href}`);
      links++;
    }
  }
  assert.ok(links > 500, `expected cross-linked declarations, got ${links}`);
  assert.ok(ids.get("node").includes("noderesults-1"), "preserve the Node pilot result anchor");
});

test("every API family has a thin include page registered in site navigation", () => {
  const nav = read("../../zensical.toml");
  for (const family of families) {
    const page = read(`../../docs/javascript/api/${family}.md`);
    assert.ok(page.includes(`--8<-- "docs/typescript-docs/.generated/${family}.md"`));
    assert.doesNotMatch(page, /^\|/m);
    assert.doesNotMatch(page, /^```/m);
    assert.ok(nav.includes(`"javascript/api/${family}.md"`), `${family}: missing navigation`);
  }
});
