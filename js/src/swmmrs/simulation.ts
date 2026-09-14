import { WorkerClient } from "./client.js";
import { LifecycleError, WorkerError } from "./exceptions.js";
import type { ModelTime, SimulationState } from "./enums.js";
import { ObjectCollection, NodeCollection, LinkCollection, SubcatchmentCollection, RainGageCollection } from "./objects/collections.js";
import { SimulationOptionsView, type SchedulePatch } from "./objects/options.js";
import { Definition, Aquifer, SnowmeltParameterSet } from "./objects/definitions.js";
import { AmmModel, type AmmAssignment } from "./objects/amm.js";
import { UnitHydrograph, type RdiiAssignment } from "./objects/rtk.js";
import { LidControl } from "./objects/lids.js";
import type { CheckpointBundle } from "./scenarios.js";
import { bytes, createWorker, runtimeInfo } from "./runtime.js";
import type { NodeResults, LinkResults, SimulationStatistics } from "./snapshots.js";
import type { FileContents, SimulationOptions, RunOptions, StepOptions, SimulationInfo, SimulationStatus, RunResults } from "./types.js";


/**
 * One isolated project, worker pool, and in-memory file system. Open with
 * {@link open}, not the private constructor, and always await {@link close}.
 * Handles and collections share this owner's lifecycle. Only one iterator or
 * manual advancement may own the solver; result reads and allowed controls can
 * run between observations. Records already returned remain usable after close.
 */
export class Simulation implements AsyncIterable<ModelTime>, AsyncDisposable {
  /** Configured-order node handles and aligned snapshot reads. */
  readonly nodes: NodeCollection;
  /** Configured-order link handles and aligned snapshot reads. */
  readonly links: LinkCollection;
  /** Configured-order subcatchment handles and aligned snapshot reads. */
  readonly subcatchments: SubcatchmentCollection;
  /** Configured-order rain-gage handles; no collection snapshot API. */
  readonly rainGages: RainGageCollection;
  /** Stable model-option view, distinct from worker creation options. */
  readonly options: SimulationOptionsView;
  /** Pollutant identities; definition editing is not exposed. */
  readonly pollutants: ObjectCollection<Definition>;
  /** Land-use identities. */
  readonly landUses: ObjectCollection<Definition>;
  /** Time-pattern identities. */
  readonly timePatterns: ObjectCollection<Definition>;
  /** Curve identities. */
  readonly curves: ObjectCollection<Definition>;
  /** Time-series identities; data editing is not exposed. */
  readonly timeSeries: ObjectCollection<Definition>;
  /** Control-rule identities. */
  readonly controls: ObjectCollection<Definition>;
  /** Transect identities. */
  readonly transects: ObjectCollection<Definition>;
  /** Editable aquifer definitions. */
  readonly aquifers: ObjectCollection<Aquifer>;
  /** Editable snowmelt parameter sets. */
  readonly snowmeltSets: ObjectCollection<SnowmeltParameterSet>;
  /** Custom cross-section shape identities. */
  readonly shapes: ObjectCollection<Definition>;
  /** Street identities. */
  readonly streets: ObjectCollection<Definition>;
  /** Inlet-design identities. */
  readonly inletDesigns: ObjectCollection<Definition>;
  /** Editable AMM definitions; node assignments are separate. */
  readonly ammModels: ObjectCollection<AmmModel>;
  /** Editable RTK definitions; RDII assignments are separate. */
  readonly unitHydrographs: ObjectCollection<UnitHydrograph>;
  /** Editable LID-control layers; unit placements belong to subcatchments. */
  readonly lidControls: ObjectCollection<LidControl>;
  readonly #client: WorkerClient;
  #iterating = false;
  #advancing = false;
  #closed = false;
  #terminateRequested = false;

