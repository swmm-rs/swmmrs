import { WorkerClient } from "./client.js";
import { LifecycleError } from "./exceptions.js";
import { bytes, createWorker } from "./runtime.js";
import type { ModelTime, NodeKind, LinkKind } from "./enums.js";
import type { FileContents } from "./types.js";
const EMPTY_TIMES: readonly ModelTime[] = [];

/** Supported result families in a binary output file. */
export type ResultElementType = "subcatchment" | "node" | "link" | "system";

/** Concentration units recorded for one pollutant. */
export type ConcentrationUnits =
  | "milligrams_per_liter"
  | "micrograms_per_liter"
  | "counts_per_liter";

/** Flow units recorded in an output header. */
export type OutputFlowUnits = "cfs" | "gpm" | "mgd" | "cms" | "lps" | "mld";

/** Unit system inferred from recognized output flow units. */
export type OutputUnitSystem = "us" | "si";

/** Preserve one unknown signed 32-bit categorical code. */
export interface UnknownCode {
  /** Unrecognized signed 32-bit value, retained without interpretation. */
  readonly code: number;
}

/** Select one unknown result attribute by its stored signed code. */
export interface ResultAttributeCode {
  /** Stored signed 32-bit result code; must be present in the selected family schema. */
  readonly code: number;
}

/** Select one pollutant column by index, exact text, bytes, or metadata name. */
export interface PollutantAttribute {
  /** Exact configured output pollutant selector; not supported for system results. */
  readonly selector: OutputElementSelector;
}

/** Known subcatchment columns. Rainfall/infiltration use in/h or mm/h; evaporation uses in/day or mm/day; snow depth uses in or mm; runoff and groundwater outflow use header flow units; groundwater elevation uses ft or m; soil moisture is a fraction. */
export type SubcatchmentResultAttribute =
  | "rainfall"
  | "snow_depth"
  | "evap_loss"
  | "infil_loss"
  | "runoff_rate"
  | "gw_outflow_rate"
  | "gw_table_elev"
  | "soil_moisture";

/** Known node columns. Depth/head use ft or m; ponded volume uses ft³ or m³; lateral/total inflow and flooding losses use header flow units. */
export type NodeResultAttribute =
  | "invert_depth"
  | "hydraulic_head"
  | "ponded_volume"
  | "lateral_inflow"
  | "total_inflow"
  | "flooding_losses";

/** Known link columns. Flow uses header flow units; depth uses ft or m; velocity uses ft/s or m/s; volume uses ft³ or m³; capacity is dimensionless. */
export type LinkResultAttribute =
  | "flow_rate"
  | "flow_depth"
  | "flow_velocity"
  | "flow_volume"
  | "capacity";

/** Known system columns. Temperature uses °F or °C; rainfall and `evap_infil_loss` use in/h or mm/h; `evap_rate` and `ptnl_evap_rate` use in/day or mm/day; snow depth uses in or mm; stored volume uses ft³ or m³; runoff and inflow/outflow/loss columns use header flow units. */
export type SystemResultAttribute =
  | "air_temp"
  | "rainfall"
  | "snow_depth"
  | "evap_infil_loss"
  | "runoff_flow"
  | "dry_weather_inflow"
  | "gw_inflow"
  | "rdii_inflow"
  | "direct_inflow"
  | "total_lateral_inflow"
  | "flood_losses"
  | "outfall_flows"
  | "volume_stored"
  | "evap_rate"
  | "ptnl_evap_rate";

/** One subcatchment schema column, including pollutant and unknown codes. */
export type SubcatchmentSchemaEntry = SubcatchmentResultAttribute | PollutantAttribute | ResultAttributeCode;
/** One node schema column, including pollutant and unknown codes. */
export type NodeSchemaEntry = NodeResultAttribute | PollutantAttribute | ResultAttributeCode;
/** One link schema column, including pollutant and unknown codes. */
export type LinkSchemaEntry = LinkResultAttribute | PollutantAttribute | ResultAttributeCode;
/** One system schema column; system results have no pollutant columns. */
export type SystemSchemaEntry = SystemResultAttribute | ResultAttributeCode;
/** Result-column selector; the attribute must belong to the selected element family. */
export type OutputAttribute =
  | SubcatchmentSchemaEntry
  | NodeSchemaEntry
  | LinkSchemaEntry
  | SystemSchemaEntry;

/** Zero-based integer index or exact stored name. Text is UTF-8 encoded; byte/name comparisons are case-sensitive, unlike Simulation collection lookups. Missing or ambiguous names reject. */
export type OutputElementSelector = number | string | Uint8Array | OutputName;

