# Exceptions

All package-specific simulation exceptions inherit from `SwmmError`. Catch the
narrowest type your application can handle. `ValidationError` and
`LifecycleError` leave a healthy owner reusable. `ConfigurationError` keeps
ordered `.diagnostics` so you can repair the configuration and retry. An
operational `SolverError` can put the generation in `FAILED`.

Native `SwmmError` instances expose `.native_code`, `.operation`, `.detail`,
and `.semantic_code`. `SolverError.code` remains a `SolverErrorCode`; its
`.semantic_code` retains the more specific path code when one exists. Secondary
cleanup failures retained by the solver are included in `.detail`.

Each `ConfigurationDiagnostic` contains `object`, `property_path`, `rule_code`,
`message`, and optional `conflicting_object`. `str(ConfigurationError)` includes
all ordered diagnostic lines for ordinary tracebacks. See [Errors and
recovery](../concepts/errors-and-recovery.md) for handling examples. Finalized
binary output uses the separate `swmmrs.output.OutputError` hierarchy.

::: swmmrs.exceptions
    options:
        show_root_heading: false
