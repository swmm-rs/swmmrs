import type { Call } from "../protocol.js";
import type { LinkResults } from "../snapshots.js";
import type {
  LinkKind,
  OrificeKind,
  WeirKind,
  RoadSurface,
  OutletHeadBasis,
  StandardCrossSectionShape,
} from "../enums.js";

export type {
  OrificeKind,
  WeirKind,
  RoadSurface,
  OutletHeadBasis,
  StandardCrossSectionShape,
} from "../enums.js";

/** Power-law outlet rating in native project head/flow units. */
export interface FunctionalOutletRating {
  /** Power-law rating discriminator. */
  readonly kind: "functional";
  /** Discharge coefficient in the rating's native units. */
  readonly coefficient: number;
  /** Head/depth exponent. */
  readonly exponent: number;
  /** Whether the rating uses depth or head difference. */
  readonly headBasis: OutletHeadBasis;
}
/** Curve-based outlet rating using a configured curve identity. */
export interface TabularOutletRating {
  /** Curve-based rating discriminator. */
  readonly kind: "tabular";
  /** Canonical rating-curve ID. */
  readonly curve: string;
  /** Whether the rating uses depth or head difference. */
  readonly headBasis: OutletHeadBasis;
}
/** Functional or tabular outlet rating, discriminated by `kind`. */
export type OutletRating = FunctionalOutletRating | TabularOutletRating;
/** Replacement outlet rating; omitted head basis defaults to `depth`, not the previous declaration's value. */
export type OutletRatingPatch =
  | {
    /** Power-law rating discriminator. */
    readonly kind: "functional";
    /** {@inheritDoc FunctionalOutletRating.coefficient} */
    readonly coefficient: number;
    /** {@inheritDoc FunctionalOutletRating.exponent} */
    readonly exponent: number;
    /** Depth or head-difference basis; defaults to `depth`. */
    readonly headBasis?: OutletHeadBasis;
  }
  | {
    /** Curve-based rating discriminator. */
    readonly kind: "tabular";
    /** {@inheritDoc TabularOutletRating.curve} */
    readonly curve: string;
    /** Depth or head-difference basis; defaults to `depth`. */
    readonly headBasis?: OutletHeadBasis;
  };

/** Circular conduit section. */
export interface CircularCrossSection {
  /** Circular section discriminator. */
  readonly kind: "circular";
  /** Diameter in project length units (ft or m). */
  readonly diameter: number;
}
/** Standard SWMM section; geometry interpretation depends on the shape. */
export interface StandardCrossSection {
  /** Standard section discriminator. */
  readonly kind: "standard";
  /** Native standard shape name. */
  readonly shape: StandardCrossSectionShape;
  /** Four native geometry parameters; dimensional entries use project length units. */
  readonly geometry: readonly [number, number, number, number];
  /** Native culvert inlet-control code. */
  readonly culvertCode: number;
}
/** Custom section backed by a configured shape. */
export interface CustomCrossSection {
  /** Custom section discriminator. */
  readonly kind: "custom";
  /** Canonical custom-shape ID. */
  readonly shape: string;
  /** Full depth in project length units. */
  readonly fullDepth: number;
  /** Native culvert inlet-control code. */
  readonly culvertCode: number;
}
/** Irregular section backed by a configured transect. */
export interface IrregularCrossSection {
  /** Irregular section discriminator. */
  readonly kind: "irregular";
  /** Canonical transect ID. */
  readonly transect: string;
  /** Native culvert inlet-control code. */
  readonly culvertCode: number;
}
/** Street section backed by a configured street definition. */
export interface StreetCrossSection {
  /** Street section discriminator. */
  readonly kind: "street";
  /** Canonical street ID. */
  readonly street: string;
  /** Native culvert inlet-control code. */
  readonly culvertCode: number;
}
/** Detached conduit section selected by `kind`; absent sections are represented by null on the containing record. */
export type CrossSection =
  | CircularCrossSection
  | StandardCrossSection
  | CustomCrossSection
  | IrregularCrossSection
  | StreetCrossSection;