/** Ordered family, element, and result-attribute selection. */
export interface SeriesSelection {
  /** Result family whose schema is queried. */
  readonly elementType: ResultElementType;
  /** Required element selector for object families; must be null for system results. */
  readonly element: OutputElementSelector | null;
  /** Family-compatible built-in attribute, pollutant selector, or stored result code. */
  readonly attribute: OutputAttribute;
}

/** Lossless stored output name, retaining bytes even when UTF-8 decoding fails. */
export class OutputName {
  #bytes: Uint8Array;
  /** Decoded UTF-8 text, or null for invalid UTF-8. A leading byte-order mark is preserved. */
  readonly text: string | null;

  /** Copy an exact stored name byte sequence.
   * @param raw - Byte values copied into owned storage.
   */
  constructor(raw: ArrayLike<number>) {
    this.#bytes = new Uint8Array(raw);
    this.text = decodeName(this.#bytes);
    Object.freeze(this);
  }

  /** Return a defensive copy of the exact stored name bytes. */
  get raw(): Uint8Array { return this.#bytes.slice(); }
}

/** Finalized status code, or null when no output trailer was present. */
export interface RunStatus {
  /** Trailer status code, or null for incomplete output without a trailer. */
  readonly code: number | null;
  /** Whether a valid finalized trailer was present. */
  readonly isFinalized: boolean;
  /** Whether the finalized status code is zero. */
  readonly isSuccess: boolean;
}

/** Fixed report schedule and available complete-period count. */
export class ReportTiming {
  /** Nominal schedule origin in SWMM serial days, not an epoch-millisecond timestamp. */
  readonly reportScheduleOrigin: number;
  /** Fixed interval between report periods in seconds. */
  readonly reportStepSeconds: number;
  /** Number of available complete periods. */
  readonly periodCount: number;

  /** Construct report timing, normally obtained from reader metadata.
   * @param reportScheduleOrigin - SWMM serial-day schedule origin.
   * @param reportStepSeconds - Report interval in seconds.
   * @param periodCount - Available complete-period count.
   */
  constructor(reportScheduleOrigin: number, reportStepSeconds: number, periodCount: number) {
    this.reportScheduleOrigin = reportScheduleOrigin;
    this.reportStepSeconds = reportStepSeconds;
    this.periodCount = periodCount;
    Object.freeze(this);
  }

  /** Return the rounded timezone-free nominal date, not the exact stored serial-day value.
   * @param period - Zero-based integer period index; period zero is one report step after the origin.
   * @returns Nominal date, or null if the integer is outside the available range.
   * @throws `TypeError` when the index is not an integer.
   */
  nominalDate(period: number): ModelTime | null {
    if (!Number.isInteger(period)) throw new TypeError("period must be an integer");
    if (period < 0 || period >= this.periodCount) return null;
    return modelTimeFromSerial(this.reportScheduleOrigin + (period + 1) * this.reportStepSeconds / 86_400);
  }
}

/** Stored subcatchment identity and static area. */
export interface SubcatchmentMetadata {
  /** Zero-based stored subcatchment index. */
  readonly index: number;
  /** Exact stored name. */
  readonly name: OutputName;
  /** Stored area in acres or hectares, according to header units. */
  readonly area: number;
}

/** Stored node identity and static properties. */
export interface NodeMetadata {
  /** Zero-based stored node index. */
  readonly index: number;
  /** Exact stored name. */
  readonly name: OutputName;
  /** Recognized node subtype or unrecognized stored code. */
  readonly kind: NodeKind | UnknownCode;
  /** Invert elevation in ft or m, according to header units. */
  readonly invertElevation: number;
  /** Maximum depth in ft or m. */
  readonly maximumDepth: number;
}

/** Stored link identity and static properties. */
export interface LinkMetadata {
  /** Zero-based stored link index. */
  readonly index: number;
  /** Exact stored name. */
  readonly name: OutputName;
  /** Recognized link subtype or unrecognized stored code. */
  readonly kind: LinkKind | UnknownCode;
  /** Inlet offset in ft or m, according to header units. */
  readonly inletOffset: number;
  /** Outlet offset in ft or m. */
  readonly outletOffset: number;
  /** Maximum depth in ft or m. */
  readonly maximumDepth: number;
  /** Stored link length in ft or m. */
  readonly length: number;
}

/** Stored pollutant identity, name, and concentration units. */
export interface PollutantMetadata {
  /** Zero-based stored pollutant index. */
  readonly index: number;
  /** Exact stored name. */
  readonly name: OutputName;
  /** Recognized concentration units or an unrecognized stored code. */
  readonly concentrationUnits: ConcentrationUnits | UnknownCode;
}

/** Ordered physical result schemas for every output family. */
export interface ResultSchema {
  /** Subcatchment columns in stored order. */
  readonly subcatchment: readonly SubcatchmentSchemaEntry[];
  /** Node columns in stored order. */
  readonly node: readonly NodeSchemaEntry[];
  /** Link columns in stored order. */
  readonly link: readonly LinkSchemaEntry[];
  /** System columns in stored order. */
  readonly system: readonly SystemSchemaEntry[];
}

/** Immutable metadata parsed from one output byte buffer. */
export interface OutputMetadata {
  /** Stored numeric solver release identifier. */
  readonly solverRelease: number;
  /** Trailer finalization and success status. */
  readonly runStatus: RunStatus;
  /** Recognized flow units or an unrecognized header code. */
  readonly flowUnits: OutputFlowUnits | UnknownCode;
  /** Unit system inferred from recognized flow units; null otherwise. */
  readonly unitSystem: OutputUnitSystem | null;
  /** Validated nominal report schedule and complete-period count. */
  readonly reportTiming: ReportTiming;
  /** Subcatchment metadata in stored order. */
  readonly subcatchments: readonly SubcatchmentMetadata[];
  /** Node metadata in stored order. */
  readonly nodes: readonly NodeMetadata[];
  /** Link metadata in stored order. */
  readonly links: readonly LinkMetadata[];
  /** Pollutant metadata in stored order. */
  readonly pollutants: readonly PollutantMetadata[];
  /** Physical column schemas, including unknown codes. */
  readonly resultSchema: ResultSchema;
}

/** Immutable selection-labelled values aligned to a bulk result axis. */
export interface OutputValueSeries {
  /** Resolved selection, with numeric element/pollutant indices. */
  readonly selection: SeriesSelection;
  /** Values aligned to the enclosing result's time axis, in stored output units. */
  readonly values: readonly number[];
}

/** Immutable values and nominal dates for one selected series. */
export interface OutputTimeSeries {
  /** Resolved selection, with numeric element/pollutant indices. */
  readonly selection: SeriesSelection;
  /** Rounded nominal dates for the selected period range. */
  readonly times: readonly ModelTime[];
  /** Values aligned to `times`, in stored output units. */
  readonly values: readonly number[];
}

/** Immutable column-oriented result for an ordered bulk request. */
export class BulkSeriesResult {
  /** Shared rounded nominal dates for the selected period range. */
  readonly times: readonly ModelTime[];
  /** Columns in requested selection order, including duplicates. */
  readonly series: readonly OutputValueSeries[];

  /** Copy the outer axis and column arrays; normally constructed by the reader with immutable, aligned columns.
   * @param times - Shared nominal date axis.
   * @param series - Selection-labelled columns aligned to that axis.
   */
  constructor(times: readonly ModelTime[], series: readonly OutputValueSeries[]) {
    this.times = freezeArray(times);
    this.series = freezeArray(series);
    Object.freeze(this);
  }

  /** Read one value using offsets local to this result, not absolute file periods.
   * @param periodOffset - Zero-based offset into `times`.
   * @param selectionOffset - Zero-based offset into `series`.
   * @returns Selected value in stored output units.
   * @throws `RangeError` if either offset is non-integral or out of range.
   */
  value(periodOffset: number, selectionOffset: number): number {
    if (!Number.isInteger(periodOffset) || periodOffset < 0 || periodOffset >= this.times.length) {
      throw new RangeError("period offset is outside this result");
    }
    if (!Number.isInteger(selectionOffset) || selectionOffset < 0 || selectionOffset >= this.series.length) {
      throw new RangeError("selection offset is outside this result");
    }
    return this.series[selectionOffset]!.values[periodOffset]!;
  }
}

/** Range and I/O strategy for output queries. Bounds are half-open. */
export interface OutputReadOptions {
  /** Inclusive integer period offset or timezone-free date; null/omission starts at zero. Dates resolve to the first nominal period at or after the bound. */
  readonly start?: number | ModelTime | null;
  /** Exclusive integer period offset or timezone-free date; null/omission ends at periodCount. Integer bounds must lie within 0…periodCount; reversed ranges reject. */
  readonly end?: number | ModelTime | null;
  /** Defaults to false (whole-period reads). True uses selective adjacent-run reads to reduce working memory without changing values. */
  readonly lowMemory?: boolean;
}

/** Alias for callers that only need period/date bounds. */
export type OutputRange = Omit<OutputReadOptions, "lowMemory">;

/** Worker override for one standalone reader. */
export interface OutputReaderOptions {
  /** Override the bundled worker asset location; resolved by the runtime when opening the reader. */
  readonly workerUrl?: string | URL;
}

/** Structured output-reader failure preserved across the worker boundary. */
export class OutputError extends Error {
  /** Machine-readable failure category, such as `invalid_period_range`, `element_not_found`, or `ambiguous_element`. */
  readonly category: string;
  /** Failing reader operation when available. */
  readonly operation: string | undefined;

  /** Construct a structured reader error.
   * @param category - Failure classification.
   * @param message - Human-readable diagnostic.
   * @param operation - Optional operation context; omitted when unavailable.
   */
  constructor(category: string, message: string, operation?: string) {
    super(message);
    this.name = "OutputError";
    this.category = category;
    this.operation = operation;
  }
}

type Operation<A extends unknown[], R> = { args: A; result: R };
type NativeSelection = [ResultElementType, number, number];
type NativeAttribute = string | { readonly pollutant: number } | { readonly code: number };
type NativeName = { readonly raw: readonly number[]; readonly text: string | null };
type NativeMetadataPayload = {
  readonly solverRelease: number;
  readonly runStatus: { readonly code: number | null; readonly isFinalized: boolean; readonly isSuccess: boolean };
  readonly flowUnits: string | { readonly code: number };
  readonly unitSystem: "us" | "si" | null;
  readonly reportTiming: { readonly reportScheduleOrigin: number; readonly reportStepSeconds: number; readonly periodCount: number };
  readonly subcatchments: readonly { readonly index: number; readonly name: NativeName; readonly area: number }[];
  readonly nodes: readonly { readonly index: number; readonly name: NativeName; readonly kind: string | { readonly code: number }; readonly invertElevation: number; readonly maximumDepth: number }[];
  readonly links: readonly { readonly index: number; readonly name: NativeName; readonly kind: string | { readonly code: number }; readonly inletOffset: number; readonly outletOffset: number; readonly maximumDepth: number; readonly length: number }[];
  readonly pollutants: readonly { readonly index: number; readonly name: NativeName; readonly concentrationUnits: string | { readonly code: number } }[];
  readonly resultSchema: {
    readonly subcatchment: readonly NativeAttribute[];
    readonly node: readonly NativeAttribute[];
    readonly link: readonly NativeAttribute[];
    readonly system: readonly NativeAttribute[];
  };
};
type NativeBulkPayload = {
  readonly times: readonly number[];
  readonly series: readonly {
    readonly selection: { readonly elementType: ResultElementType; readonly element: number | null; readonly attribute: NativeAttribute };
    readonly values: readonly number[];
  }[];
};

/** Operations consumed by the standalone output worker. Main extends the shared worker contract with these entries. */
export interface OutputOperations {
  openOutput: Operation<[Uint8Array], NativeMetadataPayload>;
  outputMetadata: Operation<[], NativeMetadataPayload>;
  outputReadBulkSeries: Operation<[readonly NativeSelection[], number, number], NativeBulkPayload>;
  outputReadBulkSeriesByPeriod: Operation<[readonly NativeSelection[], number, number], NativeBulkPayload>;
  outputReadStoredDates: Operation<[number, number], readonly number[]>;
  outputSubcatchmentSeries: Operation<[number, number, number, number], NativeBulkPayload>;
  outputNodeSeries: Operation<[number, number, number, number], NativeBulkPayload>;
  outputLinkSeries: Operation<[number, number, number, number], NativeBulkPayload>;
  outputSystemSeries: Operation<[number, number, number], NativeBulkPayload>;
  closeOutput: Operation<[], void>;
}

type OutputMethod = keyof OutputOperations;
type OutputCall = <K extends OutputMethod>(method: K, ...args: OutputOperations[K]["args"]) => Promise<OutputOperations[K]["result"]>;

/** Byte-owned standalone output reader with its own worker, independent of Simulation. Always await `close()`; copied results and metadata remain usable afterward. */
export class OutputReader implements AsyncDisposable {
  /** Immutable parsed metadata, available without another worker query. */
  readonly metadata: OutputMetadata;
  readonly #client: WorkerClient;
  #times: readonly ModelTime[] | undefined;
  #closing: Promise<void> | undefined;
  #closed = false;

  private constructor(client: WorkerClient, metadata: OutputMetadata) {
    this.#client = client;
    this.metadata = metadata;
    Object.freeze(this);
  }

  /** Open finalized or incomplete SWMM output in an independent worker.
   * @param input - File contents, never a host path. In Node, pass bytes from `fs.readFile`.
   * @param options - Worker asset override; defaults to the bundled worker.
   * @returns Reader with immutable validated metadata and available complete periods.
   * @throws {@link OutputError} for invalid output; a worker created during a failed open is stopped.
   */
  static async open(input: FileContents, options: OutputReaderOptions = {}): Promise<OutputReader> {
    const worker = await createWorker(options);
    const client = new WorkerClient(worker);
    const call = client.call as unknown as OutputCall;
    try {
      const payload = await call("openOutput", await bytes(input));
      return new OutputReader(client, metadataFromPayload(payload));
    } catch (error) {
      client.stop(error instanceof Error ? error : new Error(String(error)));
      throw outputFailure(error, "openOutput");
    }
  }

  /** True when the opened bytes contained a valid finalized trailer. */
  get isFinalized(): boolean { return this.metadata.runStatus.isFinalized; }

  /** Lazily derive the shared nominal report-date axis from validated metadata. */
  get times(): readonly ModelTime[] {
    if (this.#times === undefined) {
      const values = Array.from({ length: this.metadata.reportTiming.periodCount }, (_, period) =>
        this.metadata.reportTiming.nominalDate(period)!,
      );
      this.#times = freezeArray(values);
    }
    return this.#times;
  }

  /** Read ordered selections; duplicates remain repeated columns and empty dimensions are preserved.
   * @param selections - Family-compatible element/attribute selections in desired column order.
   * @param options - Half-open range and I/O strategy; defaults to all periods and whole-period reads.
   * @returns Detached immutable columns sharing one nominal date axis.
   * @throws {@link OutputError} for missing/ambiguous elements, absent attributes, or invalid ranges; TypeError/RangeError for malformed selectors. Reads after close reject with LifecycleError.
   * @example
   * ```typescript
   * import type { OutputReader } from "@swmmrs/swmmrs";
   * declare const reader: OutputReader;
   * const result = await reader.readBulkSeries([
   *   { elementType: "node", element: "J1", attribute: "hydraulic_head" },
   * ], { start: 0, end: 24, lowMemory: true });
   * console.log(result.times, result.series[0]?.values);
   * ```
   */
  readBulkSeries(selections: readonly SeriesSelection[], options: OutputReadOptions = {}): Promise<BulkSeriesResult> {
    const [start, end] = this.#resolveBounds(options);
    const nativeSelections = selections.map((selection) => this.#nativeSelection(selection));
    const method = options.lowMemory === true ? "outputReadBulkSeries" : "outputReadBulkSeriesByPeriod";
    return this.#call(method, nativeSelections, start, end).then((payload) => bulkFromPayload(payload, this.metadata));
  }

  /** Read one subcatchment column with the validation and lifecycle contract of `readBulkSeries`.
   * @param element - Zero-based index or exact stored name selector.
   * @param attribute - Subcatchment attribute, pollutant, or stored result code.
   * @param options - Half-open range and I/O strategy; defaults to all periods and whole-period reads.
   * @returns Immutable selection, nominal dates, and aligned values.
   */
  subcatchmentSeries(element: OutputElementSelector, attribute: SubcatchmentResultAttribute | PollutantAttribute | ResultAttributeCode, options: OutputReadOptions = {}): Promise<OutputTimeSeries> {
    return this.#singleSeries({ elementType: "subcatchment", element, attribute }, options);
  }

  /** Read one node column with the validation and lifecycle contract of `readBulkSeries`.
   * @param element - Zero-based index or exact stored name selector.
   * @param attribute - Node attribute, pollutant, or stored result code.
   * @param options - Half-open range and I/O strategy; defaults to all periods and whole-period reads.
   * @returns Immutable selection, nominal dates, and aligned values.
   */
  nodeSeries(element: OutputElementSelector, attribute: NodeResultAttribute | PollutantAttribute | ResultAttributeCode, options: OutputReadOptions = {}): Promise<OutputTimeSeries> {
    return this.#singleSeries({ elementType: "node", element, attribute }, options);
  }

  /** Read one link column with the validation and lifecycle contract of `readBulkSeries`.
   * @param element - Zero-based index or exact stored name selector.
   * @param attribute - Link attribute, pollutant, or stored result code.
   * @param options - Half-open range and I/O strategy; defaults to all periods and whole-period reads.
   * @returns Immutable selection, nominal dates, and aligned values.
   */
  linkSeries(element: OutputElementSelector, attribute: LinkResultAttribute | PollutantAttribute | ResultAttributeCode, options: OutputReadOptions = {}): Promise<OutputTimeSeries> {
    return this.#singleSeries({ elementType: "link", element, attribute }, options);
  }

  /** Read one system column with the validation and lifecycle contract of `readBulkSeries`.
   * @param attribute - System attribute or stored result code; pollutants are not supported.
   * @param options - Half-open range and I/O strategy; defaults to all periods and whole-period reads.
   * @returns Immutable selection with null element, nominal dates, and aligned values.
   */
  systemSeries(attribute: SystemResultAttribute | ResultAttributeCode, options: OutputReadOptions = {}): Promise<OutputTimeSeries> {
    return this.#singleSeries({ elementType: "system", element: null, attribute }, options);
  }

  /** Read exact stored report dates rather than the rounded nominal date axis; requires an open reader.
   * @param options - Half-open period/date bounds; defaults to all complete periods.
   * @returns Immutable SWMM serial-day values read from the output bytes.
   * @throws {@link OutputError} for invalid ranges or unreadable output; LifecycleError after close.
   */
  readStoredDates(options: OutputRange = {}): Promise<readonly number[]> {
    const [start, end] = this.#resolveBounds(options);
    return this.#call("outputReadStoredDates", start, end).then((values) => freezeArray(values));
  }

  /** Release the reader worker. Repeated calls share one cleanup request; future reads reject.
   * @returns Resolves after cleanup. Previously copied metadata/results remain usable.
   */
  async close(): Promise<void> {
    if (this.#closing !== undefined) return this.#closing;
    this.#closing = this.#call("closeOutput").finally(() => {
      this.#closed = true;
      this.#client.stop(new LifecycleError({ message: "Output reader is closed", operation: "closeOutput" }));
    });
    return this.#closing;
  }

  /** Delegate asynchronous disposal to `close()`.
   * @returns Reader cleanup completion.
   */
  [Symbol.asyncDispose](): Promise<void> { return this.close(); }

  #singleSeries(selection: SeriesSelection, options: OutputReadOptions): Promise<OutputTimeSeries> {
    return this.readBulkSeries([selection], options).then((result) => {
      const series = result.series[0];
      if (series === undefined) throw new OutputError("output", "native output returned no series");
      return Object.freeze({ selection: series.selection, times: result.times, values: series.values });
    });
  }

  #nativeSelection(selection: SeriesSelection): NativeSelection {
    if (selection.elementType === "system") {
      if (selection.element !== null) throw new TypeError("system selections do not carry an element");
      return ["system", 0, attributeCode(this.metadata, selection.elementType, selection.attribute)];
    }
    if (selection.element === null) throw new TypeError("non-system selections require an element");
    return [
      selection.elementType,
      resolveElement(this.metadata, selection.elementType, selection.element),
      attributeCode(this.metadata, selection.elementType, selection.attribute),
    ];
  }

