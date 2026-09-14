import type { Call } from "../protocol.js";

/** Canonical relationship identity returned by LID configuration reads. */
export type LidRelationshipKind = "lid" | "node" | "subcatchment" | "curve";
/** Detached canonical relationship identity. */
export interface LidRelationship<K extends LidRelationshipKind = LidRelationshipKind> {
  /** Target object family. */
  readonly kind: K;
  /** Canonical configured target ID. */
  readonly id: string;
}
/** Reference to a configured LID control. */
export type LidControlRelationship = LidRelationship<"lid">;
/** Explicit LID drain destination. */
export type LidDrainDestination = LidRelationship<"node" | "subcatchment">;
/** Reference to a configured drain-control curve. */
export type LidCurveRelationship = LidRelationship<"curve">;

/** Detached declarations for one subcatchment-local LID usage. */
export interface LidUnitConfiguration {
  /** Area per replicate in ft² or m², not acres or hectares. */
  readonly area: number;
  /** Full surface width in ft or m. */
  readonly fullWidth: number;
  /** Read-only prepared bottom width in ft or m. */
  readonly bottomWidth: number;
  /** Initial saturation percentage, 0–100. */
  readonly initialSaturation: number;
  /** Percentage of impervious runoff treated, 0–100. */
  readonly imperviousRunoffTreated: number;
  /** Percentage of pervious runoff treated, 0–100. */
  readonly perviousRunoffTreated: number;
  /** Configured LID-control identity. */
  readonly control: LidControlRelationship;
  /** Number of replicate units. */
  readonly count: number;
  /** Whether surface outflow is routed to the subcatchment's pervious area. */
  readonly routesToPervious: boolean;
  /** Explicit drain destination, or null for implicit outlet routing. */
  readonly drainDestination: LidDrainDestination | null;
}

/** Surface-layer declarations and prepared read-only values. */
export interface LidSurfaceConfiguration {
  /** Storage depth in in or mm. */
  readonly thickness: number;
  /** Vegetation volume fraction. */
  readonly vegetationVolumeFraction: number;
  /** Manning roughness coefficient. */
  readonly roughness: number;
  /** Surface slope in percent. */
  readonly slope: number;
  /** Side slope as a horizontal-to-vertical ratio. */
  readonly sideSlope: number;
  /** Derived surface runoff coefficient in project units. */
  readonly alpha: number;
  /** Whether ponded surface water overflows without delay. */
  readonly immediateOverflow: boolean;
}

/** Soil-layer declarations. */
export interface LidSoilConfiguration {
  /** Layer thickness in in or mm. */
  readonly thickness: number;
  /** Soil porosity fraction. */
  readonly porosity: number;
  /** Field-capacity moisture fraction. */
  readonly fieldCapacity: number;
  /** Wilting-point moisture fraction. */
  readonly wiltingPoint: number;
  /** Saturated conductivity in in/h or mm/h. */
  readonly saturatedConductivity: number;
  /** Conductivity-response slope. */
  readonly conductivitySlope: number;
  /** Capillary suction head in in or mm. */
  readonly suctionHead: number;
}

/** Storage-layer declarations. */
export interface LidStorageConfiguration {
  /** Layer thickness in in or mm. */
  readonly thickness: number;
  /** Void-to-solid volume ratio, not porosity. */
  readonly voidRatio: number;
  /** Saturated conductivity in in/h or mm/h. */
  readonly saturatedConductivity: number;
  /** Number of treated void volumes before clogging; zero disables clogging. */
  readonly cloggingFactor: number;
}

/** Pavement-layer declarations. */
export interface LidPavementConfiguration {
  /** Layer thickness in in or mm. */
  readonly thickness: number;
  /** Void-to-solid volume ratio, not porosity. */
  readonly voidRatio: number;
  /** Impervious pavement fraction. */
  readonly imperviousFraction: number;
  /** Saturated conductivity in in/h or mm/h. */
  readonly saturatedConductivity: number;
  /** Number of treated void volumes before clogging; zero disables clogging. */
  readonly cloggingFactor: number;
  /** Time between pavement regeneration events in seconds. */
  readonly regenerationIntervalSeconds: number;
  /** Fraction of permeability restored by regeneration. */
  readonly regenerationFraction: number;
}

