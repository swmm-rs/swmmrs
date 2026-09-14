import type { Call } from "../protocol.js";

/** Operations used by definition handles. Main extends the worker protocol with these entries. */
export interface DefinitionOperations {
  aquiferConfiguration: { args: [string]; result: AquiferConfiguration };
  configureAquifer: { args: [string, AquiferPatch]; result: void };
  snowmeltConfiguration: { args: [string]; result: SnowmeltParameterSetConfiguration };
  configureSnowmelt: { args: [string, SnowmeltParameterSetPatch]; result: void };
  snowmeltSurfaceConfiguration: { args: [string, SnowmeltSurfaceName]; result: SnowmeltSurfaceConfiguration };
  configureSnowmeltSurface: {
    args: [string, SnowmeltSurfaceName, SnowmeltSurfacePatch];
    result: void;
  };
}

/** Read-only identity for a configured definition. Generic handles do not expose editable curve/time-series data. */
export class Definition {
  /** Canonical configured definition ID. */
  readonly id: string;
  protected readonly call: Call;

  protected constructor(id: string, call: Call) {
    this.id = id;
    this.call = call;
    Object.freeze(this);
  }

  /** @internal */
  static create(id: string, call: Call): Definition { return new Definition(id, call); }
}

/** Read-only pollutant identity handle. */
export type Pollutant = Definition;
/** Read-only land-use identity handle. */
export type LandUse = Definition;
/** Read-only time-pattern identity handle. */
export type TimePattern = Definition;
/** Read-only curve identity handle. */
export type Curve = Definition;
/** Read-only time-series identity handle. */
export type TimeSeries = Definition;
/** Read-only control-rule identity handle. */
export type ControlRule = Definition;
/** Read-only transect identity handle. */
export type Transect = Definition;
/** Read-only custom-shape identity handle. */
export type CustomShape = Definition;
/** Read-only street identity handle. */
export type Street = Definition;
/** Read-only inlet-design identity handle. */
export type InletDesign = Definition;

/** Detached aquifer declarations in configured project units. */
export interface AquiferConfiguration {
  /** Soil porosity fraction. */
  readonly porosity: number;
  /** Wilting-point moisture fraction. */
  readonly wiltingPoint: number;
  /** Field-capacity moisture fraction. */
  readonly fieldCapacity: number;
  /** Saturated hydraulic conductivity in project rainfall-rate units. */
  readonly hydraulicConductivity: number;
  /** Native conductivity-response slope. */
  readonly conductivitySlope: number;
  /** Tension-response slope in project length units (ft or m). */
  readonly tensionSlope: number;
  /** Fraction of evaporation drawn from the upper zone. */
  readonly upperEvaporationFraction: number;
  /** Lower-zone evaporation depth in project length units. */
  readonly lowerEvaporationDepth: number;
  /** Lower-zone loss coefficient in in/h or mm/h. */
  readonly lowerLossCoefficient: number;
  /** Aquifer bottom elevation in ft or m. */
  readonly bottomElevation: number;
  /** Initial water-table elevation in ft or m. */
  readonly waterTableElevation: number;
  /** Initial upper-zone moisture fraction. */
  readonly upperMoisture: number;
  /** Canonical upper-evaporation time-pattern ID, or null. */
  readonly upperEvaporationPattern: string | null;
}

/** Sparse aquifer update; omitted fields retain their values. */
export interface AquiferPatch {
  /** {@inheritDoc AquiferConfiguration.porosity} */
  porosity?: number;
  /** {@inheritDoc AquiferConfiguration.wiltingPoint} */
  wiltingPoint?: number;
  /** {@inheritDoc AquiferConfiguration.fieldCapacity} */
  fieldCapacity?: number;
  /** {@inheritDoc AquiferConfiguration.hydraulicConductivity} */
  hydraulicConductivity?: number;
  /** {@inheritDoc AquiferConfiguration.conductivitySlope} */
  conductivitySlope?: number;
  /** {@inheritDoc AquiferConfiguration.tensionSlope} */
  tensionSlope?: number;
  /** {@inheritDoc AquiferConfiguration.upperEvaporationFraction} */
  upperEvaporationFraction?: number;
  /** {@inheritDoc AquiferConfiguration.lowerEvaporationDepth} */
  lowerEvaporationDepth?: number;
  /** {@inheritDoc AquiferConfiguration.lowerLossCoefficient} */
  lowerLossCoefficient?: number;
  /** {@inheritDoc AquiferConfiguration.bottomElevation} */
  bottomElevation?: number;
  /** {@inheritDoc AquiferConfiguration.waterTableElevation} */
  waterTableElevation?: number;
  /** {@inheritDoc AquiferConfiguration.upperMoisture} */
  upperMoisture?: number;
  /** Clear with null; omission retains the current relationship. */
  upperEvaporationPattern?: string | null;
}

/** Editable aquifer handle obtained from `simulation.aquifers`. */
export class Aquifer extends Definition {
  private constructor(id: string, call: Call) { super(id, call); }

  /** @internal */
  static create(id: string, call: Call): Aquifer { return new Aquifer(id, call); }

  /** Read aquifer declarations in any healthy owner state.
   * @returns Detached aquifer configuration in project units.
   */
  configuration(): Promise<AquiferConfiguration> {
    return this.call("aquiferConfiguration", this.id);
  }