  #resolveBounds(options: OutputReadOptions): [number, number] {
    if (options === null || typeof options !== "object") throw new TypeError("options must be an object");
    if (options.lowMemory !== undefined && typeof options.lowMemory !== "boolean") {
      throw new TypeError("lowMemory must be a boolean");
    }
    const count = this.metadata.reportTiming.periodCount;
    const needsAxis = isModelTime(options.start) || isModelTime(options.end);
    const axis = needsAxis ? this.times : EMPTY_TIMES;
    const start = resolveBound(options.start, 0, count, axis);
    const end = resolveBound(options.end, count, count, axis);
    if (isModelTime(options.start) && isModelTime(options.end) && options.start > options.end) {
      throw new OutputError("invalid_period_range", "start bound must not be later than end bound");
    }
    if (start > end) throw new OutputError("invalid_period_range", "resolved period range is inverted");
    return [start, end];
  }

  #call<K extends OutputMethod>(method: K, ...args: OutputOperations[K]["args"]): Promise<OutputOperations[K]["result"]> {
    if (this.#closed) return Promise.reject(new LifecycleError({ message: "Output reader is closed", operation: String(method) }));
    const call = this.#client.call as unknown as OutputCall;
    return call(method, ...args).catch((error) => { throw outputFailure(error, String(method)); });
  }
}

