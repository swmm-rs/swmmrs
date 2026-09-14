import { fromWorkerError, LifecycleError, WorkerError } from "./exceptions.js";
import type { Args, Call, Method, Response, Result } from "./protocol.js";

type Pending = { resolve: (value: unknown) => void; reject: (reason: unknown) => void };

/** Private transport for the worker shipped with this package. */
export class WorkerClient {
  readonly #worker: Worker;
  readonly #pending = new Map<number, Pending>();
  #nextId = 0;
  #failure: Error | undefined;
  #closing: Promise<void> | undefined;

  constructor(worker: Worker) {
    this.#worker = worker;
    worker.onerror = (event) => {
      event.preventDefault();
      this.stop(new WorkerError({ message: event.message || "SWMM worker failed" }));
    };
    worker.onmessageerror = () => this.stop(new WorkerError({ message: "Cannot read SWMM worker response" }));
    worker.onmessage = ({ data }: MessageEvent<Response>) => {
      const pending = this.#pending.get(data.id);
      if (!pending) return;
      this.#pending.delete(data.id);
      if (data.ok) pending.resolve(freezeRecord(data.result));
      else {
        const error = fromWorkerError(data.error);
        pending.reject(error);
        if (data.fatal) this.stop(error);
      }
    };
  }

  assertOpen(): void {
    if (this.#failure) throw this.#failure;
    if (this.#closing) throw new LifecycleError({ message: "Simulation is closing", operation: "close" });
  }

  readonly call: Call = <K extends Method>(method: K, ...args: Args<K>): Promise<Result<K>> => {
    try { this.assertOpen(); } catch (error) { return Promise.reject(error); }
    const id = this.#nextId++;
    return new Promise<Result<K>>((resolve, reject) => {
      // The bundled worker implements the same Operations contract. This is the
      // only type erasure needed to correlate request IDs with response values.
      this.#pending.set(id, { resolve: (value) => resolve(value as Result<K>), reject });
      try { this.#worker.postMessage({ id, method, args }); }
      catch (error) {
        this.#pending.delete(id);
        reject(error);
      }
    });
  };

  stop(error: Error): void {
    this.#failure ??= error;
    try { this.#worker.terminate(); } catch { /* The worker may already have exited. */ }
    for (const pending of this.#pending.values()) pending.reject(this.#failure);
    this.#pending.clear();
  }

  close(): Promise<void> {
    this.#closing ??= this.call("close").finally(() =>
      this.stop(new LifecycleError({ message: "Simulation is closed", operation: "close" })),
    );
    return this.#closing;
  }
}

/** Freeze copied domain records; output bytes remain caller-owned typed arrays. */
function freezeRecord<T>(value: T): T {
  if (value && typeof value === "object" && !ArrayBuffer.isView(value)) {
    for (const item of Object.values(value)) freezeRecord(item);
    Object.freeze(value);
  }
  return value;
}
