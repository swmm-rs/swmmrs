import type { Call } from "../protocol.js";
import type { ModelTime, NodeKind } from "../enums.js";
import type { NodeResults } from "../snapshots.js";

/** Storage area/depth relationship represented by {@link StorageShape}. */
export type StorageShapeKind =
  | "functional"
  | "tabular"
  | "cylindrical"
  | "conical"
  | "paraboloid"
  | "pyramidal";
/** Canonical storage-shape declaration returned by {@link Node.configuration}. */
export interface StorageShape {
  /** Functional, tabular, or geometric shape selector. */
  readonly kind: StorageShapeKind;
  /** Three canonical shape coefficients; interpretation depends on {@link kind}. */
  readonly coefficients: readonly [number, number, number];
  /** Canonical storage-curve ID for a tabular shape; otherwise `null`. */
  readonly curve: string | null;
}

/** Replacement storage shape supplied through {@link NodePatch.shape}. */
export interface StorageShapePatch {
  /** Shape to install; required even when only coefficients change. */
  readonly kind: StorageShapeKind;
  /** Shape coefficients, defaulting to `[0, 0, 0]`. Must be zero for a tabular shape. */
  readonly coefficients?: readonly [number, number, number];
  /** Storage-curve ID, required for a tabular shape. */
  readonly curve?: string | null;
}

/** Storage seepage or Green-Ampt exfiltration parameters. */
export interface StorageExfiltration {
  /** Saturated hydraulic conductivity in project infiltration-rate units (in/h or mm/h). */
  readonly conductivity: number;
  /** Suction head in project rainfall-depth units (in or mm). */
  readonly suctionHead: number;
  /** Initial moisture deficit as a dimensionless fraction. */
  readonly moistureDeficit: number;
}

/** Replacement exfiltration parameters; zero suction and deficit select seepage. */
export interface StorageExfiltrationPatch {
  /** {@inheritDoc StorageExfiltration.conductivity} */
  readonly conductivity: number;
  /** Suction head in in or mm; defaults to zero. */
  readonly suctionHead?: number;
  /** Initial moisture-deficit fraction; defaults to zero. */
  readonly moistureDeficit?: number;
}

/** Declared outfall boundary model; separate from a runtime stage override. */
export type OutfallBoundaryKind = "free" | "normal" | "fixed" | "tidal" | "timeseries";

/** Outfall boundary declaration, not the current runtime override. */
export interface OutfallBoundary {
  /** Boundary model. */
  readonly kind: OutfallBoundaryKind;
  /** Fixed-stage elevation in project length units. */
  readonly stage: number;
  /** Canonical tidal-curve or time-series ID; `null` for other boundary kinds. */
  readonly reference: string | null;
}

/** Replacement outfall boundary supplied through {@link NodePatch.boundary}. */
export interface OutfallBoundaryPatch {
  /** Boundary model to install. */
  readonly kind: OutfallBoundaryKind;
  /** Fixed-stage elevation in project length units; defaults to zero for `fixed`. */
  readonly stage?: number;
  /** Required curve ID for `tidal`, or time-series ID for `timeseries`. */
  readonly reference?: string | null;
}

/** Flow-diversion rule used by a divider node. */
export type DividerRuleKind = "overflow" | "cutoff" | "tabular" | "weir";

/** Canonical divider declaration returned by {@link Node.configuration}. */
export interface DividerRule {
  /** Flow-diversion rule. */
  readonly kind: DividerRuleKind;
  /** Minimum flow, maximum head, and discharge coefficient; applicability depends on the rule. */
  readonly values: readonly [number, number, number];
  /** Canonical diversion-curve ID for `tabular`; otherwise `null`. */
  readonly curve: string | null;
}