  private constructor(client: WorkerClient, info: SimulationInfo) {
    this.#client = client;
    const assertOpen = () => client.assertOpen();
    this.nodes = NodeCollection.create(info.nodeIds, client.call, assertOpen);
    this.links = LinkCollection.create(info.linkIds, client.call, assertOpen);
    this.subcatchments = SubcatchmentCollection.create(info.subcatchmentIds, client.call, assertOpen);
    this.rainGages = RainGageCollection.create(info.rainGageIds, client.call, assertOpen);
    this.options = SimulationOptionsView.create(client.call);
    this.pollutants = new ObjectCollection(info.pollutantIds, id => Definition.create(id, client.call), assertOpen);
    this.landUses = new ObjectCollection(info.landUseIds, id => Definition.create(id, client.call), assertOpen);
    this.timePatterns = new ObjectCollection(info.timePatternIds, id => Definition.create(id, client.call), assertOpen);
    this.curves = new ObjectCollection(info.curveIds, id => Definition.create(id, client.call), assertOpen);
    this.timeSeries = new ObjectCollection(info.timeSeriesIds, id => Definition.create(id, client.call), assertOpen);
    this.controls = new ObjectCollection(info.controlIds, id => Definition.create(id, client.call), assertOpen);
    this.transects = new ObjectCollection(info.transectIds, id => Definition.create(id, client.call), assertOpen);
    this.aquifers = new ObjectCollection(info.aquiferIds, id => Aquifer.create(id, client.call), assertOpen);
    this.snowmeltSets = new ObjectCollection(info.snowmeltIds, id => SnowmeltParameterSet.create(id, client.call), assertOpen);
    this.shapes = new ObjectCollection(info.shapeIds, id => Definition.create(id, client.call), assertOpen);
    this.streets = new ObjectCollection(info.streetIds, id => Definition.create(id, client.call), assertOpen);
    this.inletDesigns = new ObjectCollection(info.inletDesignIds, id => Definition.create(id, client.call), assertOpen);
    this.ammModels = new ObjectCollection(info.ammModelIds, id => AmmModel.create(id, client.call), assertOpen);
    this.unitHydrographs = new ObjectCollection(info.unitHydrographIds, id => UnitHydrograph.create(id, client.call), assertOpen);
    this.lidControls = new ObjectCollection(info.lidControlIds, id => LidControl.create(id, client.call), assertOpen);
  }

  /** Parse an INP in a new independent worker.
   * @param input - INP contents, never a host path.
   * @param files - Supporting-file contents keyed by INP-relative paths; defaults to an empty record.
   * @param options - Worker URL and thread capacity; defaults to an empty record.
   * @returns An owner in `open`; close it when finished.
   * @throws Rejects malformed input, invalid thread capacity, unavailable runtime prerequisites, or solver parse failures.
   */
  static async open(input: FileContents, files: Readonly<Record<string, FileContents>> = {}, options: SimulationOptions = {}): Promise<Simulation> {
    const { maxThreads, defaultThreads, parallel } = await runtimeInfo();
    const threads = options.threads ?? defaultThreads;
    validateThreads(threads, maxThreads, parallel);
    const stagedFiles = Object.fromEntries(await Promise.all(
      Object.entries(files).map(async ([name, value]) => [name, await bytes(value)] as const),
    ));
    const inputBytes = await bytes(input);
    const client = new WorkerClient(await createWorker(options));
    try {
      const info = await client.call("open", inputBytes, stagedFiles, threads);
      return new Simulation(client, info);
    } catch (error) {
      client.stop(error instanceof Error ? error : new Error(String(error)));
      throw error;
    }
  }

  /** Restore a checkpoint in an independent worker without rerunning the model.
   * @param checkpoint - Complete bundle, including manifest sidecars and dependencies.
   * @param options - Worker creation options (the second argument); defaults to an empty record.
   * @returns An independent owner; no worker or mutable state is shared with the source.
   * @throws Rejects unsupported, corrupt, mismatched, or incomplete checkpoint bundles.
   */
  static async resume(checkpoint: CheckpointBundle, options: SimulationOptions = {}): Promise<Simulation> {
    const { maxThreads, defaultThreads, parallel } = await runtimeInfo();
    const threads = options.threads ?? defaultThreads;
    validateThreads(threads, maxThreads, parallel);
    const client = new WorkerClient(await createWorker(options));
    try {
      return new Simulation(client, await client.call("resumeCheckpoint", checkpoint, threads));
    } catch (error) {
      client.stop(error instanceof Error ? error : new Error(String(error)));
      throw error;
    }
  }

