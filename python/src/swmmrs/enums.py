"""Typed string enumerations used by the public SWMM Python API."""

from enum import StrEnum as _StrEnum
from typing import final as _final


@_final
class SimulationState(_StrEnum):
    """Represent the authoritative simulation lifecycle state.

    Notes
    -----
    `Simulation.state` returns this enum. API operations validate the current
    state before invoking the native solver.
    """

    OPEN = "open"
    RUNNING = "running"
    COMPLETE = "complete"
    ENDED = "ended"
    FAILED = "failed"
    CLOSED = "closed"


@_final
class CustomEllipseModel(_StrEnum):
    """Select hydraulics for custom-sized elliptical conduits."""

    EPA_LEGACY = "epa_legacy"
    TRUE_ELLIPSE = "true_ellipse"


@_final
class FlowUnits(_StrEnum):
    """Represent supported project flow-unit codes."""

    CFS = "cfs"
    GPM = "gpm"
    MGD = "mgd"
    CMS = "cms"
    LPS = "lps"
    MLD = "mld"


@_final
class UnitSystem(_StrEnum):
    """Represent the project's US customary or SI unit system."""

    US = "us"
    SI = "si"


@_final
class SurchargeMethod(_StrEnum):
    """Represent Dynamic Wave surcharge treatment methods."""

    EXTRAN = "extran"
    SLOT = "slot"


@_final
class InertiaDamping(_StrEnum):
    """Represent Dynamic Wave inertial damping methods."""

    NONE = "none"
    PARTIAL = "partial"
    FULL = "full"


@_final
class NormalFlowLimit(_StrEnum):
    """Represent normal-flow limiting modes."""

    SLOPE = "slope"
    FROUDE = "froude"
    BOTH = "both"
    NEITHER = "neither"


@_final
class NodeKind(_StrEnum):
    """Represent configured node subtypes."""

    JUNCTION = "junction"
    OUTFALL = "outfall"
    STORAGE = "storage"
    DIVIDER = "divider"


@_final
class LinkKind(_StrEnum):
    """Represent configured link subtypes."""

    CONDUIT = "conduit"
    PUMP = "pump"
    ORIFICE = "orifice"
    WEIR = "weir"
    OUTLET = "outlet"


@_final
class OrificeKind(_StrEnum):
    """Represent side- and bottom-oriented orifice links."""

    SIDE = "side"
    BOTTOM = "bottom"


@_final
class WeirKind(_StrEnum):
    """Represent supported weir geometry and flow families."""

    TRANSVERSE = "transverse"
    SIDEFLOW = "sideflow"
    V_NOTCH = "v_notch"
    TRAPEZOIDAL = "trapezoidal"
    ROADWAY = "roadway"


@_final
class RoadSurface(_StrEnum):
    """Represent roadway-weir surface materials."""

    UNSPECIFIED = "unspecified"
    PAVED = "paved"
    GRAVEL = "gravel"


@_final
class OutletHeadBasis(_StrEnum):
    """Represent the independent head used by an outlet rating."""

    DEPTH = "depth"
    HEAD = "head"


@_final
class StandardCrossSectionShape(_StrEnum):
    """Represent non-reference SWMM cross-section shapes."""

    DUMMY = "dummy"
    CIRCULAR = "circular"
    FILLED_CIRCULAR = "filled_circular"
    RECT_CLOSED = "rect_closed"
    RECT_OPEN = "rect_open"
    TRAPEZOIDAL = "trapezoidal"
    TRIANGULAR = "triangular"
    PARABOLIC = "parabolic"
    POWER_FUNCTION = "power_function"
    RECT_TRIANGULAR = "rect_triangular"
    RECT_ROUND = "rect_round"
    MODIFIED_BASKET = "modified_basket"
    HORIZONTAL_ELLIPSE = "horizontal_ellipse"
    VERTICAL_ELLIPSE = "vertical_ellipse"
    ARCH = "arch"
    EGG_SHAPED = "egg_shaped"
    HORSESHOE = "horseshoe"
    GOTHIC = "gothic"
    CATENARY = "catenary"
    SEMI_ELLIPTICAL = "semi_elliptical"
    BASKET_HANDLE = "basket_handle"
    SEMI_CIRCULAR = "semi_circular"
    FORCE_MAIN = "force_main"