/** Underdrain declarations; the control-curve relationship is read-only here. */
export interface LidDrainConfiguration {
  /** Drain equation coefficient in project units. */
  readonly coefficient: number;
  /** Drain equation head exponent. */
  readonly exponent: number;
  /** Drain offset above storage bottom in in or mm. */
  readonly offset: number;
  /** Dry-weather delay before draining in seconds. */
  readonly delaySeconds: number;
  /** Head at which the drain opens, in in or mm. */
  readonly openHead: number;
  /** Head at which the drain closes, in in or mm. */
  readonly closeHead: number;
  /** Configured control-curve identity, or null. */
  readonly controlCurve: LidCurveRelationship | null;
}

/** Drainage-mat declarations and prepared runoff coefficient. */
export interface LidDrainageMatConfiguration {
  /** Layer thickness in in or mm. */
  readonly thickness: number;
  /** Mat void-volume fraction. */
  readonly voidFraction: number;
  /** Manning roughness coefficient. */
  readonly roughness: number;
  /** Derived drainage-mat runoff coefficient in project units. */
  readonly alpha: number;
}

/** Configured optional layers for one LID control; absent layers are `null`. */
export interface LidControlConfiguration {
  /** Surface layer, or null when absent. */
  readonly surface: LidSurfaceConfiguration | null;
  /** Soil layer, or null when absent. */
  readonly soil: LidSoilConfiguration | null;
  /** Storage layer, or null when absent. */
  readonly storage: LidStorageConfiguration | null;
  /** Pavement layer, or null when absent. */
  readonly pavement: LidPavementConfiguration | null;
  /** Underdrain layer, or null when absent. */
  readonly drain: LidDrainConfiguration | null;
  /** Drainage mat, or null when absent. */
  readonly drainageMat: LidDrainageMatConfiguration | null;
}

/** Sparse surface-layer update; derived fields are not writable. */
export interface LidSurfacePatch {
  /** {@inheritDoc LidSurfaceConfiguration.thickness} */
  thickness?: number;
  /** {@inheritDoc LidSurfaceConfiguration.vegetationVolumeFraction} */
  vegetationVolumeFraction?: number;
  /** {@inheritDoc LidSurfaceConfiguration.roughness} */
  roughness?: number;
  /** {@inheritDoc LidSurfaceConfiguration.slope} */
  slope?: number;
  /** {@inheritDoc LidSurfaceConfiguration.sideSlope} */
  sideSlope?: number;
}
/** Sparse soil-layer update. */
export interface LidSoilPatch {
  /** {@inheritDoc LidSoilConfiguration.thickness} */
  thickness?: number;
  /** {@inheritDoc LidSoilConfiguration.porosity} */
  porosity?: number;
  /** {@inheritDoc LidSoilConfiguration.fieldCapacity} */
  fieldCapacity?: number;
  /** {@inheritDoc LidSoilConfiguration.wiltingPoint} */
  wiltingPoint?: number;
  /** {@inheritDoc LidSoilConfiguration.saturatedConductivity} */
  saturatedConductivity?: number;
  /** {@inheritDoc LidSoilConfiguration.conductivitySlope} */
  conductivitySlope?: number;
  /** {@inheritDoc LidSoilConfiguration.suctionHead} */
  suctionHead?: number;
}
/** Sparse storage-layer update. */
export interface LidStoragePatch {
  /** {@inheritDoc LidStorageConfiguration.thickness} */
  thickness?: number;
  /** {@inheritDoc LidStorageConfiguration.voidRatio} */
  voidRatio?: number;
  /** {@inheritDoc LidStorageConfiguration.saturatedConductivity} */
  saturatedConductivity?: number;
  /** {@inheritDoc LidStorageConfiguration.cloggingFactor} */
  cloggingFactor?: number;
}
/** Sparse pavement-layer update. */
export interface LidPavementPatch {
  /** {@inheritDoc LidPavementConfiguration.thickness} */
  thickness?: number;
  /** {@inheritDoc LidPavementConfiguration.voidRatio} */
  voidRatio?: number;
  /** {@inheritDoc LidPavementConfiguration.imperviousFraction} */
  imperviousFraction?: number;
  /** {@inheritDoc LidPavementConfiguration.saturatedConductivity} */
  saturatedConductivity?: number;
  /** {@inheritDoc LidPavementConfiguration.cloggingFactor} */
  cloggingFactor?: number;
  /** {@inheritDoc LidPavementConfiguration.regenerationIntervalSeconds} */
  regenerationIntervalSeconds?: number;
  /** {@inheritDoc LidPavementConfiguration.regenerationFraction} */
  regenerationFraction?: number;
}
/** Sparse underdrain update; does not replace the control curve. */
export interface LidDrainPatch {
  /** {@inheritDoc LidDrainConfiguration.coefficient} */
  coefficient?: number;
  /** {@inheritDoc LidDrainConfiguration.exponent} */
  exponent?: number;
  /** {@inheritDoc LidDrainConfiguration.offset} */
  offset?: number;
  /** {@inheritDoc LidDrainConfiguration.delaySeconds} */
  delaySeconds?: number;
  /** {@inheritDoc LidDrainConfiguration.openHead} */
  openHead?: number;
  /** {@inheritDoc LidDrainConfiguration.closeHead} */
  closeHead?: number;
}
/** Sparse drainage-mat update; the derived coefficient is not writable. */
export interface LidDrainageMatPatch {
  /** {@inheritDoc LidDrainageMatConfiguration.thickness} */
  thickness?: number;
  /** {@inheritDoc LidDrainageMatConfiguration.voidFraction} */
  voidFraction?: number;
  /** {@inheritDoc LidDrainageMatConfiguration.roughness} */
  roughness?: number;
}