/** Replacement cross-section declaration. Non-circular variants accept an optional culvert code. */
export type CrossSectionPatch =
  | Omit<CircularCrossSection, "culvertCode">
  | (Omit<StandardCrossSection, "culvertCode"> & {
    /** Native culvert inlet-control code; defaults to zero. */
    readonly culvertCode?: number;
  })
  | (Omit<CustomCrossSection, "culvertCode"> & {
    /** Native culvert inlet-control code; defaults to zero. */
    readonly culvertCode?: number;
  })
  | (Omit<IrregularCrossSection, "culvertCode"> & {
    /** Native culvert inlet-control code; defaults to zero. */
    readonly culvertCode?: number;
  })
  | (Omit<StreetCrossSection, "culvertCode"> & {
    /** Native culvert inlet-control code; defaults to zero. */
    readonly culvertCode?: number;
  });

/** Detached inlet placement; `link.inlet()` returns null when none is configured. */
export interface InletConfiguration {
  /** Number of inlet units. */
  readonly count: number;
  /** Clogging percentage. */
  readonly percentClogged: number;
  /** Inlet flow limit in project flow units. */
  readonly flowLimit: number;
  /** Local depression in project length units. */
  readonly localDepression: number;
  /** Local width in project length units. */
  readonly localWidth: number;
  /** Canonical inlet-design ID. */
  readonly design: string;
}
/** Sparse persistent inlet-control update; omission retains a value. */
export interface InletPatch {
  /** {@inheritDoc InletConfiguration.percentClogged} */
  readonly percentClogged?: number;
  /** {@inheritDoc InletConfiguration.flowLimit} */
  readonly flowLimit?: number;
}
/** Detached current inlet results. */
export interface InletResults {
  /** Dimensionless inlet flow factor. */
  readonly flowFactor: number;
  /** Captured flow in project flow units. */
  readonly capturedFlow: number;
  /** Backflow in project flow units. */
  readonly backflow: number;
  /** Dimensionless backflow ratio. */
  readonly backflowRatio: number;
}

/** Conduit-specific declarations. */
export interface ConduitConfiguration {
  /** Conduit subtype discriminator. */
  readonly kind: "conduit";
  /** Conduit length in ft or m. */
  readonly length: number;
  /** Conduit roughness in native solver units. */
  readonly roughness: number;
  /** Number of parallel barrels. */
  readonly barrels: number;
  /** Cross-section declaration, or null. */
  readonly crossSection: CrossSection | null;
}
/** Pump-specific declarations. */
export interface PumpConfiguration {
  /** Pump subtype discriminator. */
  readonly kind: "pump";
  /** Canonical pump-curve ID, or null for ideal operation. */
  readonly curve: string | null;
  /** Initial dimensionless pump setting. */
  readonly initialSetting: number;
  /** Startup depth in ft or m. */
  readonly startupDepth: number;
  /** Shutoff depth in ft or m. */
  readonly shutoffDepth: number;
}
/** Orifice-specific declarations. */
export interface OrificeConfiguration {
  /** Orifice subtype discriminator. */
  readonly kind: "orifice";
  /** Side or bottom opening. */
  readonly orificeKind: OrificeKind;
  /** Native orifice discharge coefficient. */
  readonly dischargeCoefficient: number;
  /** Full opening time in hours. */
  readonly openingTimeHours: number;
}
/** Weir-specific declarations. */
export interface WeirConfiguration {
  /** Weir subtype discriminator. */
  readonly kind: "weir";
  /** Weir geometry. */
  readonly weirKind: WeirKind;
  /** Native weir discharge coefficient. */
  readonly dischargeCoefficient: number;
  /** Native end-discharge coefficient. */
  readonly endDischargeCoefficient: number;
  /** Number of end contractions. */
  readonly endContractions: number;
  /** Whether the weir may surcharge. */
  readonly canSurcharge: boolean;
  /** Roadway width in ft or m. */
  readonly roadwayWidth: number;
  /** Roadway surface classification. */
  readonly roadwaySurface: RoadSurface;
  /** Canonical coefficient-curve ID, or null. */
  readonly coefficientCurve: string | null;
}
/** Outlet-specific declarations. */
export interface OutletConfiguration {
  /** Outlet subtype discriminator. */
  readonly kind: "outlet";
  /** Functional or tabular rating. */
  readonly rating: OutletRating;
}
/** Kind-tagged link declarations; configuring a subtype cannot change the link's family. */
export type LinkSubtypeConfiguration =
  | ConduitConfiguration
  | PumpConfiguration
  | OrificeConfiguration
  | WeirConfiguration
  | OutletConfiguration;

