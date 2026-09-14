import type { Call } from "../protocol.js";

/** Standard runoff-response or seasonal baseflow AMM component. */
export type AmmComponentKind = "standard" | "baseflow";

/** Detached standard AMM component. `kind` is exactly `"standard"`. */
export interface AmmStandardComponent {
  /** Standard response discriminator. */
  readonly kind: "standard";
  /** Component identity within this model. */
  readonly id: string;
  /** Dry-weather capture parameter in the model's native capture units. */
  readonly dryCapture: number;
  /** Precipitation averaging window in hours. */
  readonly precipitationWindowHours: number;
  /** Response hydrograph half-life in hours. */
  readonly hydrographHalfLifeHours: number;
  /** Initial wet capture parameter in native capture units. */
  readonly initialWetCapture: number;
  /** Antecedent moisture half-life in hours. */
  readonly moistureHalfLifeHours: number;
  /** Temperature averaging window in hours. */
  readonly temperatureWindowHours: number;
  /** Spring cold seasonal hydrograph capture factor. */
  readonly springColdShcf: number;
  /** Hot seasonal hydrograph capture factor. */
  readonly hotShcf: number;
  /** Fall cold seasonal hydrograph capture factor. */
  readonly fallColdShcf: number;
}

/** Detached baseflow AMM component. `kind` is exactly `"baseflow"`. */
export interface AmmBaseflowComponent {
  /** Seasonal baseflow discriminator. */
  readonly kind: "baseflow";
  /** Component identity within this model. */
  readonly id: string;
  /** Precipitation averaging window in hours. */
  readonly precipitationWindowHours: number;
  /** Response hydrograph half-life in hours. */
  readonly hydrographHalfLifeHours: number;
  /** Temperature averaging window in hours. */
  readonly temperatureWindowHours: number;
  /** Spring cold capture parameter in native capture units. */
  readonly springColdCapture: number;
  /** Hot capture parameter in native capture units. */
  readonly hotCapture: number;
  /** Fall cold capture parameter in native capture units. */
  readonly fallColdCapture: number;
}

/** AMM component discriminated by `kind`; component arrays are replaced as a whole. */
export type AmmComponent = AmmStandardComponent | AmmBaseflowComponent;

/** Complete AMM model configuration in native project units. */
export interface AmmModelConfiguration {
  /** Canonical rain-gage ID. */
  readonly rainGage: string;
  /** Cold reference temperature in the model's configured temperature units. */
  readonly coldTemperature: number;
  /** Hot reference temperature in the model's configured temperature units. */
  readonly hotTemperature: number;
  /** Complete ordered standard/baseflow component list. */
  readonly components: readonly AmmComponent[];
}

/** Sparse AMM model update. Omitted fields retain their current values. */
export interface AmmModelPatch {
  /** {@inheritDoc AmmModelConfiguration.rainGage} */
  rainGage?: string;
  /** {@inheritDoc AmmModelConfiguration.coldTemperature} */
  coldTemperature?: number;
  /** {@inheritDoc AmmModelConfiguration.hotTemperature} */
  hotTemperature?: number;
  /** Replacement complete component array; omission retains it. */
  components?: readonly AmmComponent[];
}

/** Detached AMM node assignment in native project units. IDs are canonical. */
export interface AmmAssignment {
  /** Canonical receiving node ID. */
  readonly node: string;
  /** Canonical AMM-model ID. */
  readonly model: string;
  /** Assigned land area in acres (US) or hectares (SI). */
  readonly area: number;
}

export interface AmmOperations {
  ammModelConfiguration: { args: [string]; result: AmmModelConfiguration };
  configureAmmModel: { args: [string, AmmModelPatch]; result: void };
  ammAssignments: { args: []; result: readonly AmmAssignment[] };
  replaceAmmAssignments: { args: [readonly AmmAssignment[]]; result: void };
}

/** Editable AMM definition from `simulation.ammModels`; node assignments are managed on the simulation. */
export class AmmModel {
  /** Canonical configured model ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) { this.id = id; this.#call = call; Object.freeze(this); }
  /** @internal */
  static create(id: string, call: Call): AmmModel { return new AmmModel(id, call); }
  /** Read the definition in any healthy owner state.
   * @returns Detached complete AMM configuration.
   */
  configuration(): Promise<AmmModelConfiguration> { return this.#call("ammModelConfiguration", this.id); }
  /** Atomically update an AMM definition in `open` or `ended`; accepted edits return the owner to `open`.
   * @param patch - Sparse fields; supplying components replaces the complete array.
   * @returns Resolves after all fields are accepted.
   */
  configure(patch: AmmModelPatch): Promise<void> { return this.#call("configureAmmModel", this.id, patch); }
}
