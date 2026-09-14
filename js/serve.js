import { access, readFile, readdir } from "node:fs/promises";
import { createServer } from "node:http";

if (process.argv.includes("--help")) {
  console.log(`Usage: npm start [-- --no-isolation]

Serves the SWMM runner with the browser headers required by WebAssembly threads.
Pass --no-isolation to serve without those headers and use serial WASM by default.
Set PORT to change the default port (8080).

Example:
  npm run build && npm start`);
  process.exit(0);
}

const assets = new Map([
  ["/", ["index.html", "text/html; charset=utf-8"]],
  ["/index.html", ["index.html", "text/html; charset=utf-8"]],
  ["/test.html", ["test.html", "text/html; charset=utf-8"]],
  ["/index.js", ["index.js", "text/javascript; charset=utf-8"]],
  ["/worker.js", ["worker.js", "text/javascript; charset=utf-8"]],
]);
const headers = {
  ...(process.argv.includes("--no-isolation") ? {} : {
    "Cross-Origin-Opener-Policy": "same-origin",
    "Cross-Origin-Embedder-Policy": "require-corp",
    "Cross-Origin-Resource-Policy": "same-origin",
  }),
  "Cache-Control": "no-store",
};
const port = Number(process.env.PORT ?? 8080);

if (!Number.isInteger(port) || port < 1 || port > 65535) {
  console.error("error: PORT must be an integer from 1 to 65535\nexample: PORT=8081 npm start");
  process.exit(1);
}

try {
  await access(new URL("dist/swmmrs_bg.wasm", import.meta.url));
  await access(new URL("dist/serial/swmmrs_bg.wasm", import.meta.url));
  for (const directory of ["dist", "lib"]) {
    for (const path of await readdir(new URL(`${directory}/`, import.meta.url), { recursive: true })) {
      const type = path.endsWith(".js") ? "text/javascript; charset=utf-8"
        : path.endsWith(".wasm") ? "application/wasm"
        : path.endsWith(".map") ? "application/json" : undefined;
      if (type) assets.set(`/${directory}/${path}`, [`${directory}/${path}`, type]);
    }
  }
  await Promise.all(
    [...new Set([...assets.values()].map(([path]) => path))].map((path) =>
      access(new URL(path, import.meta.url)),
    ),
  );
} catch {
  console.error("error: browser files are missing; run npm run build first");
  process.exit(1);
}

const server = createServer(async (request, response) => {
  const asset = assets.get(new URL(request.url, "http://localhost").pathname);
  if (!asset) {
    response.writeHead(404, headers).end("Not found\n");
    return;
  }
  const [path, type] = asset;
  try {
    const contents = request.method === "HEAD" ? undefined : await readFile(new URL(path, import.meta.url));
    response.writeHead(200, { ...headers, "Content-Type": type }).end(contents);
  } catch {
    response.writeHead(500, headers).end("Cannot read asset\n");
  }
});

server.on("error", (error) => {
  console.error(`error: ${error.message}`);
  process.exitCode = 1;
});
server.listen(port, "127.0.0.1", () => {
  console.log(`SWMM runner: http://127.0.0.1:${port}`);
});