/** Detached read-only common, subtype, cross-section, and inlet declarations. */
export interface LinkConfiguration {
  /** Configured link family. */
  readonly kind: LinkKind;
  /** Object tag. */
  readonly tag: string;
  /** Canonical configured inlet-node ID. */
  readonly inletNode: string;
  /** Canonical configured outlet-node ID. */
  readonly outletNode: string;
  /** Solver direction relative to configured orientation. */
  readonly flowDirection: number;
  /** Conduit length in ft or m; null for other kinds. */
  readonly length: number | null;
  /** Conduit slope ratio; null for other kinds. */
  readonly slope: number | null;
  /** Full hydraulic depth in ft or m. */
  readonly fullDepth: number;
  /** Full-flow capacity in project flow units. */
  readonly fullFlow: number;
  /** Selection for detailed reporting. */
  readonly includedInReport: boolean;
  /** Inlet offset in project length units. */
  readonly inletOffset: number;
  /** Outlet offset in project length units. */
  readonly outletOffset: number;
  /** Initial flow in project flow units. */
  readonly initialFlow: number;
  /** Persistent flow limit in project flow units; set with {@link Link.setFlowLimit}. */
  readonly flowLimit: number;
  /** Dimensionless inlet loss coefficient. */
  readonly inletLossCoefficient: number;
  /** Dimensionless outlet loss coefficient. */
  readonly outletLossCoefficient: number;
  /** Dimensionless average loss coefficient. */
  readonly averageLossCoefficient: number;
  /** Seepage rate in in/h or mm/h. */
  readonly seepageRate: number;
  /** Flap-gate flag. */
  readonly hasFlapGate: boolean;
  /** Kind-tagged subtype record. */
  readonly subtype: LinkSubtypeConfiguration;
  /** Convenience alias for conduit declarations; null for other link families. */
  readonly crossSection: CrossSection | null;
  /** Inlet placement, or null when absent. */
  readonly inlet: InletConfiguration | null;
}

