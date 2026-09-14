import type { Call } from "../protocol.js";

/** One detached RTK short-, medium-, or long-response parameter set. */
export interface UnitHydrographResponse {
  /** Fraction of rainfall converted to RDII. */
  readonly rainfallFraction: number;
  /** Time to peak in hours. */
  readonly timeToPeakHours: number;
  /** Recession-to-time-to-peak ratio. */
  readonly recessionRatio: number;
  /** Maximum initial abstraction depth in inches or millimetres. */
  readonly maximumInitialAbstraction: number;
  /** Initial abstraction recovery rate in rainfall-depth units per day. */
  readonly initialAbstractionRecoveryRate: number;
  /** Initial abstraction depth at start in inches or millimetres. */
  readonly initialAbstractionAtStart: number;
}

/** One calendar month's named RTK responses. */
export interface UnitHydrographMonth {
  /** Short-response parameters. */
  readonly short: UnitHydrographResponse;
  /** Medium-response parameters. */
  readonly medium: UnitHydrographResponse;
  /** Long-response parameters. */
  readonly long: UnitHydrographResponse;
}

/** Complete January-through-December RTK response matrix. */
export interface UnitHydrographConfiguration {
  /** Canonical rain-gage ID. */
  readonly rainGage: string;
  /** Exactly twelve monthly records, January through December. */
  readonly monthlyResponses: readonly UnitHydrographMonth[];
}

/** Sparse RTK update. Omitted fields retain their current values. */
export interface UnitHydrographPatch {
  /** Replacement rain-gage ID. */
  rainGage?: string;
  /** Replacement complete January-through-December array, exactly twelve entries. */
  monthlyResponses?: readonly UnitHydrographMonth[];
}

/** Detached RTK RDII node assignment in native project units. IDs are canonical. */
export interface RdiiAssignment {
  /** Canonical receiving node ID. */
  readonly node: string;
  /** Canonical RTK unit-hydrograph ID. */
  readonly unitHydrograph: string;
  /** Assigned land area in acres (US) or hectares (SI). */
  readonly area: number;
}

export interface RtkOperations {
  unitHydrographConfiguration: { args: [string]; result: UnitHydrographConfiguration };
  configureUnitHydrograph: { args: [string, UnitHydrographPatch]; result: void };
  rdiiAssignments: { args: []; result: readonly RdiiAssignment[] };
  replaceRdiiAssignments: { args: [readonly RdiiAssignment[]]; result: void };
}

/** Editable RTK definition from `simulation.unitHydrographs`; node assignments are separate. */
export class UnitHydrograph {
  /** Canonical configured hydrograph ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) { this.id = id; this.#call = call; Object.freeze(this); }
  /** @internal */
  static create(id: string, call: Call): UnitHydrograph { return new UnitHydrograph(id, call); }
  /** Read the definition in any healthy owner state.
   * @returns Detached rain-gage identity and complete twelve-month response matrix.
   */
  configuration(): Promise<UnitHydrographConfiguration> { return this.#call("unitHydrographConfiguration", this.id); }
  /** Atomically update an RTK definition in `open` or `ended`; accepted edits return the owner to `open`.
   * @param patch - Sparse fields; a monthly array replaces all twelve months.
   * @returns Resolves after the complete patch is accepted.
   */
  configure(patch: UnitHydrographPatch): Promise<void> { return this.#call("configureUnitHydrograph", this.id, patch); }
}
