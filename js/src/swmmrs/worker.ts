import type { Simulation as NativeSimulation, OutputReader as NativeOutputReader } from "../../dist/swmmrs.js";
import type { ConfigurationDiagnostic, ErrorDetails } from "./exceptions.js";
import type { Request, Response, Result, Method } from "./protocol.js";
import type { RunResults } from "./types.js";
import type { CheckpointBundle } from "./scenarios.js";
import { loadWasm } from "./runtime.js";

// This module runs in a dedicated worker; the public build also uses DOM types.
const host = globalThis as unknown as {
  onmessage: ((event: MessageEvent<Request>) => void) | null;
  postMessage(message: Response, transfer?: Transferable[]): void;
};
const decoder = new TextDecoder();
let simulation: NativeSimulation | undefined;
let outputReader: NativeOutputReader | undefined;
let terminateThreadPool: (() => void) | undefined;
let queue = Promise.resolve();

function shutdown(): void {
  const errors: unknown[] = [];
  try { simulation?.close(); } catch (error) { errors.push(error); }
  try { simulation?.free(); } catch (error) { errors.push(error); }
  simulation = undefined;
  try { outputReader?.free(); } catch (error) { errors.push(error); }
  outputReader = undefined;
  try { terminateThreadPool?.(); } catch (error) { errors.push(error); }
  if (errors.length) {
    const [primary, ...secondary] = errors;
    const error = primary instanceof Error ? primary : new Error(String(primary));
    if (secondary.length) Object.assign(error, { cleanupError: secondary.map(String).join("; ") });
    throw error;
  }
}

function results(owner: NativeSimulation): RunResults {
  return {
    report: decoder.decode(owner.readFile("model.rpt")),
    output: owner.status().saveResults ? owner.readFile("model.out") : new Uint8Array(),
  };
}

