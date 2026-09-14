import type { Call } from "../protocol.js";
import type { InfilKind, OutKind } from "../enums.js";
import type { RainGageResults, SubcatchmentResults } from "../snapshots.js";
import { LidUnitCollection, type SubcatchmentLidSnapshot } from "./lids.js";

export type { InfilKind, OutKind } from "../enums.js";

/** Configured runoff destination. */
export interface SubcatchmentOutlet {
  /** Receiving node or subcatchment family. */
  readonly kind: OutKind;
  /** Canonical configured receiving ID. */
  readonly id: string;
}

/** Horton infiltration declarations. */
export interface HortonInfiltrationSettings {
  /** Infiltration-model discriminator. */
  readonly kind: "horton";
  /** Initial infiltration rate in in/h or mm/h. */
  readonly initialRate: number;
  /** Minimum infiltration rate in in/h or mm/h. */
  readonly minimumRate: number;
  /** Decay coefficient in inverse hours. */
  readonly decayCoefficient: number;
  /** Time to dry soil fully, in days. */
  readonly dryingTimeDays: number;
  /** Maximum cumulative infiltration in in or mm; null means no limit. */
  readonly maximumInfiltration: number | null;
}
/** Modified Horton infiltration declarations. */
export interface ModifiedHortonInfiltrationSettings {
  /** Infiltration-model discriminator. */
  readonly kind: "modified_horton";
  /** {@inheritDoc HortonInfiltrationSettings.initialRate} */
  readonly initialRate: number;
  /** {@inheritDoc HortonInfiltrationSettings.minimumRate} */
  readonly minimumRate: number;
  /** {@inheritDoc HortonInfiltrationSettings.decayCoefficient} */
  readonly decayCoefficient: number;
  /** {@inheritDoc HortonInfiltrationSettings.dryingTimeDays} */
  readonly dryingTimeDays: number;
  /** {@inheritDoc HortonInfiltrationSettings.maximumInfiltration} */
  readonly maximumInfiltration: number | null;
}
/** Green-Ampt infiltration declarations. */
export interface GreenAmptInfiltrationSettings {
  /** Infiltration-model discriminator. */
  readonly kind: "green_ampt";
  /** Wetting-front suction head in in or mm. */
  readonly suctionHead: number;
  /** Saturated hydraulic conductivity in in/h or mm/h. */
  readonly hydraulicConductivity: number;
  /** Initial moisture deficit fraction. */
  readonly initialMoistureDeficit: number;
}
/** Modified Green-Ampt infiltration declarations. */
export interface ModifiedGreenAmptInfiltrationSettings {
  /** Infiltration-model discriminator. */
  readonly kind: "modified_green_ampt";
  /** {@inheritDoc GreenAmptInfiltrationSettings.suctionHead} */
  readonly suctionHead: number;
  /** {@inheritDoc GreenAmptInfiltrationSettings.hydraulicConductivity} */
  readonly hydraulicConductivity: number;
  /** {@inheritDoc GreenAmptInfiltrationSettings.initialMoistureDeficit} */
  readonly initialMoistureDeficit: number;
}
/** Curve-number infiltration declarations. */
export interface CurveNumberInfiltrationSettings {
  /** Infiltration-model discriminator. */
  readonly kind: "curve_number";
  /** Dimensionless runoff curve number. */
  readonly curveNumber: number;
  /** Time to dry soil fully, in days. */
  readonly dryingTimeDays: number;
}

/** Detached infiltration declarations, narrowed by `kind`. */
export type SubcatchmentInfiltrationConfiguration =
  | HortonInfiltrationSettings
  | ModifiedHortonInfiltrationSettings
  | GreenAmptInfiltrationSettings
  | ModifiedGreenAmptInfiltrationSettings
  | CurveNumberInfiltrationSettings;