  /** Read native lifecycle state, including after cleanup.
   * @returns The current state, or `closed` after close completes.
   */
  getState(): Promise<SimulationState> { return this.#closed ? Promise.resolve("closed") : this.#client.call("state"); }
  /** Read model identities, units, solver build, effective threads, and schedule while healthy.
   * @returns Detached model information.
   */
  info(): Promise<SimulationInfo> { return this.#client.call("info"); }
  /** Read lifecycle and progress before close.
   * @returns Detached status; unavailable after cleanup.
   */
  status(): Promise<SimulationStatus> { return this.#client.call("status"); }
  /** Update calendar boundaries in `open` or `ended`; accepted edits return `ended` to `open`.
   * @param patch - Sparse timezone-free schedule. Omitted times retain their values.
   * @returns Resolves after the schedule update is accepted.
   */
  updateSchedule(patch: SchedulePatch): Promise<void> { return this.#advance("updateSchedule", () => this.#client.call("updateSchedule", patch)); }
  /** Read the complete AMM node-assignment list.
   * @returns Canonical node/model IDs and areas in project land-area units.
   */
  ammAssignments(): Promise<readonly AmmAssignment[]> { return this.#client.call("ammAssignments"); }
  /** Atomically replace all AMM assignments in `open` or `ended`; accepted edits return the owner to `open`.
   * @param assignments - Complete replacement list; an empty list clears all assignments.
   * @returns Resolves after all assignments are accepted.
   */
  replaceAmmAssignments(assignments: readonly AmmAssignment[]): Promise<void> { return this.#client.call("replaceAmmAssignments", assignments); }
  /** Read the complete RTK RDII assignment list.
   * @returns Canonical node/unit-hydrograph IDs and project land areas.
   */
  rdiiAssignments(): Promise<readonly RdiiAssignment[]> { return this.#client.call("rdiiAssignments"); }
  /** Atomically replace all RTK assignments in `open` or `ended`; accepted edits return the owner to `open`.
   * @param assignments - Complete replacement list; an empty list clears all assignments.
   * @returns Resolves after all assignments are accepted.
   */
  replaceRdiiAssignments(assignments: readonly RdiiAssignment[]): Promise<void> { return this.#client.call("replaceRdiiAssignments", assignments); }

  /** Configure hotstart input in `open` or `ended`; accepted edits return the owner to `open`.
   * @param input - Hotstart file contents, or null to clear the configured input.
   * @returns Resolves after the configuration write.
   */
  useHotstart(input: FileContents | null): Promise<void> {
    return this.#advance("useHotstart", async () => this.#client.call("useHotstart", input === null ? null : await bytes(input)));
  }
  /** Save current hydraulic state in `running` or `complete`.
   * @returns Caller-owned EPA hotstart bytes.
   */
  saveHotstart(): Promise<Uint8Array> { return this.#client.call("saveHotstart"); }
  /** Capture a quiescent `running` or `complete` owner, including validated dependencies.
   * @returns Complete portable checkpoint bundle with manifest sidecars.
   */
  saveCheckpoint(): Promise<CheckpointBundle> { return this.#client.call("exportCheckpoint"); }
  /** Stage physical and numerical checkpoint state in an `open` owner.
   * Declarations, files, and persistent forcings remain those of the receiver.
   * @param checkpoint - Compatible bundle with all required dependencies.
   * @returns Resolves with the owner still `open`; state is applied by its next start.
   * @throws Rejects incompatible identity, unsupported dependencies, corrupt manifests, or missing sidecars.
   */
  loadCheckpointState(checkpoint: CheckpointBundle): Promise<void> {
    return this.#advance("loadCheckpointState", () => this.#client.call("loadCheckpointState", checkpoint));
  }
  /** Save a checkpoint and resume it in a new independent owner.
   * @param options - Worker options for the child; defaults to an empty record.
   * @returns Independent simulation with copied run state and forcings.
   */
  async fork(options: SimulationOptions = {}): Promise<Simulation> {
    return Simulation.resume(await this.saveCheckpoint(), options);
  }

  /** Start an `open` or `ended` model.
   * @param options - Run options; `saveResults` defaults to true.
   * @returns Resolves after solver initialization succeeds.
   */
  start({ saveResults = true }: RunOptions = {}): Promise<void> {
    validateSaveResults(saveResults);
    return this.#advance("start", () => this.#client.call("start", saveResults));
  }
  /** Advance one routing step in a started run, without an active iterator.
   * @returns New model time, or null at natural completion.
   */
  step(): Promise<ModelTime | null> { return this.#advance("step", () => this.#client.call("step")); }
  /** Advance an observation interval in a started run.
   * @param seconds - Positive 32-bit integer interval in seconds.
   * @param strict - Defaults to true: land exactly on the boundary. False allows whole-step overshoot.
   * @returns New model time, or null at natural completion.
   */
  async stride(seconds: number, strict = true): Promise<ModelTime | null> {
    validateSeconds(seconds);
    if (typeof strict !== "boolean") throw new TypeError("strict must be a boolean");
    return this.#advance("stride", () => this.#client.call("stride", seconds, strict));
  }

  /** Automatic start and routing-step advancement; exhaustion retains final statistics.
   * @returns The same async generator as {@link steps} with default options.
   */
  [Symbol.asyncIterator](): AsyncGenerator<ModelTime, void, unknown> { return this.steps(); }

  /** Iterate observations, lazily starting `open` or `ended` owners on the first `next()`.
   * Early exit releases advancement ownership, leaving the run available to finish or resume.
   * @param options - Observation/run options; routing-step cadence, strict boundaries, and retained results by default.
   * @returns Generator of observation times. Natural exhaustion leaves final statistics available.
   * @throws Rejects invalid intervals/flags or competing advancement/finalization.
   */
  async *steps({ seconds, strict = true, saveResults = true }: StepOptions = {}): AsyncGenerator<ModelTime, void, unknown> {
    if (seconds !== undefined) validateSeconds(seconds);
    validateSaveResults(saveResults);
    if (typeof strict !== "boolean") throw new TypeError("strict must be a boolean");
    this.#assertManual("iterate");
    this.#iterating = true;
    try {
      const state = await this.#client.call("state");
      if (state === "open" || state === "ended") await this.#client.call("start", saveResults);
      if (state === "complete") return;
      while (true) {
        if (this.#terminateRequested) {
          await this.#client.call("end");
          return;
        }
        const time = seconds === undefined
          ? await this.#client.call("step")
          : await this.#client.call("stride", seconds, strict);
        if (time === null) return;
        yield time;
      }
    } finally {
      this.#iterating = false;
      this.#terminateRequested = false;
    }
  }

  /** End active iteration at its next observation boundary, without closing the owner.
   * @returns Immediately; the iterator performs the end operation at its next boundary.
   * @throws Synchronously throws LifecycleError if no iterator owns advancement.
   */
  terminate(): void {
    if (!this.#iterating) throw new LifecycleError({ message: "No active iterator owns advancement", operation: "terminate" });
    this.#terminateRequested = true;
  }

  #assertManual(operation: string): void {
    this.#client.assertOpen();
    if (this.#iterating) throw new LifecycleError({ message: "An active iterator owns simulation advancement", operation });
    if (this.#advancing) throw new LifecycleError({ message: "A simulation advancement is already pending", operation });
  }

  async #advance<T>(operation: string, execute: () => Promise<T>): Promise<T> {
    this.#assertManual(operation);
    this.#advancing = true;
    try { return await execute(); }
    finally { this.#advancing = false; }
  }

  /** Direct node result read retained for prototype callers; requires results to be available.
   * @param id - Node ID resolved in the worker, not by a local collection lookup.
   * @returns Detached node hydraulics; unknown IDs reject the promise.
   */
  node(id: string): Promise<NodeResults> { return this.#client.call("node", id); }
  /** Direct link result read retained for prototype callers.
   * @param id - Link ID resolved in the worker.
   * @returns Detached link hydraulics; unknown IDs reject the promise.
   */
  link(id: string): Promise<LinkResults> { return this.#client.call("link", id); }
  /** Direct equivalent of `nodes.get(id).setExternalInflow(flow)`.
   * @param id - Node ID resolved in the worker.
   * @param flow - Persistent additive inflow in project flow units.
   * @returns Resolves after updating the forcing in `open`, `running`, or `ended`.
   */
  setNodeExternalInflow(id: string, flow: number): Promise<void> { return this.#client.call("setNodeExternalInflow", id, flow); }
  /** Direct equivalent of `links.get(id).setTargetSetting(setting)`; requires `running`.
   * @param id - Link ID resolved in the worker.
   * @param setting - Dimensionless opening or pump speed factor.
   * @returns Resolves after updating the target; unknown IDs reject the promise.
   */
  setLinkTargetSetting(id: string, setting: number): Promise<void> { return this.#client.call("setLinkTargetSetting", id, setting); }

  /** Read system statistics in `running` or `complete`, before ending the run.
   * @returns Detached cumulative totals, continuity balances, and routing diagnostics.
   */
  statistics(): Promise<SimulationStatistics> { return this.#client.call("statistics"); }
  /** End the run, write requested reports, and finalize binary output; safe to repeat in `ended`.
   * Supports partial runs and retains the owner for result reads and reruns.
   * @returns Finalized report and caller-owned output bytes; output is empty with `saveResults: false`.
   */
  finish(): Promise<RunResults> { return this.#advance("finish", () => this.#client.call("finish")); }
  /** Start, run to completion, and finalize with one worker request.
   * @param options - Run options; `saveResults` defaults to true.
   * @returns Finalized files, retaining the owner in `ended` until closed or reused.
   */
  run({ saveResults = true }: RunOptions = {}): Promise<RunResults> {
    validateSaveResults(saveResults);
    return this.#advance("run", () => this.#client.call("run", saveResults));
  }
  /** End a `running` or `complete` run without detailed report tables.
   * @returns Resolves after flushing output/summary statistics and entering `ended`.
   */
  end(): Promise<void> { return this.#advance("end", () => this.#client.call("end")); }
  /** Append and flush only the runtime footer in `ended`, once, even without saved results.
   * JavaScript exposes this before close removes its in-memory files; Python report paths survive close.
   * @returns Resolves after the footer is flushed, without detailed report tables.
   */
  finalizeReport(): Promise<void> { return this.#advance("finalizeReport", () => this.#client.call("finalizeReport")); }
  /** Generate requested detailed report tables and footer in `ended`, once.
   * @returns Resolves after flushing the report.
   * @throws Rejects when the run did not retain binary results.
   */
  report(): Promise<void> { return this.#advance("report", () => this.#client.call("report")); }
  /** Discard run/results/output state in `open` or `ended` without reparsing the INP.
   * Retains declarations and persistent forcings; clears staged checkpoint state.
   * @returns Resolves with the owner in `open`.
   */
  resetSolver(): Promise<void> { return this.#advance("resetSolver", () => this.#client.call("resetSolver")); }
  /** Put active Dynamic Wave workers to sleep; requires `running`.
   * @returns Resolves after the power-state operation; lifecycle state is unchanged.
   */
  sleepWorkers(): Promise<void> { return this.#client.call("sleepWorkers"); }
  /** Copy a project or generated file before close; does not flush or finalize a run.
   * @param name - Path in the worker's project file system, not a host path.
   * @returns Caller-owned bytes. Call {@link finish} first for finalized report/output files.
   */
  readFile(name: string): Promise<Uint8Array> { return this.#client.call("readFile", name); }

  /** Release the project, workers, and in-memory files. Repeated calls share cleanup.
   * @returns Resolves when cleanup finishes; all owner-bound handles then become unusable.
   */
  async close(): Promise<void> {
    try { await this.#client.close(); }
    finally { this.#closed = true; }
  }
  /** Await resource cleanup for `await using`.
   * @returns The same cleanup promise as {@link close}.
   */
  [Symbol.asyncDispose](): Promise<void> { return this.close(); }
}

function validateSeconds(seconds: number): void {
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 2_147_483_647) throw new RangeError("seconds must be a positive 32-bit integer");
}

function validateSaveResults(value: boolean): void {
  if (typeof value !== "boolean") throw new TypeError("saveResults must be a boolean");
}

function validateThreads(threads: number, maximum: number, parallel: boolean): void {
  if (!Number.isInteger(threads) || threads < 1) throw new RangeError(`threads must be an integer between 1 and ${maximum}`);
  if (threads > 1 && !parallel) throw new WorkerError({ message: "Multiple threads require cross-origin isolation with COOP/COEP headers; use threads: 1 for serial execution", operation: "open" });
  if (threads > maximum) throw new RangeError(`threads must be an integer between 1 and ${maximum}`);
}

/**
 * Open, run, finalize, and close one INP model. Also exported as the package default.
 * @param input - INP contents, never a host path.
 * @param files - Supporting-file contents keyed by INP-relative paths; defaults to an empty record.
 * @param options - Worker and run options; retained results by default.
 * @returns Finalized report and caller-owned binary output after cleanup.
 * @throws Preserves the run error if cleanup also fails, attaching the secondary failure as `cleanupError`.
 */
export async function runSwmm(input: FileContents, files: Readonly<Record<string, FileContents>> = {}, options: SimulationOptions & RunOptions = {}): Promise<RunResults> {
  const simulation = await Simulation.open(input, files, options);
  let failure: unknown;
  try { return await simulation.run(options); }
  catch (error) { failure = error; throw error; }
  finally {
    try { await simulation.close(); }
    catch (cleanupError) {
      if (failure === undefined) throw cleanupError;
      if (failure instanceof Error) Object.assign(failure, { cleanupError });
    }
  }
}