/** Layer selector accepted by `LidControl.configure`. */
export type LidLayerName = "surface" | "soil" | "storage" | "pavement" | "drain" | "drainage_mat";
/** Sparse patch for the selected LID layer; omission retains each field. */
export type LidLayerPatch =
  | LidSurfacePatch
  | LidSoilPatch
  | LidStoragePatch
  | LidPavementPatch
  | LidDrainPatch
  | LidDrainageMatPatch;

/** Sparse usage update; omitted fields retain their declarations. */
export interface LidUnitPatch {
  /** {@inheritDoc LidUnitConfiguration.area} */
  area?: number;
  /** {@inheritDoc LidUnitConfiguration.fullWidth} */
  fullWidth?: number;
  /** {@inheritDoc LidUnitConfiguration.initialSaturation} */
  initialSaturation?: number;
  /** {@inheritDoc LidUnitConfiguration.imperviousRunoffTreated} */
  imperviousRunoffTreated?: number;
  /** {@inheritDoc LidUnitConfiguration.perviousRunoffTreated} */
  perviousRunoffTreated?: number;
  /** Canonical configured LID-control ID. */
  control?: string;
  /** {@inheritDoc LidUnitConfiguration.count} */
  count?: number;
  /** {@inheritDoc LidUnitConfiguration.routesToPervious} */
  routesToPervious?: boolean;
  /** Explicit destination, or `null` to restore implicit outlet routing. */
  drainDestination?: LidDrainDestination | null;
}