async function dispatch(request: Request): Promise<void> {
  const { id, method } = request;
  try {
    let result: Result<Method> = undefined;
    if (request.method === "open" || request.method === "resumeCheckpoint" || request.method === "openOutput") {
      if (simulation || outputReader) throw new Error("An owner is already open");
      const threads = request.method === "open" ? request.args[2]
        : request.method === "resumeCheckpoint" ? request.args[1] : 1;
      const bindings = threads === 1
        ? await import("../../dist/serial/swmmrs.js")
        : await import("../../dist/swmmrs.js");
      const moduleUrl = threads === 1
        ? new URL("../../dist/serial/swmmrs.js", import.meta.url).href
        : new URL("../../dist/swmmrs.js", import.meta.url).href;
      terminateThreadPool = bindings.terminateThreadPool;
      await loadWasm(bindings, moduleUrl);
      await bindings.initThreadPool(threads, moduleUrl);
      if (request.method === "openOutput") {
        outputReader = new bindings.OutputReader(request.args[0]);
        result = outputReader.outputMetadata();
      } else {
        simulation = request.method === "open"
          ? new bindings.Simulation(request.args[0], request.args[1])
          : bindings.Simulation.resumeCheckpoint(request.args[0]);
        result = simulation.info();
      }
    } else if (request.method === "close" || request.method === "closeOutput") {
      shutdown();
    } else if (request.method === "outputMetadata" || request.method === "outputReadBulkSeries" ||
        request.method === "outputReadBulkSeriesByPeriod" || request.method === "outputReadStoredDates" ||
        request.method === "outputSubcatchmentSeries" || request.method === "outputNodeSeries" ||
        request.method === "outputLinkSeries" || request.method === "outputSystemSeries") {
      if (!outputReader) throw new Error("Output reader is closed");
      switch (request.method) {
        case "outputMetadata": result = outputReader.outputMetadata(); break;
        case "outputReadBulkSeries": result = outputReader.outputReadBulkSeries(...request.args); break;
        case "outputReadBulkSeriesByPeriod": result = outputReader.outputReadBulkSeriesByPeriod(...request.args); break;
        case "outputReadStoredDates": result = outputReader.outputReadStoredDates(...request.args); break;
        case "outputSubcatchmentSeries": result = outputReader.outputSubcatchmentSeries(...request.args); break;
        case "outputNodeSeries": result = outputReader.outputNodeSeries(...request.args); break;
        case "outputLinkSeries": result = outputReader.outputLinkSeries(...request.args); break;
        case "outputSystemSeries": result = outputReader.outputSystemSeries(...request.args); break;
      }
    } else {
      if (!simulation) throw new Error("Simulation is closed");
      switch (request.method) {
        case "state": result = simulation.state as Result<"state">; break;
        case "info": result = simulation.info(); break;
        case "status": result = simulation.status(); break;
        case "start": simulation.start(...request.args); break;
        case "step": result = simulation.step() ?? null; break;
        case "stride": result = simulation.stride(...request.args) ?? null; break;
        case "node": result = simulation.node(...request.args); break;
        case "link": result = simulation.link(...request.args); break;
        case "nodes": result = simulation.nodes(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "links": result = simulation.links(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "subcatchment": result = simulation.subcatchment(...request.args); break;
        case "nodeExternalInflow": result = simulation.nodeExternalInflow(...request.args); break;
        case "outfallFixedStage": result = simulation.outfallFixedStage(...request.args) ?? null; break;
        case "nodeQuality": result = simulation.nodeQuality(...request.args); break;
        case "nodeQualitySnapshot": result = simulation.nodeQualitySnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "overrideNodePollutantConcentrations": simulation.overrideNodePollutantConcentrations(...request.args); break;
        case "nodeExternalPollutantMassFlux": result = simulation.nodeExternalPollutantMassFlux(...request.args); break;
        case "nodeExternalPollutantMassFluxValue": result = simulation.nodeExternalPollutantMassFluxValue(...request.args); break;
        case "updateNodeExternalPollutantMassFlux": simulation.updateNodeExternalPollutantMassFlux(...request.args); break;
        case "nodeStatistics": result = simulation.nodeStatistics(...request.args); break;
        case "storageStatistics": result = simulation.storageStatistics(...request.args); break;
        case "outfallStatistics": result = simulation.outfallStatistics(...request.args); break;
        case "nodeStatisticsSnapshot": result = simulation.nodeStatisticsSnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "nodeTotalInflowVolume": result = simulation.nodeTotalInflowVolume(...request.args); break;
        case "linkQuality": result = simulation.linkQuality(...request.args); break;
        case "linkQualitySnapshot": result = simulation.linkQualitySnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "overrideLinkPollutantConcentrations": simulation.overrideLinkPollutantConcentrations(...request.args); break;
        case "linkExternalPollutantMassFlux": result = simulation.linkExternalPollutantMassFlux(...request.args); break;
        case "linkExternalPollutantMassFluxValue": result = simulation.linkExternalPollutantMassFluxValue(...request.args); break;
        case "updateLinkExternalPollutantMassFlux": simulation.updateLinkExternalPollutantMassFlux(...request.args); break;
        case "setLinkFlowLimit": simulation.setLinkFlowLimit(...request.args); break;
        case "linkStatistics": result = simulation.linkStatistics(...request.args); break;
        case "pumpStatistics": result = simulation.pumpStatistics(...request.args); break;
        case "linkStatisticsSnapshot": result = simulation.linkStatisticsSnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "linkInlet": result = simulation.linkInlet(...request.args); break;
        case "inletResults": result = simulation.inletResults(...request.args); break;
        case "updateInlet": simulation.updateInlet(...request.args); break;
        case "subcatchments": result = simulation.subcatchments(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "rainGage": result = simulation.rainGage(...request.args); break;
        case "nodeConfiguration": result = simulation.nodeConfiguration(...request.args); break;
        case "linkConfiguration": result = simulation.linkConfiguration(...request.args); break;
        case "useRainGageExternalPrecipitation": simulation.useRainGageExternalPrecipitation(...request.args); break;
        case "rainGageExternalPrecipitationRate": result = simulation.rainGageExternalPrecipitationRate(...request.args) ?? null; break;
        case "rainGageRainfallOverride": result = simulation.rainGageRainfallOverride(...request.args) ?? null; break;
        case "subcatchmentPrecipitationScaleFactors": result = simulation.subcatchmentPrecipitationScaleFactors(...request.args); break;
        case "setSubcatchmentPrecipitationScaleFactors": simulation.setSubcatchmentPrecipitationScaleFactors(...request.args); break;
        case "subcatchmentExternalForcing": result = simulation.subcatchmentExternalForcing(...request.args); break;
        case "setSubcatchmentExternalRainfall": simulation.setSubcatchmentExternalRainfall(...request.args); break;
        case "setSubcatchmentExternalSnowfall": simulation.setSubcatchmentExternalSnowfall(...request.args); break;
        case "subcatchmentQuality": result = simulation.subcatchmentQuality(...request.args); break;
        case "subcatchmentQualitySnapshot": result = simulation.subcatchmentQualitySnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "subcatchmentNamedSettings": result = simulation.subcatchmentNamedSettings(...request.args); break;
        case "subcatchmentNamedSetting": result = simulation.subcatchmentNamedSetting(...request.args); break;
        case "updateSubcatchmentNamedSettings": simulation.updateSubcatchmentNamedSettings(...request.args); break;
        case "subcatchmentExternalPollutantBuildupIncrement": result = simulation.subcatchmentExternalPollutantBuildupIncrement(...request.args); break;
        case "subcatchmentExternalPollutantBuildupIncrementValue": result = simulation.subcatchmentExternalPollutantBuildupIncrementValue(...request.args); break;
        case "updateSubcatchmentExternalPollutantBuildupIncrement": simulation.updateSubcatchmentExternalPollutantBuildupIncrement(...request.args); break;
        case "subcatchmentStatistics": result = simulation.subcatchmentStatistics(...request.args); break;
        case "subcatchmentStatisticsSnapshot": result = simulation.subcatchmentStatisticsSnapshot(request.args[0] === undefined ? undefined : [...request.args[0]]); break;
        case "subcatchmentConfiguration": result = simulation.subcatchmentConfiguration(...request.args); break;
        case "configureNode": simulation.configureNode(...request.args); break;
        case "configureLink": simulation.configureLink(...request.args); break;
        case "configureSubcatchment": simulation.configureSubcatchment(...request.args); break;
        case "setNodeExternalInflow": simulation.setNodeExternalInflow(...request.args); break;
        case "setLinkTargetSetting": simulation.setLinkTargetSetting(...request.args); break;
        case "setOutfallStage": simulation.setOutfallStage(...request.args); break;
        case "setRainGagePrecipitation": simulation.setRainGagePrecipitation(...request.args); break;
        case "setRainGageRainfallOverride": simulation.setRainGageRainfallOverride(...request.args); break;
        case "options": result = simulation.options(); break;
        case "configureOptions": simulation.configureOptions(...request.args); break;
        case "updateSchedule": simulation.updateSchedule(...request.args); break;
        case "maximumRoutingStepSeconds": result = simulation.maximumRoutingStepSeconds(); break;
        case "ammModelConfiguration": result = simulation.ammModelConfiguration(...request.args); break;
        case "configureAmmModel": simulation.configureAmmModel(...request.args); break;
        case "ammAssignments": result = simulation.ammAssignments(); break;
        case "replaceAmmAssignments": simulation.replaceAmmAssignments(...request.args); break;
        case "unitHydrographConfiguration": result = simulation.unitHydrographConfiguration(...request.args); break;
        case "configureUnitHydrograph": simulation.configureUnitHydrograph(...request.args); break;
        case "rdiiAssignments": result = simulation.rdiiAssignments(); break;
        case "replaceRdiiAssignments": simulation.replaceRdiiAssignments(...request.args); break;
        case "aquiferConfiguration": result = simulation.aquiferConfiguration(...request.args); break;
        case "configureAquifer": simulation.configureAquifer(...request.args); break;
        case "snowmeltConfiguration": result = simulation.snowmeltConfiguration(...request.args); break;
        case "configureSnowmelt": simulation.configureSnowmelt(...request.args); break;
        case "snowmeltSurfaceConfiguration": result = simulation.snowmeltSurfaceConfiguration(...request.args); break;
        case "configureSnowmeltSurface": simulation.configureSnowmeltSurface(...request.args); break;
        case "lidControlConfiguration": result = simulation.lidControlConfiguration(...request.args); break;
        case "configureLidControl": simulation.configureLidControl(...request.args); break;
        case "lidUnitCount": result = simulation.lidUnitCount(...request.args); break;
        case "lidUnitConfiguration": result = simulation.lidUnitConfiguration(...request.args); break;
        case "configureLidUnit": simulation.configureLidUnit(...request.args); break;
        case "lidUnitSnapshot": result = simulation.lidUnitSnapshot(...request.args); break;
        case "subcatchmentLidSnapshot": result = simulation.subcatchmentLidSnapshot(...request.args); break;
        case "useHotstart": simulation.useHotstart(...request.args); break;
        case "saveHotstart": result = simulation.saveHotstart(); break;
        case "exportCheckpoint": result = simulation.exportCheckpoint(); break;
        case "loadCheckpointState": simulation.loadCheckpointState(...request.args); break;
        case "statistics": result = simulation.statistics(); break;
        case "finish": simulation.finish(); result = results(simulation); break;
        case "run": simulation.run(...request.args); result = results(simulation); break;
        case "end": simulation.end(); break;
        case "finalizeReport": simulation.finalizeReport(); break;
        case "report": simulation.report(); break;
        case "resetSolver": simulation.resetSolver(); break;
        case "sleepWorkers": simulation.sleepWorkers(); break;
        case "readFile": result = simulation.readFile(...request.args); break;
        default: {
          const unhandled: never = request;
          throw new Error(`Unknown simulation operation: ${String(unhandled)}`);
        }
      }
    }
    const transfers: ArrayBuffer[] = [];
    if (result instanceof Uint8Array) transfers.push(result.buffer as ArrayBuffer);
    else if (result && typeof result === "object") {
      if ("output" in result && result.output instanceof Uint8Array) transfers.push(result.output.buffer as ArrayBuffer);
      if (request.method === "exportCheckpoint") {
        const checkpoint = result as CheckpointBundle;
        transfers.push(checkpoint.manifest.buffer as ArrayBuffer);
        for (const bytes of Object.values(checkpoint.files)) transfers.push(bytes.buffer as ArrayBuffer);
      }
    }
    host.postMessage({ id, ok: true, result }, transfers);
  } catch (error) {
    const fatal = error instanceof WebAssembly.RuntimeError || method === "open" || method === "resumeCheckpoint" || method === "openOutput" || method === "close" || method === "closeOutput";
    const source = error && typeof error === "object" ? error as Record<string, unknown> : {};
    const detail: ErrorDetails = {
      message: error instanceof Error ? error.message : String(error),
      code: typeof source.code === "number" ? source.code : error instanceof WebAssembly.RuntimeError ? 9999 : undefined,
      operation: typeof source.operation === "string" ? source.operation : method,
      detail: typeof source.detail === "string" ? source.detail : undefined,
      semanticCode: typeof source.semanticCode === "string" ? source.semanticCode : undefined,
      category: typeof source.category === "string" ? source.category : undefined,
      diagnostics: Array.isArray(source.diagnostics) ? source.diagnostics as ConfigurationDiagnostic[] : undefined,
      report: typeof source.report === "string" ? source.report : undefined,
      cleanupError: typeof source.cleanupError === "string" ? source.cleanupError : undefined,
    };
    if (fatal && method !== "close") {
      try { shutdown(); } catch (cleanupError) { detail.cleanupError = String(cleanupError); }
    }
    host.postMessage({ id, ok: false, error: detail, fatal });
  }
}

// Native calls are synchronous. Initialization must finish before later requests.
host.onmessage = ({ data }) => { queue = queue.then(() => dispatch(data)); };
