import type { FileContents } from "./types.js";

const encoder = new TextEncoder();

type NodeProcess = {
  versions?: { node?: unknown };
  getBuiltinModule?: (specifier: string) => unknown;
};
type NodeOsModule = { availableParallelism?: () => number };
type RuntimeGlobal = typeof globalThis & {
  process?: NodeProcess;
  navigator?: { hardwareConcurrency?: number };
};

const runtimeGlobal = globalThis as RuntimeGlobal;

/** True when this module is running in Node rather than a browser page/worker. */
export const isNodeRuntime = typeof runtimeGlobal.process?.versions?.node === "string";

function hardwareConcurrency(): number {
  const browserValue = runtimeGlobal.navigator?.hardwareConcurrency;
  if (typeof browserValue === "number" && Number.isFinite(browserValue) && browserValue > 0) return Math.floor(browserValue);
  const os = runtimeGlobal.process?.getBuiltinModule?.("node:os") as NodeOsModule | undefined;
  const nodeValue = os?.availableParallelism?.();
  return typeof nodeValue === "number" && Number.isFinite(nodeValue) && nodeValue > 0 ? Math.floor(nodeValue) : 1;
}

const hostThreadCapacity = hardwareConcurrency();

export interface RuntimeInfo {
  readonly maxThreads: number;
  readonly defaultThreads: number;
  readonly parallel: boolean;
}

async function runtimeCapacity(): Promise<number> {
  if (!isNodeRuntime || runtimeGlobal.navigator?.hardwareConcurrency !== undefined || hostThreadCapacity > 1) {
    return hostThreadCapacity;
  }
  const os = await dynamicImport("node:os") as NodeOsModule;
  const value = os.availableParallelism?.();
  return typeof value === "number" && Number.isFinite(value) && value > 0 ? Math.floor(value) : hostThreadCapacity;
}

/** Return host solver capacity without requiring browser globals in Node. */
export async function runtimeInfo(): Promise<RuntimeInfo> {
  const parallel = isNodeRuntime || Boolean(runtimeGlobal.crossOriginIsolated);
  const maxThreads = await runtimeCapacity();
  return {
    maxThreads,
    defaultThreads: isNodeRuntime ? 1 : parallel ? maxThreads : 1,
    parallel,
  };
}

export interface WorkerCreationOptions {
  workerUrl?: string | URL;
}

type NodeWorkerPort = {
  on(event: "message", listener: (value: unknown) => void): NodeWorkerPort;
  on(event: "messageerror" | "error", listener: (error: unknown) => void): NodeWorkerPort;
  on(event: "exit", listener: (code: number) => void): NodeWorkerPort;
  postMessage(value: unknown, transferList?: readonly ArrayBuffer[]): void;
  terminate(): Promise<number>;
};
type NodeWorkerConstructor = new (
  filename: string | URL,
  options?: { type?: "module"; workerData?: unknown },
) => NodeWorkerPort;
type NodeWorkerModule = { Worker?: NodeWorkerConstructor };
type NodeFsModule = { readFile(path: string | URL): Promise<Uint8Array> };

async function dynamicImport(specifier: string): Promise<unknown> {
  return import(specifier);
}

function messageError(reason: unknown): Error {
  return reason instanceof Error ? reason : new Error(String(reason));
}

/** DOM-shaped adapter around node:worker_threads.Worker. */
class NodeWorkerAdapter {
  #worker: NodeWorkerPort;
  #terminated = false;
  #failed = false;
  #pendingError: Error | undefined;
  #onerror: ((event: ErrorEvent) => void) | null = null;

  onmessage: ((event: MessageEvent<unknown>) => void) | null = null;
  onmessageerror: ((event: MessageEvent<unknown>) => void) | null = null;

  get onerror(): ((event: ErrorEvent) => void) | null {
    return this.#onerror;
  }