/** Sparse infiltration update. Supply fields appropriate to the selected model; incompatible fields are rejected. */
export interface SubcatchmentInfiltrationPatch {
  /** Model selector; omission retains the configured model. */
  readonly kind?: InfilKind;
  /** {@inheritDoc HortonInfiltrationSettings.initialRate} */
  readonly initialRate?: number;
  /** {@inheritDoc HortonInfiltrationSettings.minimumRate} */
  readonly minimumRate?: number;
  /** {@inheritDoc HortonInfiltrationSettings.decayCoefficient} */
  readonly decayCoefficient?: number;
  /** {@inheritDoc HortonInfiltrationSettings.dryingTimeDays} */
  readonly dryingTimeDays?: number;
  /** {@inheritDoc HortonInfiltrationSettings.maximumInfiltration} */
  readonly maximumInfiltration?: number | null;
  /** {@inheritDoc GreenAmptInfiltrationSettings.suctionHead} */
  readonly suctionHead?: number;
  /** {@inheritDoc GreenAmptInfiltrationSettings.hydraulicConductivity} */
  readonly hydraulicConductivity?: number;
  /** {@inheritDoc GreenAmptInfiltrationSettings.initialMoistureDeficit} */
  readonly initialMoistureDeficit?: number;
  /** {@inheritDoc CurveNumberInfiltrationSettings.curveNumber} */
  readonly curveNumber?: number;
}

/** Detached groundwater declarations in project units. */
export interface SubcatchmentGroundwaterConfiguration {
  /** Canonical configured aquifer ID. */
  readonly aquifer: string;
  /** Canonical receiving node ID. */
  readonly node: string;
  /** Surface elevation in ft or m. */
  readonly surfaceElevation: number;
  /** Groundwater-flow equation coefficient in project units. */
  readonly groundwaterCoefficient: number;
  /** Groundwater-flow equation exponent. */
  readonly groundwaterExponent: number;
  /** Surface-water equation coefficient in project units. */
  readonly surfaceCoefficient: number;
  /** Surface-water equation exponent. */
  readonly surfaceExponent: number;
  /** Groundwater/surface-water interaction coefficient in project units. */
  readonly interactionCoefficient: number;
  /** Fixed receiving-node surface depth in ft or m. */
  readonly fixedSurfaceDepth: number;
  /** Aquifer bottom elevation in ft or m. */
  readonly bottomElevation: number;
  /** Initial water-table elevation in ft or m. */
  readonly waterTableElevation: number;
  /** Initial upper-zone moisture fraction. */
  readonly upperMoisture: number;
}
/** Sparse groundwater update; omission retains a field in an existing declaration. */
export interface SubcatchmentGroundwaterPatch {
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.aquifer} */
  readonly aquifer?: string;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.node} */
  readonly node?: string;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.surfaceElevation} */
  readonly surfaceElevation?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.groundwaterCoefficient} */
  readonly groundwaterCoefficient?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.groundwaterExponent} */
  readonly groundwaterExponent?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.surfaceCoefficient} */
  readonly surfaceCoefficient?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.surfaceExponent} */
  readonly surfaceExponent?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.interactionCoefficient} */
  readonly interactionCoefficient?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.fixedSurfaceDepth} */
  readonly fixedSurfaceDepth?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.bottomElevation} */
  readonly bottomElevation?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.waterTableElevation} */
  readonly waterTableElevation?: number;
  /** {@inheritDoc SubcatchmentGroundwaterConfiguration.upperMoisture} */
  readonly upperMoisture?: number;
}

