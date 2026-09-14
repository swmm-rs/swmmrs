/**
 * Detached current node hydraulics returned by `node.results()`.
 * Values are frozen copies, not live worker views or report-period time series.
 */
export interface NodeResults {
  /** Water depth above the invert in project length units (ft or m). */
  readonly depth: number;
  /** Hydraulic head in project elevation units (ft or m). */
  readonly head: number;
  /** Stored water volume in project volume units (ft³ or m³). */
  readonly volume: number;
  /** Current lateral inflow in project flow units. */
  readonly lateralInflow: number;
  /** Current total inflow in project flow units. */
  readonly totalInflow: number;
  /** Current total outflow in project flow units. */
  readonly totalOutflow: number;
  /** Current loss rate in project flow units. */
  readonly losses: number;
  /** Current flooding rate in project flow units. */
  readonly flooding: number;
  /** Storage-node hydraulic retention time in seconds; `null` for other node kinds. */
  readonly hydraulicRetentionSeconds: number | null;
}

/** Detached current link hydraulics from `link.results()`, not a historical series. */
export interface LinkResults {
  /** Current opening or pump-speed factor. */
  readonly setting: number;
  /** Target opening or pump-speed factor; native control rules may change it. */
  readonly targetSetting: number;
  /** Open duration in seconds. */
  readonly timeOpenSeconds: number;
  /** Closed duration in seconds. */
  readonly timeClosedSeconds: number;
  /** Signed flow in configured orientation, in project flow units. */
  readonly flow: number;
  /** Water depth in ft or m. */
  readonly depth: number;
  /** Velocity magnitude in ft/s or m/s. */
  readonly velocity: number;
  /** Conduit surface width in ft or m; null for other link kinds. */
  readonly topWidth: number | null;
  /** Stored water volume in ft³ or m³. */
  readonly volume: number;
  /** Conduit capacity ratio, or non-conduit setting. */
  readonly capacity: number;
  /** Upstream surface area in ft² or m². */
  readonly upstreamSurfaceArea: number;
  /** Downstream surface area in ft² or m². */
  readonly downstreamSurfaceArea: number;
  /** Dimensionless Froude number. */
  readonly froudeNumber: number;
}

/** Detached current subcatchment runoff values, available while results exist. */
export interface SubcatchmentResults {
  /** Rainfall rate in in/h or mm/h. */
  readonly rainfall: number;
  /** Evaporation rate in in/day or mm/day. */
  readonly evaporation: number;
  /** Infiltration rate in in/h or mm/h. */
  readonly infiltration: number;
  /** Runon in project flow units. */
  readonly runon: number;
  /** Runoff in project flow units. */
  readonly runoff: number;
  /** Snow depth in inches or millimetres. */
  readonly snowDepth: number;
}

/** Detached current rain-gage rates in in/h or mm/h, not accumulated depths. */
export interface RainGageResults {
  /** Total current precipitation rate. */
  readonly totalPrecip: number;
  /** Current rainfall rate. */
  readonly rainfall: number;
  /** Current snowfall rate. */
  readonly snowfall: number;
  /** Selected persistent API source rate, or null when not selected. */
  readonly externalPrecipitationRate: number | null;
  /** Highest-priority rainfall override rate, or null when cleared. */
  readonly rainfallOverride: number | null;
}

/**
 * Detached, timestamp-free columns with one value per canonical object ID.
 * Requested selection order is preserved; an empty selection yields empty columns.
 * @typeParam T - Per-object result record defining each column's values and units.
 */
export type Snapshot<T> = {
  /** Canonical object IDs defining every column's order. */
  readonly objectIds: readonly string[]
} & {
  readonly [K in keyof T]: readonly T[K][];
};
/** Aligned node hydraulic columns; keep the observation time separately. */
export type NodeSnapshot = Snapshot<NodeResults>;
/** Aligned link hydraulic columns; keep the observation time separately. */
export type LinkSnapshot = Snapshot<LinkResults>;
/** Aligned subcatchment runoff columns; keep the observation time separately. */
export type SubcatchmentSnapshot = Snapshot<SubcatchmentResults>;