function metadataFromPayload(payload: NativeMetadataPayload): OutputMetadata {
  const timing = new ReportTiming(payload.reportTiming.reportScheduleOrigin, payload.reportTiming.reportStepSeconds, payload.reportTiming.periodCount);
  const metadata: OutputMetadata = {
    solverRelease: payload.solverRelease,
    runStatus: Object.freeze({
      code: payload.runStatus.code,
      isFinalized: payload.runStatus.code !== null,
      isSuccess: payload.runStatus.code === 0,
    }),
    flowUnits: category(payload.flowUnits) as OutputFlowUnits | UnknownCode,
    unitSystem: payload.unitSystem,
    reportTiming: timing,
    subcatchments: freezeArray(payload.subcatchments.map((item) => Object.freeze({ index: item.index, name: nameFromPayload(item.name), area: item.area }))),
    nodes: freezeArray(payload.nodes.map((item) => Object.freeze({ index: item.index, name: nameFromPayload(item.name), kind: category(item.kind) as NodeKind | UnknownCode, invertElevation: item.invertElevation, maximumDepth: item.maximumDepth }))),
    links: freezeArray(payload.links.map((item) => Object.freeze({ index: item.index, name: nameFromPayload(item.name), kind: category(item.kind) as LinkKind | UnknownCode, inletOffset: item.inletOffset, outletOffset: item.outletOffset, maximumDepth: item.maximumDepth, length: item.length }))),
    pollutants: freezeArray(payload.pollutants.map((item) => Object.freeze({ index: item.index, name: nameFromPayload(item.name), concentrationUnits: category(item.concentrationUnits) as ConcentrationUnits | UnknownCode }))),
    resultSchema: Object.freeze({
      subcatchment: freezeArray(payload.resultSchema.subcatchment.map((item) => attributeFromPayload(item, "subcatchment"))),
      node: freezeArray(payload.resultSchema.node.map((item) => attributeFromPayload(item, "node"))),
      link: freezeArray(payload.resultSchema.link.map((item) => attributeFromPayload(item, "link"))),
      system: freezeArray(payload.resultSchema.system.map((item) => attributeFromPayload(item, "system"))),
    }),
  };
  return Object.freeze(metadata);
}