/** Atomic sparse configuration update; omitted fields retain their declarations. */
export interface SubcatchmentPatch {
  /** {@inheritDoc SubcatchmentConfiguration.tag} */
  readonly tag?: string;
  /** {@inheritDoc SubcatchmentConfiguration.area} */
  readonly area?: number;
  /** {@inheritDoc SubcatchmentConfiguration.imperviousFraction} */
  readonly imperviousFraction?: number;
  /** {@inheritDoc SubcatchmentConfiguration.zeroImperviousFraction} */
  readonly zeroImperviousFraction?: number;
  /** {@inheritDoc SubcatchmentConfiguration.width} */
  readonly width?: number;
  /** {@inheritDoc SubcatchmentConfiguration.slope} */
  readonly slope?: number;
  /** {@inheritDoc SubcatchmentConfiguration.imperviousRoughness} */
  readonly imperviousRoughness?: number;
  /** {@inheritDoc SubcatchmentConfiguration.perviousRoughness} */
  readonly perviousRoughness?: number;
  /** {@inheritDoc SubcatchmentConfiguration.imperviousDepressionStorage} */
  readonly imperviousDepressionStorage?: number;
  /** {@inheritDoc SubcatchmentConfiguration.perviousDepressionStorage} */
  readonly perviousDepressionStorage?: number;
  /** {@inheritDoc SubcatchmentConfiguration.curbLength} */
  readonly curbLength?: number;
  /** {@inheritDoc SubcatchmentConfiguration.includedInReport} */
  readonly includedInReport?: boolean;
  /** {@inheritDoc SubcatchmentConfiguration.rainGage} */
  readonly rainGage?: string;
  /** {@inheritDoc SubcatchmentConfiguration.outlet} */
  readonly outlet?: SubcatchmentOutlet;
  /** Sparse infiltration parameters matching the selected model. */
  readonly infiltration?: SubcatchmentInfiltrationPatch;
  /** Omit to preserve; null removes the groundwater declaration. */
  readonly groundwater?: SubcatchmentGroundwaterPatch | null;
  /** Omit to preserve; null removes the snowpack relationship. */
  readonly snowpack?: string | null;
  /** Replace the complete configured pollutant buildup map. */
  readonly initialBuildup?: Readonly<Record<string, number>>;
  /** Replace the complete configured land-use coverage map. */
  readonly coverageFractions?: Readonly<Record<string, number>>;
}

/** Detached subcatchment declarations; runtime precipitation scales are also exposed for inspection. */
export interface SubcatchmentConfiguration {
  /** User tag; an empty string clears it. */
  readonly tag: string;
  /** Dimensionless rainfall multiplier; edited through `setPrecipitationScaleFactors`. */
  readonly rainScaleFactor: number;
  /** Dimensionless snowfall multiplier; edited through `setPrecipitationScaleFactors`. */
  readonly snowScaleFactor: number;
  /** Drainage area in acres for US projects or hectares for SI projects. */
  readonly area: number;
  /** Canonical configured rain-gage ID. */
  readonly rainGage: string;
  /** Whether this subcatchment is selected for reporting. */
  readonly includedInReport: boolean;
  /** Characteristic runoff width in ft or m. */
  readonly width: number;
  /** Surface slope as a fraction, not percent. */
  readonly slope: number;
  /** Curb length in ft or m. */
  readonly curbLength: number;
  /** Impervious area fraction, 0–1. */
  readonly imperviousFraction: number;
  /** Fraction of impervious area without depression storage, 0–1. */
  readonly zeroImperviousFraction: number;
  /** Impervious Manning roughness coefficient. */
  readonly imperviousRoughness: number;
  /** Pervious Manning roughness coefficient. */
  readonly perviousRoughness: number;
  /** Impervious depression storage in in or mm. */
  readonly imperviousDepressionStorage: number;
  /** Pervious depression storage in in or mm. */
  readonly perviousDepressionStorage: number;
  /** Canonical runoff destination. */
  readonly outlet: SubcatchmentOutlet;
  /** Model-specific infiltration declarations, or null when absent. */
  readonly infiltration: SubcatchmentInfiltrationConfiguration | null;
  /** Groundwater declarations, or null when absent. */
  readonly groundwater: SubcatchmentGroundwaterConfiguration | null;
  /** Canonical snowmelt parameter-set ID, or null. */
  readonly snowpack: string | null;
  /** Configured pollutant-ID map of initial buildup in pollutant load units. */
  readonly initialBuildup: Readonly<Record<string, number>>;
  /** Configured land-use-ID map of area fractions, 0–1. */
  readonly coverageFractions: Readonly<Record<string, number>>;
}