/** Sparse conduit update; omitted fields retain their values. */
export interface ConduitPatch {
  /** {@inheritDoc ConduitConfiguration.length} */
  readonly length?: number;
  /** {@inheritDoc ConduitConfiguration.roughness} */
  readonly roughness?: number;
  /** {@inheritDoc ConduitConfiguration.barrels} */
  readonly barrels?: number;
}
/** Sparse pump update; omission retains a field. */
export interface PumpPatch {
  /** Omit to preserve; null switches to ideal operation. */
  readonly curve?: string | null;
  /** {@inheritDoc PumpConfiguration.initialSetting} */
  readonly initialSetting?: number;
  /** {@inheritDoc PumpConfiguration.startupDepth} */
  readonly startupDepth?: number;
  /** {@inheritDoc PumpConfiguration.shutoffDepth} */
  readonly shutoffDepth?: number;
}
/** Sparse orifice update; omission retains a field. */
export interface OrificePatch {
  /** {@inheritDoc OrificeConfiguration.orificeKind} */
  readonly orificeKind?: OrificeKind;
  /** {@inheritDoc OrificeConfiguration.dischargeCoefficient} */
  readonly dischargeCoefficient?: number;
  /** {@inheritDoc OrificeConfiguration.openingTimeHours} */
  readonly openingTimeHours?: number;
}
/** Sparse weir update; omission retains a field. */
export interface WeirPatch {
  /** {@inheritDoc WeirConfiguration.weirKind} */
  readonly weirKind?: WeirKind;
  /** {@inheritDoc WeirConfiguration.dischargeCoefficient} */
  readonly dischargeCoefficient?: number;
  /** {@inheritDoc WeirConfiguration.endDischargeCoefficient} */
  readonly endDischargeCoefficient?: number;
  /** {@inheritDoc WeirConfiguration.endContractions} */
  readonly endContractions?: number;
  /** {@inheritDoc WeirConfiguration.canSurcharge} */
  readonly canSurcharge?: boolean;
  /** {@inheritDoc WeirConfiguration.roadwayWidth} */
  readonly roadwayWidth?: number;
  /** {@inheritDoc WeirConfiguration.roadwaySurface} */
  readonly roadwaySurface?: RoadSurface;
  /** Omit to preserve; null clears the optional coefficient curve. */
  readonly coefficientCurve?: string | null;
}
/** Sparse outlet update; omission retains its rating. */
export interface OutletPatch {
  /** Replacement rating declaration. */
  readonly rating?: OutletRatingPatch;
}
/** Select the existing link family and patch only its writable subtype fields. */
export type LinkSubtypePatch =
  | ({
    /** Select an existing conduit. */
    readonly kind: "conduit";
  } & ConduitPatch)
  | ({
    /** Select an existing pump. */
    readonly kind: "pump";
  } & PumpPatch)
  | ({
    /** Select an existing orifice. */
    readonly kind: "orifice";
  } & OrificePatch)
  | ({
    /** Select an existing weir. */
    readonly kind: "weir";
  } & WeirPatch)
  | ({
    /** Select an existing outlet. */
    readonly kind: "outlet";
  } & OutletPatch);

/** Sparse atomic link declaration update. Omitted fields retain their values; derived fields are not writable. */
export interface LinkPatch {
  /** {@inheritDoc LinkConfiguration.tag} */
  readonly tag?: string;
  /** {@inheritDoc LinkConfiguration.inletNode} */
  readonly inletNode?: string;
  /** {@inheritDoc LinkConfiguration.outletNode} */
  readonly outletNode?: string;
  /** {@inheritDoc LinkConfiguration.inletOffset} */
  readonly inletOffset?: number;
  /** {@inheritDoc LinkConfiguration.outletOffset} */
  readonly outletOffset?: number;
  /** {@inheritDoc LinkConfiguration.includedInReport} */
  readonly includedInReport?: boolean;
  /** {@inheritDoc LinkConfiguration.initialFlow} */
  readonly initialFlow?: number;
  /** {@inheritDoc LinkConfiguration.inletLossCoefficient} */
  readonly inletLossCoefficient?: number;
  /** {@inheritDoc LinkConfiguration.outletLossCoefficient} */
  readonly outletLossCoefficient?: number;
  /** {@inheritDoc LinkConfiguration.averageLossCoefficient} */
  readonly averageLossCoefficient?: number;
  /** {@inheritDoc LinkConfiguration.seepageRate} */
  readonly seepageRate?: number;
  /** {@inheritDoc LinkConfiguration.hasFlapGate} */
  readonly hasFlapGate?: boolean;
  /** Replacement conduit cross-section. */
  readonly crossSection?: CrossSectionPatch;
  /** Sparse kind-tagged subtype update; cannot change link family. */
  readonly subtype?: LinkSubtypePatch;
}

