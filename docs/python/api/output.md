# Binary output

Open a SWMM binary output file with `OutputReader` to inspect its metadata and
read immutable result series. It detects finalized files and recovers complete
records when the final trailer is missing. An interrupted run may still have
useful results to offer.

The reader exposes exact Stored Report Dates and a shared Nominal Report Date
axis. It wraps the standalone `swmm-output` reader and works independently of a
live `Simulation`.

::: swmmrs.output
    options:
        show_root_heading: false
        members:
            - OutputReader
            - BulkSeriesResult
            - OutputValueSeries
            - OutputTimeSeries
            - SeriesSelection
            - PollutantAttribute
            - OutputName
            - OutputMetadata
            - ResultSchema
            - ReportTiming
            - SubcatchmentMetadata
            - NodeMetadata
            - LinkMetadata
            - PollutantMetadata
            - ResultElementType
            - SubcatchmentResultAttribute
            - NodeResultAttribute
            - LinkResultAttribute
            - SystemResultAttribute
            - ResultAttributeCode
            - ConcentrationUnits
            - FlowUnits
            - UnitSystem
            - NodeKind
            - LinkKind
            - RunStatus
            - UnknownCode
            - OutputError