/** Detached pollutant results; all arrays follow `pollutantIds`. */
export interface SubcatchmentQuality {
  /** Canonical pollutant IDs in configured order. */
  readonly pollutantIds: readonly string[];
  /** Runoff concentrations in each pollutant's configured concentration units. */
  readonly runoffConcentrations: readonly number[];
  /** Ponded-water concentrations in configured concentration units. */
  readonly pondedConcentrations: readonly number[];
  /** Current buildup in each pollutant's load units. */
  readonly buildupLoads: readonly number[];
  /** Cumulative washoff in each pollutant's load units. */
  readonly totalWashoffLoads: readonly number[];
}
/** Detached pollutant-major matrices: outer rows follow `pollutantIds`, inner columns follow `objectIds`. */
export interface SubcatchmentQualitySnapshot {
  /** Selected canonical subcatchment IDs in requested order. */
  readonly objectIds: readonly string[];
  /** Canonical pollutant IDs in configured order. */
  readonly pollutantIds: readonly string[];
  /** Runoff concentrations in configured pollutant concentration units. */
  readonly runoffConcentrations: readonly (readonly number[])[];
  /** Ponded concentrations in configured pollutant concentration units. */
  readonly pondedConcentrations: readonly (readonly number[])[];
  /** Current buildup in configured pollutant load units. */
  readonly buildupLoads: readonly (readonly number[])[];
  /** Cumulative washoff in configured pollutant load units. */
  readonly totalWashoffLoads: readonly (readonly number[])[];
}

/** Detached cumulative runoff statistics, available before `end()`. */
export interface SubcatchmentStatistics {
  /** Total precipitation depth in in or mm. */
  readonly precipitation: number;
  /** Total runon volume in ft³ or m³. */
  readonly runonVolume: number;
  /** Total evaporated volume in ft³ or m³. */
  readonly evaporationVolume: number;
  /** Total infiltrated volume in ft³ or m³. */
  readonly infiltrationVolume: number;
  /** Total runoff volume in ft³ or m³. */
  readonly runoffVolume: number;
  /** Maximum runoff rate in project flow units. */
  readonly maximumRunoff: number;
  /** Impervious runoff volume in ft³ or m³. */
  readonly imperviousRunoffVolume: number;
  /** Pervious runoff volume in ft³ or m³. */
  readonly perviousRunoffVolume: number;
}
/** Detached statistics arrays, all aligned to `objectIds`. */
export interface SubcatchmentStatisticsSnapshot {
  /** Selected canonical subcatchment IDs in requested order. */
  readonly objectIds: readonly string[];
  /** {@inheritDoc SubcatchmentStatistics.precipitation} */
  readonly precipitation: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.runonVolume} */
  readonly runonVolume: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.evaporationVolume} */
  readonly evaporationVolume: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.infiltrationVolume} */
  readonly infiltrationVolume: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.runoffVolume} */
  readonly runoffVolume: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.maximumRunoff} */
  readonly maximumRunoff: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.imperviousRunoffVolume} */
  readonly imperviousRunoffVolume: readonly number[];
  /** {@inheritDoc SubcatchmentStatistics.perviousRunoffVolume} */
  readonly perviousRunoffVolume: readonly number[];
}

/** Persistent dimensionless precipitation multipliers; both initially default to 1. */
export interface SubcatchmentPrecipitationScaleFactors {
  /** Rainfall multiplier. */
  readonly rainfall: number;
  /** Snowfall multiplier. */
  readonly snowfall: number;
}
/** Persistent external precipitation additions in in/h or mm/h; initially zero. */
export interface SubcatchmentExternalForcing {
  /** External rainfall rate. */
  readonly rainfall: number;
  /** External snowfall rate. */
  readonly snowfall: number;
}
/** Select initial pollutant buildup (`loading`) or land-use area fractions (`coverage`). */
export type SubcatchmentNamedSettingsKind = "loading" | "coverage";
/** Canonical pollutant-ID/load or land-use-ID/fraction map, as selected by the operation. */
export type SubcatchmentNamedValues = Readonly<Record<string, number>>;
/** Canonical pollutant-ID map; values use the operation's concentration or load units. */
export type SubcatchmentPollutantValues = Readonly<Record<string, number>>;