/** Detached quality arrays in pollutant-ID order, using configured concentration/load units. */
export interface LinkQuality {
  /** Canonical pollutant IDs defining array order. */
  readonly pollutantIds: readonly string[];
  /** Current link concentrations. */
  readonly concentrations: readonly number[];
  /** Current reactor concentrations. */
  readonly reactorConcentrations: readonly number[];
  /** Cumulative transported pollutant loads. */
  readonly totalLoads: readonly number[];
}
/** Detached pollutant-major quality matrices: `[pollutantIndex][objectIndex]`, in configured units. */
export interface LinkQualitySnapshot {
  /** Canonical link IDs defining inner-row order. */
  readonly objectIds: readonly string[];
  /** Canonical pollutant IDs defining outer-row order. */
  readonly pollutantIds: readonly string[];
  /** Pollutant-major matrices; each inner row follows objectIds. */
  readonly concentrations: readonly (readonly number[])[];
  /** Reactor concentrations in configured concentration units. */
  readonly reactorConcentrations: readonly (readonly number[])[];
  /** Transported loads in configured pollutant load units. */
  readonly totalLoads: readonly (readonly number[])[];
}
/** Detached cumulative link statistics for the current run. */
export interface LinkStatistics {
  /** Maximum flow in project flow units. */
  readonly maximumFlow: number;
  /** Timezone-free ModelTime string of maximum flow. */
  readonly maximumFlowTime: string;
  /** Maximum velocity in ft/s or m/s. */
  readonly maximumVelocity: number;
  /** Maximum depth in ft or m. */
  readonly maximumDepth: number;
  /** Maximum street fill fraction; null without a street section. */
  readonly maximumStreetFillFraction: number | null;
  /** Normal-flow duration in seconds. */
  readonly timeNormalFlowSeconds: number;
  /** Inlet-control duration in seconds. */
  readonly timeInletControlSeconds: number;
  /** Surcharge duration in seconds. */
  readonly timeSurchargedSeconds: number;
  /** Duration full at the upstream end, in seconds. */
  readonly timeFullUpstreamSeconds: number;
  /** Duration full at the downstream end, in seconds. */
  readonly timeFullDownstreamSeconds: number;
  /** Full-flow duration in seconds. */
  readonly timeFullFlowSeconds: number;
  /** Capacity-limited duration in seconds. */
  readonly timeCapacityLimitedSeconds: number;
  /** Seven seconds-valued durations: dry, upstream dry, downstream dry, subcritical, supercritical, upstream critical, downstream critical. */
  readonly timeInFlowClassSeconds: readonly number[];
  /** Courant-critical duration in seconds. */
  readonly timeCourantCriticalSeconds: number;
  /** Flow-reversal count. */
  readonly flowTurns: number;
  /** Flow-reversal sign. */
  readonly flowTurnSign: number;
}
/** Detached pump-only cumulative statistics. */
export interface PumpStatistics {
  /** Pump utilization time in seconds. */
  readonly timeUtilizedSeconds: number;
  /** Minimum pumped flow in project flow units. */
  readonly minimumFlow: number;
  /** Average pumped flow in project flow units. */
  readonly averageFlow: number;
  /** Maximum pumped flow in project flow units. */
  readonly maximumFlow: number;
  /** Pumped volume in project volume units. */
  readonly pumpedVolume: number;
  /** Energy consumed in native pump-report energy units. */
  readonly energyConsumed: number;
  /** Time below the pump curve in seconds. */
  readonly offCurveLowSeconds: number;
  /** Time above the pump curve in seconds. */
  readonly offCurveHighSeconds: number;
  /** Number of pump startups. */
  readonly startupCount: number;
  /** Number of statistical observation periods. */
  readonly periodCount: number;
}
/** Detached cumulative columns in object-ID order; units match {@link LinkStatistics}. */
export interface LinkStatisticsSnapshot {
  /** Canonical link IDs in requested order. */
  readonly objectIds: readonly string[];
  /** {@inheritDoc LinkStatistics.maximumFlow} */
  readonly maximumFlow: readonly number[];
  /** {@inheritDoc LinkStatistics.maximumFlowTime} */
  readonly maximumFlowTime: readonly string[];
  /** {@inheritDoc LinkStatistics.maximumVelocity} */
  readonly maximumVelocity: readonly number[];
  /** {@inheritDoc LinkStatistics.maximumDepth} */
  readonly maximumDepth: readonly number[];
  /** {@inheritDoc LinkStatistics.maximumStreetFillFraction} */
  readonly maximumStreetFillFraction: readonly (number | null)[];
  /** {@inheritDoc LinkStatistics.timeNormalFlowSeconds} */
  readonly timeNormalFlowSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeInletControlSeconds} */
  readonly timeInletControlSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeSurchargedSeconds} */
  readonly timeSurchargedSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeFullUpstreamSeconds} */
  readonly timeFullUpstreamSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeFullDownstreamSeconds} */
  readonly timeFullDownstreamSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeFullFlowSeconds} */
  readonly timeFullFlowSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.timeCapacityLimitedSeconds} */
  readonly timeCapacityLimitedSeconds: readonly number[];
  /** Object-major matrix: one row per link, seven durations in FlowClass order. */
  readonly timeInFlowClassSeconds: readonly (readonly number[])[];
  /** {@inheritDoc LinkStatistics.timeCourantCriticalSeconds} */
  readonly timeCourantCriticalSeconds: readonly number[];
  /** {@inheritDoc LinkStatistics.flowTurns} */
  readonly flowTurns: readonly number[];
  /** {@inheritDoc LinkStatistics.flowTurnSign} */
  readonly flowTurnSign: readonly number[];
}

