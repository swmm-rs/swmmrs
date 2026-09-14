/** Native identity attached to a configuration diagnostic, not a lookup handle. */
export interface ConfigurationObjectIdentity {
  /** Native object-family name. */
  readonly objectType: string;
  /** Canonical object ID. */
  readonly id: string;
  /** Native configured-order index for diagnostics. */
  readonly index: number;
}
/** One rejected configuration rule; the complete patch remains unapplied. */
export interface ConfigurationDiagnostic {
  /** Object whose declaration failed validation. */
  readonly object: ConfigurationObjectIdentity;
  /** Property path identifying the rejected value. */
  readonly propertyPath: string;
  /** Stable native validation-rule identifier. */
  readonly ruleCode: string;
  /** Human-readable explanation. */
  readonly message: string;
  /** Other object involved in the conflict, or null. */
  readonly conflictingObject: ConfigurationObjectIdentity | null;
}

/** Details retained when an error crosses the worker boundary. */
export interface ErrorDetails {
  /** Required human-readable error text. */
  message: string;
  /** Native numeric code, when supplied. */
  code?: number | undefined;
  /** Operation that failed. */
  operation?: string | undefined;
  /** Additional native detail. */
  detail?: string | undefined;
  /** Semantic category, preserving unknown future values. */
  semanticCode?: string | undefined;
  /** Native failure category. */
  category?: string | undefined;
  /** Configuration diagnostics; presence takes precedence over numeric-code mapping. */
  diagnostics?: readonly ConfigurationDiagnostic[] | undefined;
  /** Partial report text, when available. */
  report?: string | undefined;
  /** Serialized secondary cleanup failure, when supplied by the worker. */
  cleanupError?: string | undefined;
}

/** Base runtime exception for binding and solver failures; supports `instanceof`.
 * Ordinary Error message, name, and stack are retained. Standalone output failures
 * use `OutputError`, not this hierarchy.
 */
export class SwmmError extends Error {
  /** Native numeric failure code, when available. */
  readonly code: number | undefined;
  /** Failed public operation, when available. */
  readonly operation: string | undefined;
  /** Additional native failure detail. */
  readonly detail: string | undefined;
  /** Preserves unknown future categories rather than discarding them. */
  readonly semanticCode: string | undefined;
  /** Native error category. */
  readonly category: string | undefined;
  /** Partial report text, when available. */
  readonly report: string | undefined;
  /** Secondary cleanup failure retained after a primary run failure. */
  cleanupError: unknown;

  /** Construct an error with serialized or caller-supplied details.
   * @param details - Message and optional structured failure metadata.
   */
  constructor(details: ErrorDetails) {
    super(details.message);
    this.name = new.target.name;
    this.code = details.code;
    this.operation = details.operation;
    this.detail = details.detail;
    this.semanticCode = details.semanticCode;
    this.category = details.category;
    this.report = details.report;
    this.cleanupError = details.cleanupError;
  }
}
/** Native failure without a more specific binding category, including unmapped future codes. */
export class SolverError extends SwmmError {}
/** Invalid phase, closed owner, or competing advancement. Native codes: 501, 502, 503, 2001, 2002, 2013. */
export class LifecycleError extends SwmmError {}
/** Retained owner-bound view no longer matches the live owner. */
export class StaleViewError extends LifecycleError {}
/** Invalid value or read. Native codes: 504, 507, 508, 509, 2003, 2006, 2010, 2012. */
export class ValidationError extends SwmmError {}
/** Configuration rejected with object-level diagnostics; takes precedence over code-based categorization. */
export class ConfigurationError extends ValidationError {
  /** Frozen diagnostic array with frozen object identities. */
  readonly diagnostics: readonly ConfigurationDiagnostic[];
  /** Construct a configuration failure and defensively freeze its diagnostics.
   * @param details - Error metadata and optional diagnostic list (defaults to empty).
   */
  constructor(details: ErrorDetails) {
    super(details);
    this.diagnostics = Object.freeze((details.diagnostics ?? []).map(diagnostic => Object.freeze({
      ...diagnostic,
      object: Object.freeze({ ...diagnostic.object }),
      conflictingObject: diagnostic.conflictingObject === null ? null : Object.freeze({ ...diagnostic.conflictingObject }),
    })));
  }
}
/** Unknown object/definition ID. Native codes: 505, 506, 2000, 2004, 2005, 2007, 2008, 2009. */
export class ObjectNotFoundError extends SwmmError {}
/** Worker startup, communication, or runtime prerequisite failure. */
export class WorkerError extends SwmmError {}
/** Internal native simulation failure, mapped from code 9999. */
export class InternalSimulationError extends WorkerError {}

/** @internal */
export function fromWorkerError(details: ErrorDetails): SwmmError {
  if (details.diagnostics !== undefined) return new ConfigurationError(details);
  if ([501, 502, 503, 2001, 2002, 2013].includes(details.code ?? -1)) return new LifecycleError(details);
  if ([505, 506, 2000, 2004, 2005, 2007, 2008, 2009].includes(details.code ?? -1)) return new ObjectNotFoundError(details);
  if ([504, 507, 508, 509, 2003, 2006, 2010, 2012].includes(details.code ?? -1)) return new ValidationError(details);
  if (details.code === 9999) return new InternalSimulationError(details);
  return new SolverError(details);
}