function bulkFromPayload(payload: NativeBulkPayload, metadata: OutputMetadata): BulkSeriesResult {
  const times = freezeArray(payload.times.map(modelTimeFromSerial));
  const series = payload.series.map((item) => Object.freeze({
    selection: selectionFromPayload(item.selection, metadata),
    values: freezeArray(item.values),
  }));
  return new BulkSeriesResult(times, series);
}

function selectionFromPayload(payload: NativeBulkPayload["series"][number]["selection"], metadata: OutputMetadata): SeriesSelection {
  const elementType = payload.elementType;
  const attribute = attributeFromPayload(payload.attribute, elementType);
  return Object.freeze({ elementType, element: elementType === "system" ? null : payload.element, attribute });
}

function attributeFromPayload<K extends ResultElementType>(value: NativeAttribute, _family: K): ResultSchema[K][number] {
  const attribute = typeof value === "string" ? value
    : Object.freeze("pollutant" in value ? { selector: value.pollutant } : { code: value.code });
  return attribute as ResultSchema[K][number];
}

function attributeCode(metadata: OutputMetadata, family: ResultElementType, attribute: OutputAttribute): number {
  if (typeof attribute === "string") {
    const known = knownAttributeCode(family, attribute);
    if (known === undefined) throw new TypeError(`unknown ${family} attribute ${attribute}`);
    return known;
  }
  if ("selector" in attribute) {
    const pollutant = resolvePollutant(metadata, attribute.selector);
    const schema = schemaFor(metadata, family);
    const entry = schema.find((candidate) => typeof candidate === "object" && candidate !== null && "selector" in candidate && candidate.selector === pollutant);
    if (entry === undefined) throw new OutputError("attribute_not_found", `pollutant ${pollutant} is absent from ${family} schema`);
    return pollutantCode(family, pollutant);
  }
  if (!Number.isInteger(attribute.code) || attribute.code < -2_147_483_648 || attribute.code > 2_147_483_647) {
    throw new RangeError("result code must fit a signed 32-bit integer");
  }
  return attribute.code;
}

