import type { SimulationInfo } from "./types.js";

/** Transferable bytes and files captured at one quiescent simulation boundary. */
export interface CheckpointBundle {
  /** Bundle format identifier. */
  readonly format: "swmmrs.checkpoint.bundle";
  /** Supported bundle format version. */
  readonly version: 1;
  /** Isolated solver root used for manifest paths; never use it as a host path. */
  readonly root: string;
  /** Manifest path relative to `root`. */
  readonly manifestPath: string;
  /** Canonical core checkpoint JSON, untouched by the binding. */
  readonly manifest: Uint8Array;
  /** Manifest sidecars and validated external dependencies, keyed relative to `root`. */
  readonly files: Readonly<Record<string, Uint8Array>>;
}

type Operation<Args extends unknown[], Result> = { args: Args; result: Result };

/** Worker protocol additions for hotstarts, checkpoints, resume, and state load. */
export interface ScenarioOperations {
  useHotstart: Operation<[Uint8Array | null], void>;
  saveHotstart: Operation<[], Uint8Array>;
  exportCheckpoint: Operation<[], CheckpointBundle>;
  loadCheckpointState: Operation<[CheckpointBundle], void>;
  /** The thread count is the final argument, matching the ordinary open operation. */
  resumeCheckpoint: Operation<[CheckpointBundle, number], SimulationInfo>;
}

/** Low-level scenario operation names; prefer the methods on `Simulation` for execution. */
export type ScenarioMethod = keyof ScenarioOperations;
/** Argument tuple for a low-level scenario operation.
 * @typeParam K - Scenario operation name.
 */
export type ScenarioArgs<K extends ScenarioMethod> = ScenarioOperations[K]["args"];
/** Resolved result type for a low-level scenario operation.
 * @typeParam K - Scenario operation name.
 */
export type ScenarioResult<K extends ScenarioMethod> = ScenarioOperations[K]["result"];