/** Cumulative routing volumes in project volume units; continuity error is percent. */
export interface RoutingTotals {
  /** Dry-weather inflow volume. */
  readonly dryWeatherInflow: number;
  /** Wet-weather inflow volume. */
  readonly wetWeatherInflow: number;
  /** Groundwater inflow volume. */
  readonly groundwaterInflow: number;
  /** RDII inflow volume. */
  readonly rdiiInflow: number;
  /** External inflow volume. */
  readonly externalInflow: number;
  /** Flooding volume. */
  readonly flooding: number;
  /** Outfall discharge volume. */
  readonly outflow: number;
  /** Evaporation loss volume. */
  readonly evaporationLoss: number;
  /** Seepage loss volume. */
  readonly seepageLoss: number;
  /** Reaction loss in the native routing balance. */
  readonly reactionLoss: number;
  /** Initial system storage volume. */
  readonly initialStorage: number;
  /** Final system storage volume. */
  readonly finalStorage: number;
  /** Routing continuity error in percent. */
  readonly continuityError: number;
}

/** Cumulative runoff water-balance depths in inches or millimetres, except percent continuity error. */
export interface RunoffTotals {
  /** Rainfall depth. */
  readonly rainfall: number;
  /** Evaporation loss depth. */
  readonly evaporationLoss: number;
  /** Infiltration loss depth. */
  readonly infiltrationLoss: number;
  /** Runoff depth. */
  readonly runoff: number;
  /** LID drainage depth. */
  readonly lidDrainage: number;
  /** Outfall runon depth. */
  readonly outfallRunon: number;
  /** Initial surface-storage depth. */
  readonly initialSurfaceStorage: number;
  /** Final surface-storage depth. */
  readonly finalSurfaceStorage: number;
  /** Initial snow-cover depth. */
  readonly initialSnowCover: number;
  /** Final snow-cover depth. */
  readonly finalSnowCover: number;
  /** Removed snow depth. */
  readonly snowRemoved: number;
  /** Runoff continuity error in percent. */
  readonly continuityError: number;
}

/** Detached time-step and convergence diagnostics for the current run. */
export interface RoutingDiagnostics {
  /** Average routing step in seconds. */
  readonly averageTimeStepSeconds: number;
  /** Minimum routing step in seconds. */
  readonly minimumTimeStepSeconds: number;
  /** Maximum routing step in seconds. */
  readonly maximumTimeStepSeconds: number;
  /** Number of routing steps. */
  readonly stepCount: number;
  /** Number of nonconverged steps. */
  readonly nonconvergedStepCount: number;
  /** Nonconverged steps as a percentage of all steps. */
  readonly nonconvergedStepPercentage: number;
  /** Average trial iterations per routing step. */
  readonly averageIterations: number;
}

/** Cumulative system records from `simulation.statistics()` in `running` or `complete`, before ending. */
export interface SimulationStatistics {
  /** Routing-volume continuity balance. */
  readonly routingTotals: RoutingTotals;
  /** Runoff-depth continuity balance. */
  readonly runoffTotals: RunoffTotals;
  /** Groundwater continuity error in percent. */
  readonly groundwaterContinuityError: number;
  /** System quality continuity error in percent. */
  readonly qualityContinuityError: number;
  /** Routing time-step and convergence data. */
  readonly routingDiagnostics: RoutingDiagnostics;
  /** Pollutant-ID balances; seepage follows native report units, including safe log10 units for counts. */
  readonly qualityBalances: Readonly<Record<string, {
    /** Pollutant continuity error in percent. */
    readonly continuityError: number;
    /** Pollutant seepage loss in native report units. */
    readonly seepageLoss: number;
  }>>;
}
