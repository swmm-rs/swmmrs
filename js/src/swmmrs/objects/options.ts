import type { CustomEllipseModel, InertiaDamping, ModelTime, NormalFlowLimit, SurchargeMethod } from "../enums.js";
import type { Call } from "../protocol.js";

/** Calendar month and day for seasonal sweeping; validated by the native solver. */
export type SweepDay = readonly [month: number, day: number];

/** Sparse model-option update. Omitted fields retain their values; tolerances are not writable. */
export interface ModelOptionsPatch {
  /** Nominal routing step in seconds. Resets Courant factor to zero unless both are supplied. */
  routingStepSeconds?: number;
  /** Binary report interval in whole seconds, independent of iterator observation cadence. */
  reportStepSeconds?: number;
  /** Enable detailed report tables; object selection still uses `includedInReport`. */
  detailedReportingEnabled?: boolean;
  /** Ellipse cross-section interpretation. */
  customEllipseModel?: CustomEllipseModel;
  /** Surcharge treatment. */
  surchargeMethod?: SurchargeMethod;
  /** Allow configured nodal surface ponding. */
  allowPonding?: boolean;
  /** Dynamic Wave inertia-damping rule. */
  inertiaDamping?: InertiaDamping;
  /** Normal-flow limiting rule. */
  normalFlowLimit?: NormalFlowLimit;
  /** Allow steady-state routing skips. */
  skipSteadyState?: boolean;
  /** Process rainfall; does not create missing definitions. */
  rainfallEnabled?: boolean;
  /** Process rainfall-dependent infiltration/inflow. */
  rdiiEnabled?: boolean;
  /** Process snowmelt. */
  snowmeltEnabled?: boolean;
  /** Process groundwater. */
  groundwaterEnabled?: boolean;
  /** Route hydraulic flow. */
  routingEnabled?: boolean;
  /** Process water quality. */
  qualityEnabled?: boolean;
  /** Control-rule interval in whole seconds. */
  ruleStepSeconds?: number;
  /** First month/day for seasonal sweeping. */
  sweepStart?: SweepDay;
  /** Last month/day for seasonal sweeping. */
  sweepEnd?: SweepDay;
  /** Maximum Dynamic Wave trial iterations. */
  maximumTrials?: number;
  /** Model-requested solver threads, distinct from the worker capacity in `SimulationOptions.threads`. */
  requestedThreads?: number;
  /** Lower bound for variable routing steps in seconds. */
  minimumRoutingStepSeconds?: number;
  /** Conduit-lengthening time step in seconds. */
  lengtheningStepSeconds?: number;
  /** Antecedent dry period in seconds. */
  antecedentDrySeconds?: number;
  /** Variable-step adjustment factor. Explicit writes must be positive; reads may contain the disabled value zero. */
  courantFactor?: number;
  /** Minimum nodal surface area in ft² or m². */
  minimumSurfaceArea?: number;
  /** Minimum conduit slope as a ratio, not a percentage. */
  minimumConduitSlope?: number;
}

/** Complete read-only option record, including all writable fields and solver tolerances. */
export interface ModelOptions extends Readonly<Required<ModelOptionsPatch>> {
  /** Read-only hydraulic head convergence tolerance. */
  readonly headTolerance: number;
  /** Read-only system-flow convergence tolerance. */
  readonly systemFlowTolerance: number;
  /** Read-only lateral-flow convergence tolerance. */
  readonly lateralFlowTolerance: number;
}

/** Sparse calendar boundaries for `Simulation.updateSchedule()`; omitted times retain their values. */
export interface SchedulePatch {
  /** Simulation start in timezone-free model calendar time. */
  startTime?: ModelTime;
  /** First reporting time in timezone-free model calendar time. */
  reportStart?: ModelTime;
  /** Simulation end in timezone-free model calendar time. */
  endTime?: ModelTime;
}

/** Option-family operations added to the shared worker protocol. */
export interface OptionsOperations {
  updateSchedule: { args: [SchedulePatch]; result: void };
  maximumRoutingStepSeconds: { args: []; result: number };
}

/** Stable model options. Updates are atomic and require an open or ended run. */
export class SimulationOptionsView {
  readonly #call: Call;
  private constructor(call: Call) { this.#call = call; }
  /** @internal */
  static create(call: Call): SimulationOptionsView { return new SimulationOptionsView(call); }
  /** Read the complete option record in a healthy owner state.
   * @returns Detached {@link ModelOptions}, including read-only solver tolerances.
   */
  read(): Promise<ModelOptions> { return this.#call("options"); }
  /** Atomically update model options in `open` or `ended`; accepted edits return `ended` to `open`.
   * @param patch - Sparse option values. Unknown keys, invalid types/enums, and solver-invalid values reject the whole patch.
   * @returns Resolves after all supplied fields are accepted.
   * @example
   * ```typescript
   * import type { Simulation } from "@swmmrs/swmmrs";
   * declare const simulation: Simulation;
   * await simulation.options.update({ routingStepSeconds: 30, reportStepSeconds: 300, qualityEnabled: true });
   * ```
   */
  update(patch: ModelOptionsPatch): Promise<void> { return this.#call("configureOptions", patch); }
  /** Read the maximum routing interval allowed by current configuration.
   * @returns Maximum routing interval in seconds.
   */
  maximumRoutingStepSeconds(): Promise<number> { return this.#call("maximumRoutingStepSeconds"); }
}
