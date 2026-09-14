import { parentPort, workerData, Worker as NodeWorker } from "node:worker_threads";

if (!parentPort) throw new Error("worker-node.js must run in a worker thread");

const data = workerData && typeof workerData === "object" ? workerData : {};
const target = typeof data.target === "string"
  ? data.target
  : new URL("./lib/swmmrs/worker.js", import.meta.url).href;
const workerName = typeof data.name === "string" ? data.name : "";
const messageListeners = new Set();
const errorListeners = new Set();
const messageErrorListeners = new Set();
const pendingMessages = [];
let messageHandler;
let messageErrorHandler;

function errorEvent(error) {
  const value = error instanceof Error ? error : new Error(String(error));
  return { message: value.message, error: value, preventDefault() {} };
}

function dispatch(type, event) {
  const listeners = type === "message"
    ? messageListeners
    : type === "error" ? errorListeners : messageErrorListeners;
  const handler = type === "message" ? messageHandler
    : type === "messageerror" ? messageErrorHandler : globalThis[`on${type}`];
  if (type === "message" && listeners.size === 0 && typeof handler !== "function") {
    pendingMessages.push(event);
    return;
  }
  for (const listener of [...listeners]) listener(event);
  if (typeof handler === "function") handler(event);
}

function flushPendingMessages() {
  for (const event of pendingMessages.splice(0)) dispatch("message", event);
}

function setGlobal(name, value) {
  try {
    Object.defineProperty(globalThis, name, { configurable: true, writable: true, value });
  } catch {
    globalThis[name] = value;
  }
}

class NodeWebWorker {
  #worker;
  #terminated = false;
  #listeners = new Map();
  #pending = new Map();
  onmessage = null;
  onmessageerror = null;
  onerror = null;

  constructor(url, options = {}) {
    const childTarget = url instanceof URL ? url.href : String(url);
    this.#worker = new NodeWorker(new URL("./worker-node.js", import.meta.url), {
      type: "module",
      workerData: { target: childTarget, name: options.name ?? "" },
    });
    this.#worker.on("message", (value) => this.#dispatch("message", { data: value }));
    this.#worker.on("messageerror", (error) => this.#dispatch("messageerror", { data: error }));
    this.#worker.on("error", (error) => this.#dispatch("error", errorEvent(error)));
    this.#worker.on("exit", (code) => {
      if (!this.#terminated && code !== 0) this.#dispatch("error", errorEvent(new Error(`Worker exited with code ${code}`)));
      else if (!this.#terminated && code === 0) this.#dispatch("error", errorEvent(new Error("Worker exited unexpectedly")));
    });
  }

  addEventListener(type, listener) {
    if (typeof listener !== "function") return;
    const listeners = this.#listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.#listeners.set(type, listeners);
    for (const event of this.#pending.get(type)?.splice(0) ?? []) listener(event);
  }

  removeEventListener(type, listener) {
    this.#listeners.get(type)?.delete(listener);
  }

  postMessage(value, transfer) {
    this.#worker.postMessage(value, transfer);
  }

  terminate() {
    this.#terminated = true;
    return this.#worker.terminate();
  }

  #dispatch(type, event) {
    const listeners = this.#listeners.get(type);
    const property = type === "message" ? this.onmessage
      : type === "messageerror" ? this.onmessageerror : this.onerror;
    if ((!listeners || listeners.size === 0) && typeof property !== "function") {
      const pending = this.#pending.get(type) ?? [];
      pending.push(event);
      this.#pending.set(type, pending);
      return;
    }
    for (const listener of listeners ?? []) listener(event);
    if (typeof property === "function") property(event);
  }
}

setGlobal("self", globalThis);
setGlobal("name", workerName);
setGlobal("crossOriginIsolated", true);
setGlobal("Worker", NodeWebWorker);
setGlobal("postMessage", (value, transfer) => parentPort.postMessage(value, transfer));
setGlobal("addEventListener", (type, listener) => {
  if (type === "message") {
    messageListeners.add(listener);
    flushPendingMessages();
  } else if (type === "error") errorListeners.add(listener);
  else if (type === "messageerror") messageErrorListeners.add(listener);
});
setGlobal("removeEventListener", (type, listener) => {
  if (type === "message") messageListeners.delete(listener);
  else if (type === "error") errorListeners.delete(listener);
  else if (type === "messageerror") messageErrorListeners.delete(listener);
});
Object.defineProperty(globalThis, "onmessage", {
  configurable: true,
  get: () => messageHandler,
  set: (value) => {
    messageHandler = typeof value === "function" ? value : undefined;
    if (messageHandler) flushPendingMessages();
  },
});
Object.defineProperty(globalThis, "onmessageerror", {
  configurable: true,
  get: () => messageErrorHandler,
  set: (value) => { messageErrorHandler = typeof value === "function" ? value : undefined; },
});

parentPort.on("message", (value) => dispatch("message", { data: value }));
parentPort.on("messageerror", (error) => dispatch("messageerror", { data: error }));

await import(target);