/** Replacement divider rule supplied through {@link NodePatch.rule}. */
export interface DividerRulePatch {
  /** Flow-diversion rule to install. */
  readonly kind: DividerRuleKind;
  /** Rule values, defaulting to zeros. `cutoff` accepts only minimum flow; `tabular` requires zeros. */
  readonly values?: readonly [number, number, number];
  /** Diversion-curve ID, required for `tabular`. */
  readonly curve?: string | null;
}

/**
 * Sparse, atomic declaration update accepted by {@link Node.configure}.
 *
 * Omitted top-level fields retain their values. Nested shape, exfiltration,
 * boundary, and rule records replace that declaration rather than merging it.
 * Kind-specific fields require the corresponding node kind. Runtime forcings,
 * results, and the node ID are not accepted here.
 */
export interface NodePatch {
  /** {@inheritDoc NodeConfiguration.tag} */
  readonly tag?: string;
  /** {@inheritDoc NodeConfiguration.invertElevation} */
  readonly invertElevation?: number;
  /** {@inheritDoc NodeConfiguration.fullDepth} */
  readonly fullDepth?: number;
  /** {@inheritDoc NodeConfiguration.surchargeDepth} */
  readonly surchargeDepth?: number;
  /** {@inheritDoc NodeConfiguration.pondedArea} */
  readonly pondedArea?: number;
  /** {@inheritDoc NodeConfiguration.initialDepth} */
  readonly initialDepth?: number;
  /** {@inheritDoc NodeConfiguration.includedInReport} */
  readonly includedInReport?: boolean;
  /** Replacement storage area/depth relationship; storage nodes only. */
  readonly shape?: StorageShapePatch;
  /** Storage evaporation fraction; storage nodes only. */
  readonly evaporationFraction?: number;
  /** Replacement storage exfiltration parameters; `null` disables exfiltration. */
  readonly exfiltration?: StorageExfiltrationPatch | null;
  /** Replacement declared boundary; outfalls only. */
  readonly boundary?: OutfallBoundaryPatch;
  /** Whether reverse flow is blocked; outfalls only. */
  readonly hasFlapGate?: boolean;
  /** Receiving subcatchment ID for an outfall; `null` clears the relation. */
  readonly routeToSubcatchment?: string | null;
  /** Replacement flow-diversion rule; dividers only. */
  readonly rule?: DividerRulePatch;
  /** Diverted link ID for a divider; `null` clears the relation. */
  readonly divertedLink?: string | null;
}

/**
 * Detached, read-only node declarations returned by {@link Node.configuration}.
 * Subtype fields are `null` when they do not apply to this node kind.
 */
export interface NodeConfiguration {
  /** Configured node kind; handles do not have public subtype subclasses. */
  readonly kind: NodeKind;
  /** Object tag. */
  readonly tag: string;
  /** Invert elevation in project length units (ft or m). */
  readonly invertElevation: number;
  /** Full depth above the invert in project length units. */
  readonly fullDepth: number;
  /** Additional surcharge depth in project length units. */
  readonly surchargeDepth: number;
  /** Ponding area in project length squared (ft² or m²). */
  readonly pondedArea: number;
  /** Initial water depth in project length units. */
  readonly initialDepth: number;
  /** Whether this node is selected for detailed reporting. */
  readonly includedInReport: boolean;
  /** Persistent additive API inflow in project flow units; set with {@link Node.setExternalInflow}. */
  readonly externalInflow: number;
  /** Storage area/depth relationship; `null` for other node kinds. */
  readonly shape: StorageShape | null;
  /** Storage evaporation fraction; `null` for other node kinds. */
  readonly evaporationFraction: number | null;
  /** Storage exfiltration parameters; `null` when absent or not applicable. */
  readonly exfiltration: StorageExfiltration | null;
  /** Declared outfall boundary; `null` for other node kinds. */
  readonly boundary: OutfallBoundary | null;
  /** Outfall reverse-flow flap gate; `null` for other node kinds. */
  readonly hasFlapGate: boolean | null;
  /** Canonical receiving-subcatchment ID for an outfall; `null` when unset or not applicable. */
  readonly routeToSubcatchment: string | null;
  /** Divider flow-diversion rule; `null` for other node kinds. */
  readonly rule: DividerRule | null;
  /** Canonical diverted-link ID for a divider; `null` when unset or not applicable. */
  readonly divertedLink: string | null;
}