export interface LinkOperations {
  linkConfiguration: { args: [string]; result: LinkConfiguration };
  configureLink: { args: [string, LinkPatch]; result: void };
  linkQuality: { args: [string]; result: LinkQuality };
  linkQualitySnapshot: { args: [readonly string[] | undefined]; result: LinkQualitySnapshot };
  overrideLinkPollutantConcentrations: { args: [string, Record<string, number>]; result: void };
  linkExternalPollutantMassFlux: { args: [string]; result: Record<string, number> };
  linkExternalPollutantMassFluxValue: { args: [string, string]; result: number };
  updateLinkExternalPollutantMassFlux: { args: [string, Record<string, number>, boolean]; result: void };
  setLinkFlowLimit: { args: [string, number]; result: void };
  linkStatistics: { args: [string]; result: LinkStatistics };
  pumpStatistics: { args: [string]; result: PumpStatistics };
  linkStatisticsSnapshot: { args: [readonly string[] | undefined]; result: LinkStatisticsSnapshot };
  linkInlet: { args: [string]; result: InletConfiguration | null };
  inletResults: { args: [string]; result: InletResults | null };
  updateInlet: { args: [string, InletPatch]; result: void };
}

/** Owner-bound link from `simulation.links.get(id)`. No public subtype subclasses.
 * Reads are detached; configuration is available in healthy states. Wrong-kind
 * operations reject without changing the model. Statistics require an active run.
 */