@_final
class InfilKind(_StrEnum):
    """Represent configured subcatchment infiltration models."""

    HORTON = "horton"
    MODIFIED_HORTON = "modified_horton"
    GREEN_AMPT = "green_ampt"
    MODIFIED_GREEN_AMPT = "modified_green_ampt"
    CURVE_NUMBER = "curve_number"


@_final
class OutKind(_StrEnum):
    """Represent configured subcatchment outlet relationship kinds."""

    NODE = "node"
    SUBCATCHMENT = "subcatchment"


@_final
class FlowClass(_StrEnum):
    """Represent current hydraulic flow classifications."""

    DRY = "dry"
    UPSTREAM_DRY = "upstream_dry"
    DOWNSTREAM_DRY = "downstream_dry"
    SUBCRITICAL = "subcritical"
    SUPERCRITICAL = "supercritical"
    UPSTREAM_CRITICAL = "upstream_critical"
    DOWNSTREAM_CRITICAL = "downstream_critical"


@_final
class SolverErrorCode(_StrEnum):
    """Represent stable native solver, project, and file failure categories.

    Notes
    -----
    `SolverError.code` uses this enum. `message` supplies a human-readable
    description suitable for diagnostics.
    """

    MEMORY = "memory"
    KINWAVE = "kinwave"
    ODE_SOLVER = "ode_solver"
    TIMESTEP = "timestep"
    SUBCATCH_OUTLET = "subcatch_outlet"
    AQUIFER_PARAMS = "aquifer_params"
    GROUND_ELEV = "ground_elev"
    LENGTH = "length"
    ELEV_DROP = "elev_drop"
    ROUGHNESS = "roughness"
    BARRELS = "barrels"
    SLOPE = "slope"
    NO_XSECT = "no_xsect"
    XSECT = "xsect"
    NO_CURVE = "no_curve"
    PUMP_LIMITS = "pump_limits"
    LOOP = "loop"
    MULTI_OUTLET = "multi_outlet"
    DUMMY_LINK = "dummy_link"
    DIVIDER = "divider"
    DIVIDER_LINK = "divider_link"
    WEIR_DIVIDER = "weir_divider"
    NODE_DEPTH = "node_depth"
    REGULATOR = "regulator"
    STORAGE_VOLUME = "storage_volume"
    OUTFALL = "outfall"
    REGULATOR_SHAPE = "regulator_shape"
    NO_OUTLETS = "no_outlets"
    UNITHYD_TIMES = "unithyd_times"
    UNITHYD_RATIOS = "unithyd_ratios"
    RDII_AREA = "rdii_area"
    RAIN_FILE_CONFLICT = "rain_file_conflict"
    RAIN_GAGE_FORMAT = "rain_gage_format"
    RAIN_GAGE_TSERIES = "rain_gage_tseries"
    RAIN_GAGE_INTERVAL = "rain_gage_interval"
    CYCLIC_TREATMENT = "cyclic_treatment"
    CURVE_SEQUENCE = "curve_sequence"
    TIMESERIES_SEQUENCE = "timeseries_sequence"
    SNOWMELT_PARAMS = "snowmelt_params"
    SNOWPACK_PARAMS = "snowpack_params"
    LID_TYPE = "lid_type"
    LID_LAYER = "lid_layer"
    LID_PARAMS = "lid_params"
    LID_AREAS = "lid_areas"
    LID_CAPTURE_AREA = "lid_capture_area"
    START_DATE = "start_date"
    REPORT_DATE = "report_date"
    REPORT_STEP = "report_step"
    INPUT = "input"
    LINE_LENGTH = "line_length"
    ITEMS = "items"
    KEYWORD = "keyword"
    DUP_NAME = "dup_name"
    NAME = "name"
    NUMBER = "number"
    DATETIME = "datetime"
    RULE = "rule"
    TRANSECT_UNKNOWN = "transect_unknown"
    TRANSECT_SEQUENCE = "transect_sequence"
    TRANSECT_TOO_FEW = "transect_too_few"
    TRANSECT_TOO_MANY = "transect_too_many"
    TRANSECT_MANNING = "transect_manning"
    TRANSECT_OVERBANK = "transect_overbank"
    TRANSECT_NO_DEPTH = "transect_no_depth"
    MATH_EXPR = "math_expr"
    INFIL_PARAMS = "infil_params"
    FILE_NAME = "file_name"
    INP_FILE = "inp_file"
    RPT_FILE = "rpt_file"
    OUT_FILE = "out_file"
    OUT_SIZE = "out_size"
    OUT_WRITE = "out_write"
    OUT_READ = "out_read"
    RAIN_FILE_SCRATCH = "rain_file_scratch"
    RAIN_FILE_OPEN = "rain_file_open"
    RAIN_FILE_DATA = "rain_file_data"
    RAIN_FILE_SEQUENCE = "rain_file_sequence"
    RAIN_FILE_FORMAT = "rain_file_format"
    RAIN_IFACE_FORMAT = "rain_iface_format"
    RAIN_FILE_GAGE = "rain_file_gage"
    RUNOFF_FILE_OPEN = "runoff_file_open"
    RUNOFF_FILE_FORMAT = "runoff_file_format"
    RUNOFF_FILE_END = "runoff_file_end"
    RUNOFF_FILE_READ = "runoff_file_read"
    HOTSTART_FILE_OPEN = "hotstart_file_open"
    HOTSTART_FILE_FORMAT = "hotstart_file_format"
    HOTSTART_FILE_READ = "hotstart_file_read"
    NO_CLIMATE_FILE = "no_climate_file"
    CLIMATE_FILE_OPEN = "climate_file_open"
    CLIMATE_FILE_READ = "climate_file_read"
    CLIMATE_END_OF_FILE = "climate_end_of_file"
    RDII_FILE_SCRATCH = "rdii_file_scratch"
    RDII_FILE_OPEN = "rdii_file_open"
    RDII_FILE_FORMAT = "rdii_file_format"
    ROUTING_FILE_OPEN = "routing_file_open"
    ROUTING_FILE_FORMAT = "routing_file_format"
    ROUTING_FILE_NOMATCH = "routing_file_nomatch"
    ROUTING_FILE_NAMES = "routing_file_names"
    TABLE_FILE_OPEN = "table_file_open"
    TABLE_FILE_READ = "table_file_read"
    CHECKPOINT_INVALID = "checkpoint_invalid"
    CHECKPOINT_UNSUPPORTED_FEATURE = "checkpoint_unsupported_feature"
    CHECKPOINT_INTEGRITY = "checkpoint_integrity"
    CHECKPOINT_EXTERNAL_DEPENDENCY = "checkpoint_external_dependency"
    CHECKPOINT_SIDECAR_VALIDATION = "checkpoint_sidecar_validation"
    CHECKPOINT_REBUILD = "checkpoint_rebuild"
    CHECKPOINT_COMPATIBILITY = "checkpoint_compatibility"
    CHECKPOINT_DECODE = "checkpoint_decode"
    CHECKPOINT_SOURCE_IDENTITY = "checkpoint_source_identity"
    CHECKPOINT_DESTINATION_OPEN = "checkpoint_destination_open"
    CHECKPOINT_DESTINATION_VALIDATION = "checkpoint_destination_validation"
    CHECKPOINT_SOURCE_FLUSH = "checkpoint_source_flush"
    CHECKPOINT_SOURCE_PREFLIGHT = "checkpoint_source_preflight"
    CHECKPOINT_SIDECAR_COPY = "checkpoint_sidecar_copy"
    CHECKPOINT_RAIN_RESOURCE = "checkpoint_rain_resource"
    CHECKPOINT_SIDECAR_ABSENT = "checkpoint_sidecar_absent"
    CHECKPOINT_RDII_RESOURCE = "checkpoint_rdii_resource"
    CHECKPOINT_ROUTING_INTERFACE = "checkpoint_routing_interface"
    CHECKPOINT_RUNOFF_INTERFACE = "checkpoint_runoff_interface"
    CHECKPOINT_READ = "checkpoint_read"
    CHECKPOINT_APPEND_ROLE = "checkpoint_append_role"
    SYSTEM = "system"

    @property
    def message(self) -> str:
        """Return the stable human-readable semantic category."""

        return self.value.replace("_", " ")


__all__ = (
    "CustomEllipseModel",
    "FlowClass",
    "FlowUnits",
    "InertiaDamping",
    "InfilKind",
    "LinkKind",
    "NodeKind",
    "NormalFlowLimit",
    "OrificeKind",
    "OutKind",
    "OutletHeadBasis",
    "RoadSurface",
    "StandardCrossSectionShape",
    "SurchargeMethod",
    "SimulationState",
    "SolverErrorCode",
    "UnitSystem",
    "WeirKind",
)