  /** Atomically update aquifer declarations in `open` or `ended`; accepted edits return the owner to `open`.
   * @param patch - Sparse field values; null clears the optional time-pattern relationship.
   * @returns Resolves after all fields are accepted.
   */
  configure(patch: AquiferPatch): Promise<void> {
    return this.call("configureAquifer", this.id, patch);
  }
}

/** Snowmelt surface selector within a parameter set. */
export type SnowmeltSurfaceName = "plowable" | "impervious" | "pervious";

/** Detached snowmelt-surface declarations. */
export interface SnowmeltSurfaceConfiguration {
  /** Minimum melt coefficient in in/(h·°F) for US projects or mm/(h·°C) for SI projects. */
  readonly minimumMeltCoefficient: number;
  /** Maximum melt coefficient in in/(h·°F) for US projects or mm/(h·°C) for SI projects. */
  readonly maximumMeltCoefficient: number;
  /** Base melt temperature in °F for US projects or °C for SI projects. */
  readonly baseTemperature: number;
  /** Free-water holding-capacity fraction. */
  readonly freeWaterFraction: number;
  /** Initial snow water-equivalent depth in in or mm. */
  readonly initialSnowDepth: number;
  /** Initial free-water depth in in or mm. */
  readonly initialFreeWater: number;
  /** Snow water-equivalent depth for full coverage in in or mm. */
  readonly snowDepthForFullCoverage: number;
}

/** Sparse snowmelt-surface update; omission retains a field. */
export interface SnowmeltSurfacePatch {
  /** {@inheritDoc SnowmeltSurfaceConfiguration.minimumMeltCoefficient} */
  minimumMeltCoefficient?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.maximumMeltCoefficient} */
  maximumMeltCoefficient?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.baseTemperature} */
  baseTemperature?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.freeWaterFraction} */
  freeWaterFraction?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.initialSnowDepth} */
  initialSnowDepth?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.initialFreeWater} */
  initialFreeWater?: number;
  /** {@inheritDoc SnowmeltSurfaceConfiguration.snowDepthForFullCoverage} */
  snowDepthForFullCoverage?: number;
}

/** Detached snowmelt parameter set with all three surface records. */
export interface SnowmeltParameterSetConfiguration {
  /** Plowable fraction of impervious area. */
  readonly plowableFraction: number;
  /** Plowable-surface melt parameters. */
  readonly plowable: SnowmeltSurfaceConfiguration;
  /** Impervious-surface melt parameters. */
  readonly impervious: SnowmeltSurfaceConfiguration;
  /** Pervious-surface melt parameters. */
  readonly pervious: SnowmeltSurfaceConfiguration;
  /** Snow water-equivalent depth triggering plowing, in in or mm. */
  readonly plowDepth: number;
  /** Five removal fractions in native solver destination order. */
  readonly removalFractions: readonly [number, number, number, number, number];
  /** Canonical receiving subcatchment ID, or null. */
  readonly removalSubcatchment: string | null;
}

/** Sparse parameter-set update; use `configureSurface` for individual surface parameters. */
export interface SnowmeltParameterSetPatch {
  /** {@inheritDoc SnowmeltParameterSetConfiguration.plowableFraction} */
  plowableFraction?: number;
  /** {@inheritDoc SnowmeltParameterSetConfiguration.plowDepth} */
  plowDepth?: number;
  /** {@inheritDoc SnowmeltParameterSetConfiguration.removalFractions} */
  removalFractions?: readonly [number, number, number, number, number];
  /** Clear with null; omission retains the current relationship. */
  removalSubcatchment?: string | null;
}

/** Editable snowmelt parameter-set identity handle. */
export class SnowmeltParameterSet extends Definition {
  private constructor(id: string, call: Call) { super(id, call); }

  /** @internal */
  static create(id: string, call: Call): SnowmeltParameterSet {
    return new SnowmeltParameterSet(id, call);
  }

  /** Read all parameter-set declarations in a healthy owner state.
   * @returns Detached parameter set with three surface records.
   */
  configuration(): Promise<SnowmeltParameterSetConfiguration> {
    return this.call("snowmeltConfiguration", this.id);
  }

  /** Atomically update set-level declarations in `open` or `ended`; accepted edits return the owner to `open`.
   * @param patch - Sparse set fields; null clears the optional receiving-subcatchment relation.
   * @returns Resolves after the update is accepted.
   */
  configure(patch: SnowmeltParameterSetPatch): Promise<void> {
    return this.call("configureSnowmelt", this.id, patch);
  }

  /** Read one surface in a healthy owner state.
   * @param surface - Plowable, impervious, or pervious surface selector.
   * @returns Detached surface parameters.
   */
  surfaceConfiguration(surface: SnowmeltSurfaceName): Promise<SnowmeltSurfaceConfiguration> {
    return this.call("snowmeltSurfaceConfiguration", this.id, surface);
  }

  /** Atomically update one surface in `open` or `ended`; accepted edits return the owner to `open`.
   * @param surface - Surface to update.
   * @param patch - Sparse surface values; omitted fields retain their values.
   * @returns Resolves after all supplied fields are accepted.
   */
  configureSurface(surface: SnowmeltSurfaceName, patch: SnowmeltSurfacePatch): Promise<void> {
    return this.call("configureSnowmeltSurface", this.id, surface, patch);
  }
}