export class Link {
  /** Canonical configured link ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) { this.id = id; this.#call = call; Object.freeze(this); }
  /** @internal */
  static create(id: string, call: Call): Link { return new Link(id, call); }
  /** Read hydraulics in `running`, `complete`, or `ended`.
   * @returns Detached current link results.
   */
  results(): Promise<LinkResults> { return this.#call("link", this.id); }
  /** Read common and kind-specific declarations in any healthy owner state.
   * @returns Detached configuration, including cross-section and optional inlet.
   */
  configuration(): Promise<LinkConfiguration> { return this.#call("linkConfiguration", this.id); }
  /** Atomically update declarations in `open` or `ended`; accepted writes return `ended` to `open`.
   * @param patch - Sparse common/subtype fields; invalid fields or a wrong-kind subtype reject the whole patch.
   * @returns Resolves after all fields are accepted.
   */
  configure(patch: LinkPatch): Promise<void> { return this.#call("configureLink", this.id, patch); }
  /** Change the target opening or pump speed in `running`; native control rules may change it later.
   * @param setting - Finite dimensionless factor. Non-pumps clamp to [0, 1]; pumps clamp at zero and may exceed one.
   * @returns Resolves after setting the target.
   */
  setTargetSetting(setting: number): Promise<void> { return this.#call("setLinkTargetSetting", this.id, setting); }
  /** Set a persistent flow limit in `open`, `running`, or `ended`.
   * @param flowLimit - Limit in project flow units.
   * @returns Resolves after the forcing is updated.
   */
  setFlowLimit(flowLimit: number): Promise<void> { return this.#call("setLinkFlowLimit", this.id, flowLimit); }
  /** Read quality in `running`, `complete`, or `ended`.
   * @returns Detached pollutant-aligned concentrations and transported loads.
   */
  quality(): Promise<LinkQuality> { return this.#call("linkQuality", this.id); }
  /** Queue sparse overrides for the next quality step in `running`; they expire after that step.
   * @param values - Pollutant-ID concentration map in configured concentration units.
   * @returns Resolves after valid IDs and concentrations are queued.
   */
  overridePollutantConcentrations(values: Record<string, number>): Promise<void> {
    return this.#call("overrideLinkPollutantConcentrations", this.id, values);
  }
  /** Read persistent pollutant mass fluxes in `open`, `running`, or `ended`.
   * @returns Pollutant-ID map in configured mass-flux units.
   */
  externalPollutantMassFlux(): Promise<Record<string, number>> {
    return this.#call("linkExternalPollutantMassFlux", this.id);
  }
  /** Read one persistent mass flux in `open`, `running`, or `ended`.
   * @param pollutantId - Configured pollutant ID.
   * @returns Value in configured pollutant mass-flux units; unknown IDs reject.
   */
  externalPollutantMassFluxValue(pollutantId: string): Promise<number> {
    return this.#call("linkExternalPollutantMassFluxValue", this.id, pollutantId);
  }
  /** Update persistent mass fluxes in `open`, `running`, or `ended`.
   * Nonzero flux requires a non-dummy conduit; other kinds accept only an all-zero clear.
   * @param values - Pollutant-ID map in configured mass-flux units.
   * @param replace - Defaults to false: merge. True replaces the whole map, resetting omitted IDs to zero.
   * @returns Resolves after the atomic update.
   */
  updateExternalPollutantMassFlux(values: Record<string, number>, replace = false): Promise<void> {
    return this.#call("updateLinkExternalPollutantMassFlux", this.id, values, replace);
  }
  /** Read cumulative statistics in `running` or `complete`, before ending the run.
   * @returns Detached link statistics.
   */
  statistics(): Promise<LinkStatistics> { return this.#call("linkStatistics", this.id); }
  /** Read pump-only cumulative statistics in `running` or `complete`.
   * @returns Detached pump statistics; non-pump calls reject.
   */
  pumpStatistics(): Promise<PumpStatistics> { return this.#call("pumpStatistics", this.id); }
  /** Read the inlet placement in any healthy owner state.
   * @returns Detached inlet configuration, or null when absent.
   */
  inlet(): Promise<InletConfiguration | null> { return this.#call("linkInlet", this.id); }
  /** Read inlet results in `running`, `complete`, or `ended`.
   * @returns Detached current inlet values, or null when no inlet is configured.
   */
  inletResults(): Promise<InletResults | null> { return this.#call("inletResults", this.id); }
  /** Update persistent inlet controls in `open`, `running`, or `ended`.
   * @param patch - Sparse clogging percentage and/or flow limit; omission retains a value.
   * @returns Resolves after the update is accepted.
   */
  updateInlet(patch: InletPatch): Promise<void> { return this.#call("updateInlet", this.id, patch); }
}