export interface SubcatchmentOperations {
  subcatchmentConfiguration: { args: [string]; result: SubcatchmentConfiguration };
  configureSubcatchment: { args: [string, SubcatchmentPatch]; result: void };
  useRainGageExternalPrecipitation: { args: [string, number]; result: void };
  rainGageExternalPrecipitationRate: { args: [string]; result: number | null };
  rainGageRainfallOverride: { args: [string]; result: number | null };
  subcatchmentPrecipitationScaleFactors: {
    args: [string];
    result: SubcatchmentPrecipitationScaleFactors;
  };
  setSubcatchmentPrecipitationScaleFactors: { args: [string, number, number]; result: void };
  subcatchmentExternalForcing: { args: [string]; result: SubcatchmentExternalForcing };
  setSubcatchmentExternalRainfall: { args: [string, number]; result: void };
  setSubcatchmentExternalSnowfall: { args: [string, number]; result: void };
  subcatchmentQuality: { args: [string]; result: SubcatchmentQuality };
  subcatchmentQualitySnapshot: {
    args: [readonly string[] | undefined];
    result: SubcatchmentQualitySnapshot;
  };
  subcatchmentNamedSettings: {
    args: [string, SubcatchmentNamedSettingsKind];
    result: SubcatchmentNamedValues;
  };
  subcatchmentNamedSetting: {
    args: [string, SubcatchmentNamedSettingsKind, string];
    result: number;
  };
  updateSubcatchmentNamedSettings: {
    args: [string, SubcatchmentNamedSettingsKind, SubcatchmentNamedValues, boolean];
    result: void;
  };
  subcatchmentExternalPollutantBuildupIncrement: {
    args: [string];
    result: SubcatchmentPollutantValues;
  };
  subcatchmentExternalPollutantBuildupIncrementValue: {
    args: [string, string];
    result: number;
  };
  updateSubcatchmentExternalPollutantBuildupIncrement: {
    args: [string, SubcatchmentPollutantValues, boolean];
    result: void;
  };
  subcatchmentStatistics: { args: [string]; result: SubcatchmentStatistics };
  subcatchmentStatisticsSnapshot: {
    args: [readonly string[] | undefined];
    result: SubcatchmentStatisticsSnapshot;
  };
}

function qualityMapping(
  quality: SubcatchmentQuality,
  field: "runoffConcentrations" | "pondedConcentrations" | "buildupLoads" | "totalWashoffLoads",
): SubcatchmentPollutantValues {
  const values: Record<string, number> = {};
  for (const [index, pollutantId] of quality.pollutantIds.entries()) {
    const value = quality[field][index];
    if (value !== undefined) values[pollutantId] = value;
  }
  return values;
}

