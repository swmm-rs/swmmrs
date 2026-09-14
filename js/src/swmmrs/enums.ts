/** Owner lifecycle state. A type-only union, not a runtime enum. */
export type SimulationState = "open" | "running" | "complete" | "ended" | "failed" | "closed";
/** Project flow units: ft³/s, US gal/min, million US gal/day, m³/s, L/s, or million L/day. */
export type FlowUnits = "Cfs" | "Gpm" | "Mgd" | "Cms" | "Lps" | "Mld";
/** No routing, steady flow, kinematic wave, extended kinematic wave, or Dynamic Wave. */
export type RoutingModel = "NoRouting" | "Sf" | "Kw" | "Ekw" | "Dw";
/** Configured node kind. A type-only string union, not a JavaScript enum object. */
export type NodeKind = "Junction" | "Outfall" | "Storage" | "Divider";
/** Configured hydraulic link family; handles do not have public subtype subclasses. */
export type LinkKind = "Conduit" | "Pump" | "Orifice" | "Weir" | "Outlet";
/** Model calendar time, YYYY-MM-DDTHH:mm:ss, with no timezone. */
export type ModelTime = string;
/** Model unit system: US customary or SI. */
export type UnitSystem = "us" | "si";
/** Legacy EPA ellipse interpretation or true-ellipse geometry. */
export type CustomEllipseModel = "epa_legacy" | "true_ellipse";
/** EXTRAN surcharge treatment or the slot method. */
export type SurchargeMethod = "extran" | "slot";
/** Dynamic Wave inertial-term damping. */
export type InertiaDamping = "none" | "partial" | "full";
/** Criterion for limiting flow to the normal-flow value. */
export type NormalFlowLimit = "slope" | "froude" | "both" | "neither";
/** Side or bottom orifice orientation. */
export type OrificeKind = "side" | "bottom";
/** Supported weir geometries. */
export type WeirKind = "transverse" | "sideflow" | "v_notch" | "trapezoidal" | "roadway";
/** Roadway-weir surface used by its discharge relationship. */
export type RoadSurface = "unspecified" | "paved" | "gravel";
/** Outlet rating uses upstream depth or hydraulic head difference. */
export type OutletHeadBasis = "depth" | "head";
/** Native standard cross-section shape names; geometry interpretation depends on the shape. */
export type StandardCrossSectionShape =
  | "dummy" | "circular" | "filled_circular" | "rect_closed" | "rect_open"
  | "trapezoidal" | "triangular" | "parabolic" | "power_function" | "rect_triangular"
  | "rect_round" | "modified_basket" | "horizontal_ellipse" | "vertical_ellipse"
  | "arch" | "egg_shaped" | "horseshoe" | "gothic" | "catenary" | "semi_elliptical"
  | "basket_handle" | "semi_circular" | "force_main";
/** Supported Horton, Green-Ampt, and curve-number infiltration models. */
export type InfilKind = "horton" | "modified_horton" | "green_ampt" | "modified_green_ampt" | "curve_number";
/** Receiving object family for a subcatchment outlet. */
export type OutKind = "node" | "subcatchment";
/** Seven hydraulic flow classes, in the same order used by link statistics duration arrays. */
export type FlowClass = "dry" | "upstream_dry" | "downstream_dry" | "subcritical" | "supercritical" | "upstream_critical" | "downstream_critical";
/** Known semantic solver categories. Numeric native codes remain available on errors. */
export type SolverErrorCode =
  | "memory" | "kinwave" | "ode_solver" | "timestep" | "subcatch_outlet"
  | "aquifer_params" | "ground_elev" | "length" | "elev_drop" | "roughness" | "barrels"
  | "slope" | "no_xsect" | "xsect" | "no_curve" | "pump_limits" | "loop" | "multi_outlet"
  | "dummy_link" | "divider" | "divider_link" | "weir_divider" | "node_depth" | "regulator"
  | "storage_volume" | "outfall" | "regulator_shape" | "no_outlets" | "unithyd_times"
  | "unithyd_ratios" | "rdii_area" | "rain_file_conflict" | "rain_gage_format"
  | "rain_gage_tseries" | "rain_gage_interval" | "cyclic_treatment" | "curve_sequence"
  | "timeseries_sequence" | "snowmelt_params" | "snowpack_params" | "lid_type"
  | "lid_layer" | "lid_params" | "lid_areas" | "lid_capture_area" | "start_date"
  | "report_date" | "report_step" | "input" | "line_length" | "items" | "keyword"
  | "dup_name" | "name" | "number" | "datetime" | "rule" | "transect_unknown"
  | "transect_sequence" | "transect_too_few" | "transect_too_many" | "transect_manning"
  | "transect_overbank" | "transect_no_depth" | "math_expr" | "infil_params" | "file_name"
  | "inp_file" | "rpt_file" | "out_file" | "out_size" | "out_write" | "out_read"
  | "rain_file_scratch" | "rain_file_open" | "rain_file_data" | "rain_file_sequence"
  | "rain_file_format" | "rain_iface_format" | "rain_file_gage" | "runoff_file_open"
  | "runoff_file_format" | "runoff_file_end" | "runoff_file_read" | "hotstart_file_open"
  | "hotstart_file_format" | "hotstart_file_read" | "no_climate_file" | "climate_file_open"
  | "climate_file_read" | "climate_end_of_file" | "rdii_file_scratch" | "rdii_file_open"
  | "rdii_file_format" | "routing_file_open" | "routing_file_format" | "routing_file_nomatch"
  | "routing_file_names" | "table_file_open" | "table_file_read" | "checkpoint_invalid"
  | "checkpoint_unsupported_feature" | "checkpoint_integrity" | "checkpoint_external_dependency"
  | "checkpoint_sidecar_validation" | "checkpoint_rebuild" | "checkpoint_compatibility"
  | "checkpoint_decode" | "checkpoint_source_identity" | "checkpoint_destination_open"
  | "checkpoint_destination_validation" | "checkpoint_source_flush" | "checkpoint_source_preflight"
  | "checkpoint_sidecar_copy" | "checkpoint_rain_resource" | "checkpoint_sidecar_absent"
  | "checkpoint_rdii_resource" | "checkpoint_routing_interface" | "checkpoint_runoff_interface"
  | "checkpoint_read" | "checkpoint_append_role" | "system";