/** Detached current concentrations, in each pollutant's configured concentration units. */
export interface NodeQuality {
  /** Canonical pollutant IDs defining the order of every concentration array. */
  readonly pollutantIds: readonly string[];
  /** Current node concentrations, aligned with {@link pollutantIds}. */
  readonly concentrations: readonly number[];
  /** Current inflow concentrations, aligned with {@link pollutantIds}. */
  readonly inflowConcentrations: readonly number[];
  /** Current reactor concentrations, aligned with {@link pollutantIds}. */
  readonly reactorConcentrations: readonly number[];
}

/**
 * Detached quality snapshot returned by `simulation.nodes.qualitySnapshot(ids)`.
 * Matrices are pollutant-major: `[pollutantIndex][objectIndex]`.
 * Concentrations use each pollutant's configured concentration units.
 */
export interface NodeQualitySnapshot {
  /** Canonical node IDs in selection order, defining each matrix's inner dimension. */
  readonly objectIds: readonly string[];
  /** Canonical pollutant IDs defining each matrix's outer dimension. */
  readonly pollutantIds: readonly string[];
  /** Current node concentrations. */
  readonly concentrations: readonly (readonly number[])[];
  /** Current inflow concentrations. */
  readonly inflowConcentrations: readonly (readonly number[])[];
  /** Current reactor concentrations. */
  readonly reactorConcentrations: readonly (readonly number[])[];
}

/** Detached cumulative statistics for the current run, read with {@link Node.statistics}. */
export interface NodeStatistics {
  /** Average reported depth in project length units; zero before reporting samples exist. */
  readonly averageDepth: number;
  /** Maximum depth in project length units. */
  readonly maximumDepth: number;
  /** Model calendar time of maximum depth, without a timezone. */
  readonly maximumDepthTime: ModelTime;
  /** Maximum reported depth in project length units. */
  readonly maximumReportedDepth: number;
  /** Cumulative flooded volume in project volume units (ft³ or m³). */
  readonly floodedVolume: number;
  /** Cumulative time flooded, in seconds. */
  readonly timeFloodedSeconds: number;
  /** Cumulative time surcharged, in seconds. */
  readonly timeSurchargedSeconds: number;
  /** Cumulative time Courant-critical, in seconds. */
  readonly timeCourantCriticalSeconds: number;
  /** Cumulative lateral inflow volume in project volume units, not a flow rate. */
  readonly totalLateralInflow: number;
  /** Maximum lateral inflow in project flow units. */
  readonly maximumLateralInflow: number;
  /** Maximum total inflow in project flow units. */
  readonly maximumInflow: number;
  /** Maximum overflow in project flow units. */
  readonly maximumOverflow: number;
  /** Maximum ponded volume in project volume units. */
  readonly maximumPondedVolume: number;
  /** Number of nonconverged routing steps. */
  readonly nonconvergedCount: number;
  /** Model calendar time of maximum inflow, without a timezone. */
  readonly maximumInflowTime: ModelTime;
  /** Model calendar time of maximum overflow, without a timezone. */
  readonly maximumOverflowTime: ModelTime;
}

/** Detached storage-only cumulative statistics, read with {@link Node.storageStatistics}. */
export interface StorageStatistics {
  /** Initial stored volume in project volume units (ft³ or m³). */
  readonly initialVolume: number;
  /** Average reported volume in project volume units; zero before reporting samples exist. */
  readonly averageVolume: number;
  /** Maximum stored volume in project volume units. */
  readonly maximumVolume: number;
  /** Maximum inflow in project flow units. */
  readonly maximumInflow: number;
  /** Cumulative evaporated volume in project volume units. */
  readonly evaporationLosses: number;
  /** Cumulative exfiltrated volume in project volume units. */
  readonly exfiltrationLosses: number;
  /** Model calendar time of maximum volume, without a timezone. */
  readonly maximumVolumeTime: ModelTime;
}