/** Detached LID water balance and layer state. Water-balance totals are equivalent depths, not volumes. */
export interface LidUnitSnapshot {
  /** Cumulative inflow depth in in or mm. */
  readonly inflow: number;
  /** Cumulative evaporation depth in in or mm. */
  readonly evaporation: number;
  /** Cumulative infiltration depth in in or mm. */
  readonly infiltration: number;
  /** Cumulative surface outflow depth in in or mm. */
  readonly surfaceOutflow: number;
  /** Cumulative drain outflow depth in in or mm. */
  readonly drainOutflow: number;
  /** Initial stored water as an equivalent depth in in or mm. */
  readonly initialVolume: number;
  /** Final stored water as an equivalent depth in in or mm. */
  readonly finalVolume: number;
  /** Surface water depth in in or mm. */
  readonly surfaceDepth: number;
  /** Pavement water depth in in or mm. */
  readonly pavementDepth: number;
  /** Storage water depth in in or mm. */
  readonly storageDepth: number;
  /** Volumetric soil moisture fraction. */
  readonly soilMoisture: number;
  /** Elapsed dry time in seconds. */
  readonly dryTimeSeconds: number;
  /** Previous drain flow in project flow units. */
  readonly oldDrainFlow: number;
  /** Current drain flow in project flow units. */
  readonly newDrainFlow: number;
  /** Evaporation rate in in/h or mm/h. */
  readonly evaporationRate: number;
  /** Maximum native-soil infiltration rate in in/h or mm/h. */
  readonly maximumNativeInfiltrationRate: number;
  /** Surface inflow rate in in/h or mm/h. */
  readonly surfaceInflowRate: number;
  /** Surface infiltration rate in in/h or mm/h. */
  readonly surfaceInfiltrationRate: number;
  /** Surface evaporation rate in in/h or mm/h. */
  readonly surfaceEvaporationRate: number;
  /** Surface outflow rate in in/h or mm/h. */
  readonly surfaceOutflowRate: number;
  /** Pavement evaporation rate in in/h or mm/h. */
  readonly pavementEvaporationRate: number;
  /** Pavement percolation rate in in/h or mm/h. */
  readonly pavementPercolationRate: number;
  /** Soil evaporation rate in in/h or mm/h. */
  readonly soilEvaporationRate: number;
  /** Soil percolation rate in in/h or mm/h. */
  readonly soilPercolationRate: number;
  /** Storage inflow rate in in/h or mm/h. */
  readonly storageInflowRate: number;
  /** Storage exfiltration rate in in/h or mm/h. */
  readonly storageExfiltrationRate: number;
  /** Storage evaporation rate in in/h or mm/h. */
  readonly storageEvaporationRate: number;
  /** Storage drainage rate in in/h or mm/h. */
  readonly storageDrainRate: number;
  /** Previous surface-layer flux in ft/s or m/s, not rainfall-rate units. */
  readonly surfaceFluxRate: number;
  /** Previous soil-layer flux in ft/s or m/s. */
  readonly soilFluxRate: number;
  /** Previous storage-layer flux in ft/s or m/s. */
  readonly storageFluxRate: number;
  /** Previous pavement-layer flux in ft/s or m/s. */
  readonly pavementFluxRate: number;
}

/** Detached aggregate LID-group results for one subcatchment. */
export interface SubcatchmentLidSnapshot {
  /** Non-LID pervious area in ft² or m². */
  readonly perviousArea: number;
  /** Flow routed to pervious area in project flow units. */
  readonly flowToPerviousArea: number;
  /** Previous aggregate drain flow in project flow units. */
  readonly oldDrainFlow: number;
  /** Current aggregate drain flow in project flow units. */
  readonly newDrainFlow: number;
}

/** Worker operations owned by the LID family. */
export interface LidOperations {
  lidControlConfiguration: { args: [string]; result: LidControlConfiguration };
  configureLidControl: { args: [string, LidLayerName, LidLayerPatch]; result: void };
  lidUnitCount: { args: [string]; result: number };
  lidUnitConfiguration: { args: [string, number]; result: LidUnitConfiguration };
  configureLidUnit: { args: [string, number, LidUnitPatch]; result: void };
  lidUnitSnapshot: { args: [string, number]; result: LidUnitSnapshot };
  subcatchmentLidSnapshot: { args: [string]; result: SubcatchmentLidSnapshot };
}