function knownAttributeCode(family: ResultElementType, attribute: string): number | undefined {
  const known: Record<ResultElementType, readonly string[]> = {
    subcatchment: ["rainfall", "snow_depth", "evap_loss", "infil_loss", "runoff_rate", "gw_outflow_rate", "gw_table_elev", "soil_moisture"],
    node: ["invert_depth", "hydraulic_head", "ponded_volume", "lateral_inflow", "total_inflow", "flooding_losses"],
    link: ["flow_rate", "flow_depth", "flow_velocity", "flow_volume", "capacity"],
    system: ["air_temp", "rainfall", "snow_depth", "evap_infil_loss", "runoff_flow", "dry_weather_inflow", "gw_inflow", "rdii_inflow", "direct_inflow", "total_lateral_inflow", "flood_losses", "outfall_flows", "volume_stored", "evap_rate", "ptnl_evap_rate"],
  };
  const index = known[family].indexOf(attribute);
  return index < 0 ? undefined : index;
}

function pollutantCode(family: ResultElementType, index: number): number {
  const offset = family === "subcatchment" ? 8 : family === "node" ? 6 : 5;
  if (family === "system") throw new TypeError("system selections do not support pollutants");
  const code = offset + index;
  if (code > 2_147_483_647) throw new RangeError("pollutant result code does not fit a signed 32-bit integer");
  return code;
}