/** Detached outfall-only cumulative statistics, read with {@link Node.outfallStatistics}. */
export interface OutfallStatistics {
  /** Average discharge in project flow units; zero before discharge periods exist. */
  readonly averageFlow: number;
  /** Maximum discharge in project flow units. */
  readonly maximumFlow: number;
  /** Cumulative discharged loads keyed by canonical pollutant ID, in configured load units. */
  readonly pollutantLoads: Readonly<Record<string, number>>;
  /** Number of discharge periods used for the average. */
  readonly periodCount: number;
}
/**
 * Detached cumulative statistics from `simulation.nodes.statisticsSnapshot(ids)`.
 * Every column follows {@link objectIds}; values and units match {@link NodeStatistics}.
 */
export interface NodeStatisticsSnapshot {
  /** Canonical node IDs in selection order. */
  readonly objectIds: readonly string[];
  /** {@inheritDoc NodeStatistics.averageDepth} */
  readonly averageDepth: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumDepth} */
  readonly maximumDepth: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumDepthTime} */
  readonly maximumDepthTime: readonly ModelTime[];
  /** {@inheritDoc NodeStatistics.maximumReportedDepth} */
  readonly maximumReportedDepth: readonly number[];
  /** {@inheritDoc NodeStatistics.floodedVolume} */
  readonly floodedVolume: readonly number[];
  /** {@inheritDoc NodeStatistics.timeFloodedSeconds} */
  readonly timeFloodedSeconds: readonly number[];
  /** {@inheritDoc NodeStatistics.timeSurchargedSeconds} */
  readonly timeSurchargedSeconds: readonly number[];
  /** {@inheritDoc NodeStatistics.timeCourantCriticalSeconds} */
  readonly timeCourantCriticalSeconds: readonly number[];
  /** {@inheritDoc NodeStatistics.totalLateralInflow} */
  readonly totalLateralInflow: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumLateralInflow} */
  readonly maximumLateralInflow: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumInflow} */
  readonly maximumInflow: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumOverflow} */
  readonly maximumOverflow: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumPondedVolume} */
  readonly maximumPondedVolume: readonly number[];
  /** {@inheritDoc NodeStatistics.nonconvergedCount} */
  readonly nonconvergedCount: readonly number[];
  /** {@inheritDoc NodeStatistics.maximumInflowTime} */
  readonly maximumInflowTime: readonly ModelTime[];
  /** {@inheritDoc NodeStatistics.maximumOverflowTime} */
  readonly maximumOverflowTime: readonly ModelTime[];
}

/**
 * Pollutant-ID-to-value mapping. Concentration overrides use configured concentration
 * units; external mass fluxes use configured pollutant mass per project time units.
 * Inputs may be sparse. Returned mappings are detached and read-only.
 */
export type PollutantValues = Readonly<Record<string, number>>;

export interface NodeOperations {
  nodeConfiguration: { args: [string]; result: NodeConfiguration };
  configureNode: { args: [string, NodePatch]; result: void };
  nodeExternalInflow: { args: [string]; result: number };
  outfallFixedStage: { args: [string]; result: number | null };
  nodeQuality: { args: [string]; result: NodeQuality };
  nodeQualitySnapshot: { args: [readonly string[] | undefined]; result: NodeQualitySnapshot };
  overrideNodePollutantConcentrations: { args: [string, PollutantValues]; result: void };
  nodeExternalPollutantMassFlux: { args: [string]; result: PollutantValues };
  nodeExternalPollutantMassFluxValue: { args: [string, string]; result: number };
  updateNodeExternalPollutantMassFlux: { args: [string, PollutantValues, boolean]; result: void };
  nodeStatistics: { args: [string]; result: NodeStatistics };
  storageStatistics: { args: [string]; result: StorageStatistics };
  outfallStatistics: { args: [string]; result: OutfallStatistics };
  nodeStatisticsSnapshot: { args: [readonly string[] | undefined]; result: NodeStatisticsSnapshot };
  nodeTotalInflowVolume: { args: [string]; result: number };
}