/** Shared LID-control handle obtained from `simulation.lidControls`. */
export class LidControl {
  /** Canonical configured control ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) {
    this.id = id;
    this.#call = call;
    Object.freeze(this);
  }
  /** @internal */
  static create(id: string, call: Call): LidControl { return new LidControl(id, call); }
  /** Read configured layers in a healthy owner state.
   * @returns Detached layer records; absent layers are null. Derived fields retain their last prepared values until preparation.
   */
  configuration(): Promise<LidControlConfiguration> { return this.#call("lidControlConfiguration", this.id); }
  /** Atomically update an existing surface layer in `open` or `ended`; accepted edits return the owner to `open`.
   * @param layer - Existing layer to update; this does not add absent layers.
   * @param patch - Sparse layer fields in project units; omission retains a value.
   * @returns Resolves after all supplied fields are accepted.
   */
  configure(layer: "surface", patch: LidSurfacePatch): Promise<void>;
  /** Update an existing soil layer. Same state and atomicity contract as the surface overload.
   * @param layer - Soil selector.
   * @param patch - Sparse soil declarations.
   */
  configure(layer: "soil", patch: LidSoilPatch): Promise<void>;
  /** Update an existing storage layer. Same state and atomicity contract as the surface overload.
   * @param layer - Storage selector.
   * @param patch - Sparse storage declarations.
   */
  configure(layer: "storage", patch: LidStoragePatch): Promise<void>;
  /** Update an existing pavement layer. Same state and atomicity contract as the surface overload.
   * @param layer - Pavement selector.
   * @param patch - Sparse pavement declarations.
   */
  configure(layer: "pavement", patch: LidPavementPatch): Promise<void>;
  /** Update an existing underdrain. Same state and atomicity contract as the surface overload.
   * @param layer - Drain selector.
   * @param patch - Sparse drain declarations.
   */
  configure(layer: "drain", patch: LidDrainPatch): Promise<void>;
  /** Update an existing drainage mat. Same state and atomicity contract as the surface overload.
   * @param layer - Drainage-mat selector.
   * @param patch - Sparse mat declarations.
   */
  configure(layer: "drainage_mat", patch: LidDrainageMatPatch): Promise<void>;
  /** Update a dynamically selected existing layer. Same state and atomicity contract as the surface overload.
   * @param layer - Existing layer selector.
   * @param patch - Sparse declarations matching the selected layer.
   */
  configure(layer: LidLayerName, patch: LidLayerPatch): Promise<void>;
  configure(layer: LidLayerName, patch: LidLayerPatch): Promise<void> {
    return this.#call("configureLidControl", this.id, layer, patch);
  }
}


/** LID usage handle identified by its subcatchment and local index. */
export class LidUnit {
  /** Canonical owning subcatchment ID. */
  readonly subcatchmentId: string;
  /** Zero-based position within the owning subcatchment's LID usages. */
  readonly index: number;
  readonly #call: Call;
  private constructor(subcatchmentId: string, index: number, call: Call) {
    this.subcatchmentId = subcatchmentId;
    this.index = index;
    this.#call = call;
    Object.freeze(this);
  }
  /** @internal */
  static create(subcatchmentId: string, index: number, call: Call): LidUnit {
    return new LidUnit(subcatchmentId, index, call);
  }
  /** Read usage declarations in a healthy owner state.
   * @returns Detached configuration in project units.
   */
  configuration(): Promise<LidUnitConfiguration> {
    return this.#call("lidUnitConfiguration", this.subcatchmentId, this.index);
  }
  /** Atomically update this usage in `open` or `ended`; accepted edits return the owner to `open`.
   * @param patch - Sparse usage declarations; null restores implicit drain routing.
   * @returns Resolves after all supplied fields are accepted.
   */
  configure(patch: LidUnitPatch): Promise<void> {
    return this.#call("configureLidUnit", this.subcatchmentId, this.index, patch);
  }
  /** Read detached water balance and layer results in `running` or `complete`, before `end()`.
   * @returns One usage's current results; later steps do not mutate them.
   */
  snapshot(): Promise<LidUnitSnapshot> {
    return this.#call("lidUnitSnapshot", this.subcatchmentId, this.index);
  }
}

/** Asynchronously indexed LID units owned by one subcatchment. */
export class LidUnitCollection {
  /** Canonical owning subcatchment ID. */
  readonly subcatchmentId: string;
  readonly #call: Call;
  private constructor(subcatchmentId: string, call: Call) {
    this.subcatchmentId = subcatchmentId;
    this.#call = call;
    Object.freeze(this);
  }
  /** @internal */
  static create(subcatchmentId: string, call: Call): LidUnitCollection {
    return new LidUnitCollection(subcatchmentId, call);
  }
  /** Read the number of configured usages; unlike model collections this queries the worker.
   * @returns Subcatchment-local usage count.
   */
  count(): Promise<number> { return this.#call("lidUnitCount", this.subcatchmentId); }
  /** Resolve one subcatchment-local usage in a healthy owner state.
   * @param index - Zero-based usage index, not a configured LID-control index.
   * @returns New handle for the selected usage.
   * @throws `RangeError` if the index is non-integral, negative, or out of range.
   */
  async at(index: number): Promise<LidUnit> {
    if (!Number.isInteger(index) || index < 0) throw new RangeError(`LID unit index out of range: ${index}`);
    const count = await this.count();
    if (index >= count) throw new RangeError(`LID unit index out of range: ${index}`);
    return LidUnit.create(this.subcatchmentId, index, this.#call);
  }
}