  set onerror(handler: ((event: ErrorEvent) => void) | null) {
    this.#onerror = handler;
    const pending = this.#pendingError;
    if (handler && pending) {
      this.#pendingError = undefined;
      queueMicrotask(() => handler(this.#errorEvent(pending)));
    }
  }

  constructor(worker: NodeWorkerPort) {
    this.#worker = worker;
    worker.on("message", (data) => this.onmessage?.({ data } as MessageEvent<unknown>));
    worker.on("messageerror", (error) => this.onmessageerror?.({ data: error } as MessageEvent<unknown>));
    worker.on("error", (error) => this.#reportError(error));
    worker.on("exit", (code) => {
      if (!this.#terminated && code !== 0) this.#reportError(new Error(`SWMM worker exited with code ${code}`));
      else if (!this.#terminated && code === 0) this.#reportError(new Error("SWMM worker exited unexpectedly"));
    });
  }

  postMessage(message: unknown, transfer?: Transferable[]): void {
    this.#worker.postMessage(message, transfer as readonly ArrayBuffer[] | undefined);
  }

  terminate(): void {
    this.#terminated = true;
    void this.#worker.terminate();
  }

  #errorEvent(error: Error): ErrorEvent {
    return { message: error.message, error, preventDefault() {} } as ErrorEvent;
  }

  #reportError(reason: unknown): void {
    if (this.#terminated || this.#failed) return;
    this.#failed = true;
    const error = messageError(reason);
    if (this.#onerror) this.#onerror(this.#errorEvent(error));
    else this.#pendingError = error;
  }
}

/** Convert supported model input values to bytes without requiring browser globals. */
export async function bytes(value: FileContents): Promise<Uint8Array> {
  if (typeof value === "string") return encoder.encode(value);
  if (typeof Blob !== "undefined" && value instanceof Blob) return new Uint8Array(await value.arrayBuffer());
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (ArrayBuffer.isView(value)) return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  throw new TypeError("SWMM files must be strings, Blobs, ArrayBuffers, or typed arrays");
}

function wasmInitializer(bindings: unknown): (input?: unknown) => Promise<unknown> {
  if (!bindings || typeof bindings !== "object" || !("default" in bindings)) {
    throw new TypeError("WASM bindings must export a default initializer");
  }
  const initializer = bindings.default;
  if (typeof initializer !== "function") throw new TypeError("WASM bindings must export a default initializer");
  return initializer as (input?: unknown) => Promise<unknown>;
}

/** Read a generated wasm file from disk in Node. Browser callers should use loadWasm. */
export async function wasmInput(url: string | URL): Promise<Uint8Array> {
  if (!isNodeRuntime) throw new Error("wasmInput is only available in Node");
  const fs = await dynamicImport("node:fs/promises") as NodeFsModule;
  const value = await fs.readFile(url);
  return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
}
/** Initialize generated wasm with file bytes in Node and its normal URL loader in browsers. */
export async function loadWasm(bindings: unknown, moduleUrl: string | URL): Promise<unknown> {
  const initializer = wasmInitializer(bindings);
  if (!isNodeRuntime) return initializer();
  return initializer({ module_or_path: await wasmInput(new URL("./swmmrs_bg.wasm", moduleUrl)) });
}

/** Create one isolated worker using browser Worker or node:worker_threads. */
export async function createWorker(options: WorkerCreationOptions = {}): Promise<Worker> {
  if (!isNodeRuntime) {
    const WorkerConstructor = runtimeGlobal.Worker;
    if (typeof WorkerConstructor !== "function") throw new Error("SWMM needs browser workers");
    const url = options.workerUrl ?? new URL("./worker.js", import.meta.url);
    return new WorkerConstructor(url, { type: "module" });
  }

  const { Worker: NodeWorker } = await dynamicImport("node:worker_threads") as NodeWorkerModule;
  if (!NodeWorker) throw new Error("Node worker_threads are unavailable");
  const target = options.workerUrl ?? new URL("./worker.js", import.meta.url);
  const bootstrap = new URL("../../worker-node.js", import.meta.url);
  return new NodeWorkerAdapter(new NodeWorker(bootstrap, {
    type: "module",
    workerData: { target: target instanceof URL ? target.href : target, name: "" },
  })) as unknown as Worker;
}