function resolveElement(metadata: OutputMetadata, family: Exclude<ResultElementType, "system">, selector: OutputElementSelector): number {
  if (typeof selector === "number") {
    if (!Number.isInteger(selector) || selector < 0) throw new RangeError("element index must be a nonnegative integer");
    const count = family === "subcatchment" ? metadata.subcatchments.length : family === "node" ? metadata.nodes.length : metadata.links.length;
    if (selector >= count) throw new OutputError("invalid_element_id", `invalid ${family} element index ${selector}`);
    return selector;
  }
  const raw = selector instanceof OutputName ? selector.raw : typeof selector === "string" ? new TextEncoder().encode(selector) : selector;
  const records = family === "subcatchment" ? metadata.subcatchments : family === "node" ? metadata.nodes : metadata.links;
  let found = -1;
  let count = 0;
  for (const [index, record] of records.entries()) {
    if (bytesEqual(record.name.raw, raw)) { found = index; count += 1; }
  }
  if (count === 0) throw new OutputError("element_not_found", `${family} element name was not found`);
  if (count > 1) throw new OutputError("ambiguous_element", `${family} element name matched ${count} elements`);
  return found;
}

function resolvePollutant(metadata: OutputMetadata, selector: OutputElementSelector): number {
  if (typeof selector === "number") {
    if (!Number.isInteger(selector) || selector < 0 || selector >= metadata.pollutants.length) throw new OutputError("pollutant_not_found", `pollutant index ${selector} was not found`);
    return selector;
  }
  const raw = selector instanceof OutputName ? selector.raw : typeof selector === "string" ? new TextEncoder().encode(selector) : selector;
  let found = -1;
  let count = 0;
  for (const [index, pollutant] of metadata.pollutants.entries()) {
    if (bytesEqual(pollutant.name.raw, raw)) { found = index; count += 1; }
  }
  if (count === 0) throw new OutputError("pollutant_not_found", "pollutant name was not found");
  if (count > 1) throw new OutputError("ambiguous_pollutant", `pollutant name matched ${count} pollutants`);
  return found;
}

