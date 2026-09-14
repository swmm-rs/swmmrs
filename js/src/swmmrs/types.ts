import type { FlowUnits, ModelTime, RoutingModel, SimulationState, UnitSystem } from "./enums.js";

/**
 * File contents, never a host path. Strings are text; `Blob` includes browser
 * `File` values. Byte views use their byte offset and length; Node `Buffer` is accepted.
 */
export type FileContents = string | Blob | ArrayBuffer | ArrayBufferView;
/** Worker creation options, distinct from the INP model options. */
export interface SimulationOptions {
  /**
   * Positive integer worker capacity. Node defaults to 1; isolated browser pages
   * may use hardware concurrency. One selects serial WASM; higher values require
   * browser cross-origin isolation. Effective solver threads may be capped by the model.
   */
  threads?: number;
  /** Worker asset URL override; otherwise resolved relative to the compiled module. */
  workerUrl?: string | URL;
}
/** Options for starting a run or executing one model to completion. */
export interface RunOptions {
  /** Retain binary report-period results; defaults to true. False returns empty output bytes. */
  saveResults?: boolean;
}
/** Automatic advancement and observation cadence for `Simulation.steps()`. */
export interface StepOptions extends RunOptions {
  /** Positive 32-bit integer seconds between observations. Omit for each routing step. */
  seconds?: number;
  /** Land exactly on observation boundaries; defaults to true. Ignored without `seconds`. False allows overshoot by a routing step. */
  strict?: boolean;
}
/** Detached model identities, solver metadata, and configured schedule from `simulation.info()`. */
export interface SimulationInfo {
  /** Configured project flow units. */
  readonly flowUnits: FlowUnits;
  /** Configured US or SI unit system. */
  readonly unitSystem: UnitSystem;
  /** Native solver release. */
  readonly solverVersion: string;
  /** Embedded native solver build identifier. */
  readonly solverBuildId: string;
  /** Configured routing model. */
  readonly routeModel: RoutingModel;
  /** Effective solver thread count after runtime and model caps. */
  readonly effectiveThreads: number;
  /** Canonical node IDs in configured order. */
  readonly nodeIds: readonly string[];
  /** Canonical link IDs in configured order. */
  readonly linkIds: readonly string[];
  /** Canonical subcatchment IDs in configured order. */
  readonly subcatchmentIds: readonly string[];
  /** Canonical rain-gage IDs in configured order. */
  readonly rainGageIds: readonly string[];
  /** Canonical pollutant IDs in configured order. */
  readonly pollutantIds: readonly string[];
  /** Canonical land-use IDs in configured order. */
  readonly landUseIds: readonly string[];
  /** Canonical time-pattern IDs in configured order. */
  readonly timePatternIds: readonly string[];
  /** Canonical curve IDs in configured order. */
  readonly curveIds: readonly string[];
  /** Canonical time-series IDs in configured order. */
  readonly timeSeriesIds: readonly string[];
  /** Canonical control-rule IDs in configured order. */
  readonly controlIds: readonly string[];
  /** Canonical transect IDs in configured order. */
  readonly transectIds: readonly string[];
  /** Canonical aquifer IDs in configured order. */
  readonly aquiferIds: readonly string[];
  /** Canonical snowmelt parameter-set IDs in configured order. */
  readonly snowmeltIds: readonly string[];
  /** Canonical custom-shape IDs in configured order. */
  readonly shapeIds: readonly string[];
  /** Canonical street IDs in configured order. */
  readonly streetIds: readonly string[];
  /** Canonical inlet-design IDs in configured order. */
  readonly inletDesignIds: readonly string[];
  /** Canonical AMM-model IDs in configured order. */
  readonly ammModelIds: readonly string[];
  /** Canonical RTK unit-hydrograph IDs in configured order. */
  readonly unitHydrographIds: readonly string[];
  /** Canonical LID-control IDs in configured order. */
  readonly lidControlIds: readonly string[];
  /** Configured simulation start, without a timezone. */
  readonly startTime: ModelTime;
  /** Configured simulation end, without a timezone. */
  readonly endTime: ModelTime;
  /** First reporting time, without a timezone. */
  readonly reportStart: ModelTime;
}
/** Detached lifecycle/progress record. `simulation.status()` is unavailable after close. */
export interface SimulationStatus {
  /** Current owner lifecycle state. */
  readonly state: SimulationState;
  /** Whether the native project is open. */
  readonly isOpen: boolean;
  /** Whether a solver run has started. */
  readonly isStarted: boolean;
  /** Whether accepted declaration changes need solver preparation. */
  readonly configurationDirty: boolean;
  /** Whether this run retains binary report-period results. */
  readonly saveResults: boolean;
  /** Current model time, or null before a result time exists. */
  readonly currentTime: ModelTime | null;
  /** Elapsed simulation time in seconds. */
  readonly elapsedSeconds: number;
  /** Configured simulation duration in seconds. */
  readonly durationSeconds: number;
  /** Completion in the range 0 to 100. */
  readonly percentComplete: number;
  /** Number of routing steps executed. */
  readonly stepCount: number;
  /** Number of report periods written. */
  readonly reportPeriodCount: number;
  /** Native warning count. */
  readonly warningCount: number;
}
/** Finalized, caller-owned files returned by `run()`, `finish()`, or `runSwmm()`. */
export interface RunResults {
  /** Finalized report text. */
  readonly report: string;
  /** Finalized binary output bytes; empty with `saveResults: false`. Not a live worker view. */
  readonly output: Uint8Array;
}