/**
 * Owner-bound node handle obtained with `simulation.nodes.get(id)`.
 *
 * The constructor is private. Use {@link NodeConfiguration.kind} to distinguish
 * junctions, storage nodes, outfalls, and dividers; there are no public subtype
 * classes. Reads return detached, frozen records. Mutations require explicit
 * methods, not assignments to those records.
 *
 * Operations reject when the owner is closed or failed, or the requested operation
 * is invalid for its lifecycle state or node kind. Wrong-kind calls do not mutate
 * the model. Read {@link statistics} before ending the run; hydraulic and quality
 * reads remain available in `ended`.
 */
export class Node {
  /** Canonical configured node ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) { this.id = id; this.#call = call; Object.freeze(this); }
  /** @internal */
  static create(id: string, call: Call): Node { return new Node(id, call); }

  /**
   * Read current hydraulics in `running`, `complete`, or `ended`.
   * @returns A detached {@link NodeResults} record in project units, not a time series.
   */
  results(): Promise<NodeResults> { return this.#call("node", this.id); }

  /**
   * Read common and kind-specific declarations in any healthy owner state.
   * @returns A detached {@link NodeConfiguration}; non-applicable subtype fields are `null`.
   */
  configuration(): Promise<NodeConfiguration> { return this.#call("nodeConfiguration", this.id); }

  /**
   * Atomically update declarations in `open` or `ended`.
   * An accepted edit in `ended` returns the owner to `open`.
   * @param patch - Sparse {@link NodePatch}. Omitted top-level fields retain their values.
   * @returns Resolves after the complete patch is accepted.
   * @throws Rejects for an invalid patch, wrong node kind, or invalid lifecycle state;
   * no part of a rejected patch is applied.
   * @example
   * ```typescript
   * import type { Node } from "@swmmrs/swmmrs";
   * declare const node: Node;
   * await node.configure({ fullDepth: 4.5, initialDepth: 0.25 });
   * ```
   */
  configure(patch: NodePatch): Promise<void> { return this.#call("configureNode", this.id, patch); }

  /**
   * Read the persistent additive API inflow in any healthy owner state.
   * @returns Inflow in project flow units, excluding other inflow sources.
   */
  externalInflow(): Promise<number> { return this.#call("nodeExternalInflow", this.id); }

  /**
   * Set persistent additive API inflow in `open`, `running`, or `ended`.
   * @param flow - Inflow in project flow units. Zero removes the API contribution.
   * @returns Resolves after the forcing is updated.
   */
  setExternalInflow(flow: number): Promise<void> { return this.#call("setNodeExternalInflow", this.id, flow); }

  /**
   * Set a persistent fixed-stage override in `open`, `running`, or `ended`.
   * This runtime forcing is separate from {@link NodePatch.boundary}.
   * @param stage - Outfall boundary elevation in project length units (ft or m).
   * @returns Resolves after the forcing is updated.
   * @throws Rejects for a non-outfall node or an invalid stage or lifecycle state.
   */
  setOutfallStage(stage: number): Promise<void> { return this.#call("setOutfallStage", this.id, stage); }

  /**
   * Read the effective fixed stage in any healthy owner state; outfalls only.
   * @returns Runtime override if set, otherwise the declared fixed stage, in project
   * length units; `null` when neither supplies a fixed stage.
   * @throws Rejects for a non-outfall node.
   */
  fixedStage(): Promise<number | null> { return this.#call("outfallFixedStage", this.id); }

  /**
   * Read current pollutant concentrations in `running`, `complete`, or `ended`.
   * @returns Detached {@link NodeQuality} arrays aligned with their pollutant IDs.
   */
  quality(): Promise<NodeQuality> { return this.#call("nodeQuality", this.id); }

  /**
   * Queue concentration overrides for the next quality-routing step; requires `running`.
   * The override expires after that step rather than becoming a persistent forcing.
   * @param values - Sparse pollutant-ID map of nonnegative concentrations in each
   * pollutant's configured concentration units.
   * @returns Resolves after the overrides are queued.
   * @throws Rejects invalid IDs, concentrations, or lifecycle states.
   */
  overridePollutantConcentrations(values: PollutantValues): Promise<void> {
    return this.#call("overrideNodePollutantConcentrations", this.id, values);
  }

  /**
   * Read persistent external pollutant mass fluxes in `open`, `running`, or `ended`.
   * @returns A detached {@link PollutantValues} map in configured pollutant mass-flux units.
   */
  externalPollutantMassFlux(): Promise<PollutantValues> {
    return this.#call("nodeExternalPollutantMassFlux", this.id);
  }

  /**
   * Read one persistent external mass flux in `open`, `running`, or `ended`.
   * @param pollutantId - Configured pollutant ID.
   * @returns The pollutant's external mass flux in its configured mass-flux units.
   * @throws Rejects an unknown pollutant ID or invalid lifecycle state.
   */
  externalPollutantMassFluxValue(pollutantId: string): Promise<number> {
    return this.#call("nodeExternalPollutantMassFluxValue", this.id, pollutantId);
  }

  /**
   * Update persistent external pollutant mass fluxes in `open`, `running`, or `ended`.
   * @param values - Sparse pollutant-ID map in configured pollutant mass-flux units.
   * @param replace - Defaults to `false`: merge supplied IDs. With `true`, replace
   * the complete mapping and reset omitted pollutants to zero.
   * @returns Resolves after the atomic update.
   * @throws Rejects invalid pollutant IDs, values, or lifecycle states.
   */
  updateExternalPollutantMassFlux(values: PollutantValues, replace = false): Promise<void> {
    return this.#call("updateNodeExternalPollutantMassFlux", this.id, values, replace);
  }

  /**
   * Reset all persistent external pollutant mass fluxes to zero.
   * Requires `open`, `running`, or `ended`; equivalent to
   * `updateExternalPollutantMassFlux({}, true)`.
   * @returns Resolves after the complete mapping is cleared.
   */
  clearExternalPollutantMassFlux(): Promise<void> {
    return this.updateExternalPollutantMassFlux({}, true);
  }

  /**
   * Read cumulative node statistics in `running` or `complete`, before ending the run.
   * @returns A detached {@link NodeStatistics} record for the current run.
   */
  statistics(): Promise<NodeStatistics> { return this.#call("nodeStatistics", this.id); }

  /**
   * Read storage-only statistics in `running` or `complete`.
   * @returns A detached {@link StorageStatistics} record for the current run.
   * @throws Rejects for a non-storage node or invalid lifecycle state.
   */
  storageStatistics(): Promise<StorageStatistics> { return this.#call("storageStatistics", this.id); }

  /**
   * Read outfall-only statistics in `running` or `complete`.
   * @returns A detached {@link OutfallStatistics} record for the current run.
   * @throws Rejects for a non-outfall node or invalid lifecycle state.
   */
  outfallStatistics(): Promise<OutfallStatistics> { return this.#call("outfallStatistics", this.id); }

  /**
   * Read cumulative total inflow volume in `running` or `complete`.
   * @returns Volume in project volume units (ft³ or m³), not a flow rate.
   */
  totalInflowVolume(): Promise<number> { return this.#call("nodeTotalInflowVolume", this.id); }
}