/** Subcatchment handle owned by one Simulation. Worker reads return detached values; use `configure` only in `open` or `ended`. */
export class Subcatchment {
  /** Canonical configured subcatchment ID. */
  readonly id: string;
  /** Asynchronously indexed LID usages belonging to this subcatchment. */
  readonly lidUnits: LidUnitCollection;
  readonly #call: Call;
  private constructor(id: string, call: Call) {
    this.id = id;
    this.#call = call;
    this.lidUnits = LidUnitCollection.create(id, call);
    Object.freeze(this);
  }
  /** @internal */
  static create(id: string, call: Call): Subcatchment { return new Subcatchment(id, call); }
  /** Read current runoff results in `running`, `complete`, or `ended`.
   * @returns Detached result record in project units.
   */
  results(): Promise<SubcatchmentResults> { return this.#call("subcatchment", this.id); }
  /** Read aggregate LID-group results in `running` or `complete`, before `end()`.
   * @returns Detached aggregate LID results.
   * @throws A solver error when this subcatchment has no LID group.
   */
  lidSnapshot(): Promise<SubcatchmentLidSnapshot> { return this.#call("subcatchmentLidSnapshot", this.id); }
  /** Read declarations in any healthy owner state.
   * @returns Detached configuration with canonical relationship IDs.
   */
  configuration(): Promise<SubcatchmentConfiguration> { return this.#call("subcatchmentConfiguration", this.id); }
  /** Atomically update declarations in `open` or `ended`; preparation-sensitive edits return the owner to `open`.
   * @param patch - Sparse project-unit declarations. Null removes optional relationships; supplied buildup/coverage maps replace those complete maps.
   * @returns Resolves after all supplied fields are accepted; rejected patches apply no fields.
   */
  configure(patch: SubcatchmentPatch): Promise<void> { return this.#call("configureSubcatchment", this.id, patch); }
  /** Read persistent precipitation multipliers in a healthy owner state.
   * @returns Dimensionless rainfall and snowfall factors.
   */
  precipitationScaleFactors(): Promise<SubcatchmentPrecipitationScaleFactors> {
    return this.#call("subcatchmentPrecipitationScaleFactors", this.id);
  }
  /** Read the persistent dimensionless rainfall multiplier; initially 1. */
  rainScaleFactor(): Promise<number> {
    return this.precipitationScaleFactors().then(({ rainfall }) => rainfall);
  }
  /** Read the persistent dimensionless snowfall multiplier; initially 1. */
  snowScaleFactor(): Promise<number> {
    return this.precipitationScaleFactors().then(({ snowfall }) => snowfall);
  }
  /** Replace persistent precipitation multipliers in `open`, `running`, or `ended`.
   * @param rainfall - Finite nonnegative rainfall multiplier.
   * @param snowfall - Finite nonnegative snowfall multiplier.
   * @returns Resolves after both values are accepted; values persist until replaced.
   */
  setPrecipitationScaleFactors(rainfall: number, snowfall: number): Promise<void> {
    return this.#call("setSubcatchmentPrecipitationScaleFactors", this.id, rainfall, snowfall);
  }
  /** Read persistent external rainfall/snowfall additions in a healthy owner state.
   * @returns Rates in in/h or mm/h, initially zero.
   */
  externalForcing(): Promise<SubcatchmentExternalForcing> {
    return this.#call("subcatchmentExternalForcing", this.id);
  }
  /** Read the persistent external rainfall addition in in/h or mm/h. */
  externalRainfall(): Promise<number> {
    return this.externalForcing().then(({ rainfall }) => rainfall);
  }
  /** Replace the persistent rainfall addition in `open`, `running`, or `ended`.
   * @param value - Finite nonnegative rate in in/h or mm/h; zero clears the addition.
   */
  setExternalRainfall(value: number): Promise<void> {
    return this.#call("setSubcatchmentExternalRainfall", this.id, value);
  }
  /** Read the persistent external snowfall addition in in/h or mm/h. */
  externalSnowfall(): Promise<number> {
    return this.externalForcing().then(({ snowfall }) => snowfall);
  }
  /** Replace the persistent snowfall addition in `open`, `running`, or `ended`.
   * @param value - Finite nonnegative rate in in/h or mm/h; zero clears the addition.
   */
  setExternalSnowfall(value: number): Promise<void> {
    return this.#call("setSubcatchmentExternalSnowfall", this.id, value);
  }
  /** Read pollutant-aligned quality results in `running`, `complete`, or `ended`.
   * @returns Detached concentrations and loads in configured pollutant units.
   */
  quality(): Promise<SubcatchmentQuality> { return this.#call("subcatchmentQuality", this.id); }
  /** Read runoff concentrations keyed by pollutant ID in `running`, `complete`, or `ended`; values use configured concentration units. */
  runoffPollutantConcentration(): Promise<SubcatchmentPollutantValues> {
    return this.quality().then((quality) => qualityMapping(quality, "runoffConcentrations"));
  }
  /** Read ponded-water concentrations keyed by pollutant ID in `running`, `complete`, or `ended`; values use configured concentration units. */
  pondedPollutantConcentration(): Promise<SubcatchmentPollutantValues> {
    return this.quality().then((quality) => qualityMapping(quality, "pondedConcentrations"));
  }
  /** Read current buildup keyed by pollutant ID in `running`, `complete`, or `ended`; values use configured pollutant load units. */
  pollutantBuildup(): Promise<SubcatchmentPollutantValues> {
    return this.quality().then((quality) => qualityMapping(quality, "buildupLoads"));
  }
  /** Read cumulative washoff keyed by pollutant ID in `running`, `complete`, or `ended`; values use configured pollutant load units. */
  pollutantTotalLoad(): Promise<SubcatchmentPollutantValues> {
    return this.quality().then((quality) => qualityMapping(quality, "totalWashoffLoads"));
  }
  /** Read configured loading or coverage in a healthy owner state.
   * @param kind - `loading` for pollutant buildup; `coverage` for land-use fractions.
   * @returns Detached canonical-ID map in pollutant load units or area fractions, respectively.
   */
  namedSettings(kind: SubcatchmentNamedSettingsKind): Promise<SubcatchmentNamedValues> {
    return this.#call("subcatchmentNamedSettings", this.id, kind);
  }
  /** Read one named declaration in a healthy owner state.
   * @param kind - Loading or coverage selector.
   * @param name - Configured pollutant ID or land-use ID, respectively.
   * @returns Initial load or coverage fraction for the selected definition.
   */
  namedSetting(kind: SubcatchmentNamedSettingsKind, name: string): Promise<number> {
    return this.#call("subcatchmentNamedSetting", this.id, kind, name);
  }
  /** Atomically update loading or coverage in `open` or `ended`; accepted declaration edits return the owner to `open`.
   * @param kind - Loading or coverage selector.
   * @param values - Configured pollutant-ID/load or land-use-ID/fraction map.
   * @param replace - False by default merges supplied entries; true clears omitted entries to zero.
   * @returns Resolves after the update is accepted.
   */
  updateNamedSettings(
    kind: SubcatchmentNamedSettingsKind,
    values: SubcatchmentNamedValues,
    replace = false,
  ): Promise<void> {
    return this.#call("updateSubcatchmentNamedSettings", this.id, kind, values, replace);
  }
  /** Clear every entry in one named declaration family in `open` or `ended`.
   * @param kind - Loading or coverage selector.
   */
  clearNamedSettings(kind: SubcatchmentNamedSettingsKind): Promise<void> {
    return this.updateNamedSettings(kind, {}, true);
  }
  /** Read configured initial buildup by pollutant ID in a healthy owner state; values use pollutant load units. */
  initialBuildup(): Promise<SubcatchmentNamedValues> { return this.namedSettings("loading"); }
  /** Update configured initial buildup in `open` or `ended`.
   * @param values - Canonical pollutant-ID map in configured load units.
   * @param replace - False by default merges; true clears omitted pollutants to zero.
   */
  updateInitialBuildup(values: SubcatchmentNamedValues, replace = false): Promise<void> {
    return this.updateNamedSettings("loading", values, replace);
  }
  /** Clear configured initial buildup in `open` or `ended`. */
  clearInitialBuildup(): Promise<void> { return this.clearNamedSettings("loading"); }
  /** Read configured land-use area fractions by ID in a healthy owner state. */
  coverageFractions(): Promise<SubcatchmentNamedValues> { return this.namedSettings("coverage"); }
  /** Update land-use coverage in `open` or `ended`.
   * @param values - Canonical land-use-ID map of area fractions, not percentages.
   * @param replace - False by default merges; true clears omitted land uses to zero.
   */
  updateCoverageFractions(values: SubcatchmentNamedValues, replace = false): Promise<void> {
    return this.updateNamedSettings("coverage", values, replace);
  }
  /** Clear all land-use coverage declarations in `open` or `ended`. */
  clearCoverageFractions(): Promise<void> { return this.clearNamedSettings("coverage"); }
  /** Read persistent external buildup increments in `open`, `running`, or `ended`; values are loads applied on runoff updates, not concentrations. */
  externalPollutantBuildupIncrement(): Promise<SubcatchmentPollutantValues> {
    return this.#call("subcatchmentExternalPollutantBuildupIncrement", this.id);
  }
  /** Read one persistent buildup increment in `open`, `running`, or `ended`.
   * @param pollutantId - Configured pollutant ID.
   * @returns External increment in that pollutant's load units.
   */
  externalPollutantBuildupIncrementValue(pollutantId: string): Promise<number> {
    return this.#call("subcatchmentExternalPollutantBuildupIncrementValue", this.id, pollutantId);
  }
  /** Update persistent buildup increments in `open`, `running`, or `ended`.
   * @param values - Configured pollutant-ID map of finite nonnegative increments in pollutant load units.
   * @param replace - False by default merges; true clears omitted pollutants to zero.
   * @returns Resolves after the update; increments persist until explicitly replaced or cleared.
   */
  updateExternalPollutantBuildupIncrement(
    values: SubcatchmentPollutantValues,
    replace = false,
  ): Promise<void> {
    return this.#call(
      "updateSubcatchmentExternalPollutantBuildupIncrement",
      this.id,
      values,
      replace,
    );
  }
  /** Clear all persistent buildup increments in `open`, `running`, or `ended`. */
  clearExternalPollutantBuildupIncrement(): Promise<void> {
    return this.updateExternalPollutantBuildupIncrement({}, true);
  }
  /** Read cumulative statistics in `running` or `complete`, before `end()`.
   * @returns Detached runoff statistics in project units.
   */
  statistics(): Promise<SubcatchmentStatistics> { return this.#call("subcatchmentStatistics", this.id); }
}

/** Rain-gage handle owned by one Simulation; source selection and temporary overrides are distinct persistent inputs. */
export class RainGage {
  /** Canonical configured rain-gage ID. */
  readonly id: string;
  readonly #call: Call;
  private constructor(id: string, call: Call) { this.id = id; this.#call = call; Object.freeze(this); }
  /** @internal */
  static create(id: string, call: Call): RainGage { return new RainGage(id, call); }
  /** Read current precipitation in `running`, `complete`, or `ended`.
   * @returns Detached rainfall, snowfall, and total rates in in/h or mm/h.
   */
  results(): Promise<RainGageResults> { return this.#call("rainGage", this.id); }
  /** Read total precipitation rate, not accumulated depth, in in/h or mm/h; requires `running`, `complete`, or `ended`. */
  totalPrecip(): Promise<number> { return this.results().then(({ totalPrecip }) => totalPrecip); }
  /** Read liquid rainfall rate in in/h or mm/h; requires `running`, `complete`, or `ended`. */
  rainfall(): Promise<number> { return this.results().then(({ rainfall }) => rainfall); }
  /** Read snowfall water-equivalent rate in in/h or mm/h; requires `running`, `complete`, or `ended`. */
  snowfall(): Promise<number> { return this.results().then(({ snowfall }) => snowfall); }
  /** Read the selected API source rate in a healthy owner state.
   * @returns Rate in in/h or mm/h, or null when the API is not the underlying source.
   */
  externalPrecipitationRate(): Promise<number | null> {
    return this.#call("rainGageExternalPrecipitationRate", this.id);
  }
  /** Select the API as the persistent precipitation source in `open`, `running`, or `ended`.
   * @param rate - Finite nonnegative rate in in/h or mm/h; zero selects a dry API source, not the original source.
   * @returns Resolves after source selection; the value persists until replaced.
   */
  useExternalPrecipitation(rate: number): Promise<void> {
    return this.#call("useRainGageExternalPrecipitation", this.id, rate);
  }
  /** Alias for selecting the persistent API precipitation source in `open`, `running`, or `ended`.
   * @param rate - Finite nonnegative rate in in/h or mm/h; this is not a temporary override.
   */
  setPrecipitation(rate: number): Promise<void> {
    return this.#call("setRainGagePrecipitation", this.id, rate);
  }
  /** Read the temporary source override in a healthy owner state.
   * @returns Rate in in/h or mm/h, or null when no override is active.
   */
  rainfallOverride(): Promise<number | null> {
    return this.#call("rainGageRainfallOverride", this.id);
  }
  /** Override the current source in `open`, `running`, or `ended` without changing source selection.
   * @param rate - Finite nonnegative rate in in/h or mm/h; null clears the override and resumes the selected underlying source.
   * @returns Resolves after the update; an override remains active until changed or cleared.
   */
  setRainfallOverride(rate: number | null): Promise<void> {
    return this.#call("setRainGageRainfallOverride", this.id, rate);
  }
}