function schemaFor(metadata: OutputMetadata, family: ResultElementType): readonly OutputAttribute[] {
  return metadata.resultSchema[family] as readonly OutputAttribute[];
}

function resolveBound(bound: number | ModelTime | null | undefined, fallback: number, count: number, axis: readonly ModelTime[]): number {
  if (bound === undefined || bound === null) return fallback;
  if (typeof bound === "number") {
    if (!Number.isInteger(bound)) throw new TypeError("period bounds must be integers or ModelTime strings");
    if (bound < 0 || bound > count) throw new RangeError("period bound is outside the available range");
    return bound;
  }
  if (!isModelTime(bound)) throw new TypeError("date bounds must be timezone-free ModelTime strings");
  let low = 0;
  let high = axis.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (axis[middle]! < bound) low = middle + 1;
    else high = middle;
  }
  return low;
}

function isModelTime(value: unknown): value is ModelTime {
  if (typeof value !== "string") return false;
  const match = /^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})$/.exec(value);
  if (match === null) return false;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const hour = Number(match[4]);
  const minute = Number(match[5]);
  const second = Number(match[6]);
  if (year < 1 || year > 9999 || month < 1 || month > 12 || hour > 23 || minute > 59 || second > 59) return false;
  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  date.setUTCHours(hour, minute, second, 0);
  return date.getUTCFullYear() === year
    && date.getUTCMonth() === month - 1
    && date.getUTCDate() === day
    && date.getUTCHours() === hour
    && date.getUTCMinutes() === minute
    && date.getUTCSeconds() === second;
}

function modelTimeFromSerial(serial: number): ModelTime {
  const totalSeconds = serial * 86_400;
  if (!Number.isFinite(totalSeconds)) throw new OutputError("datetime_out_of_range", "nominal report date is outside the supported time range");
  const rounded = roundHalfEven(totalSeconds);
  const unixSeconds = rounded - 2_209_161_600;
  if (unixSeconds < -62_135_596_800 || unixSeconds > 253_402_300_799) {
    throw new OutputError("datetime_out_of_range", "nominal report date is outside the supported time range");
  }
  const date = new Date(unixSeconds * 1_000);
  if (!Number.isFinite(date.getTime())) throw new OutputError("datetime_out_of_range", "nominal report date is outside the supported time range");
  return date.toISOString().slice(0, 19) as ModelTime;
}

function roundHalfEven(value: number): number {
  const lower = Math.floor(value);
  const fraction = value - lower;
  if (fraction < 0.5) return lower;
  if (fraction > 0.5) return lower + 1;
  return lower % 2 === 0 ? lower : lower + 1;
}

function decodeName(raw: Uint8Array): string | null {
  try { return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(raw); }
  catch { return null; }
}

function nameFromPayload(value: NativeName): OutputName {
  return new OutputName(value.raw);
}

function category(value: string | { readonly code: number }): string | UnknownCode {
  return typeof value === "string" ? value : Object.freeze({ code: value.code });
}

function bytesEqual(left: ArrayLike<number>, right: ArrayLike<number>): boolean {
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index += 1) if (left[index] !== right[index]) return false;
  return true;
}

function freezeArray<T>(values: readonly T[]): readonly T[] {
  return Object.freeze(Array.from(values));
}

function outputFailure(error: unknown, operation: string): Error {
  if (error instanceof OutputError) return error;
  const source = error as { category?: unknown; message?: unknown; operation?: unknown } | null;
  if (source && typeof source.category === "string") {
    return new OutputError(source.category, typeof source.message === "string" ? source.message : String(error), typeof source.operation === "string" ? source.operation : operation);
  }
  return error instanceof Error ? error : new Error(String(error));
}
