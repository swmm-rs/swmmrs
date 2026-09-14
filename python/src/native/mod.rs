pub(super) use std::collections::HashMap;
pub(super) use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(feature = "test-support")]
pub(super) use std::sync::Condvar;
pub(super) use std::sync::Mutex;
pub(super) use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(feature = "test-support")]
pub(super) use std::time::Duration;

pub(super) use pyo3::IntoPyObjectExt;
pub(super) use pyo3::exceptions::{PyIndexError, PyKeyError, PyRuntimeError, PyTypeError};
pub(super) use pyo3::prelude::*;
pub(super) use pyo3::sync::{MutexExt, PyOnceLock};
pub(super) use pyo3::types::{PyAny, PyBool, PyDict, PyFloat, PyInt, PyString, PyTuple, PyType};
pub(super) use swmmrs::engine::consts;
pub(super) use swmmrs::engine::datetime::{
    DateTime, datetime_decodeDate, datetime_decodeTime, datetime_encodeDate, datetime_encodeTime,
};
pub(super) use swmmrs::engine::enums::{
    CustomEllipseModel, FlowUnitsType, InertialDampingType, LinkType, NodeType, NormalFlowType,
    ObjectType, OrificeType, RoadSurface, StorageType, SurchargeMethodType, UnitsType, WeirType,
    XsectType,
};
pub(super) use swmmrs::engine::error::{ErrorCode, SwmmError};
pub(super) use swmmrs::simulation::lids::{
    LidControlField, LidControlRead, LidControlValue, LidDrainDestination, LidDrainDestinationRead,
    LidDrainPatch, LidDrainageMatPatch, LidLayerPatch, LidPavementPatch, LidSoilPatch,
    LidStoragePatch, LidSurfacePatch, LidUnitField, LidUnitPatch, LidUnitSnapshotRead,
    LidUnitValue, SubcatchmentLidSnapshotRead,
};
pub(super) use swmmrs::simulation::links::{
    ConduitPatch, CrossSectionConfiguration, InletConfigurationField, InletConfigurationValue,
    InletPersistentPatch, InletResultField, LinkConfigurationField, LinkConfigurationValue,
    LinkCrossSectionValue, LinkOutletRatingValue, LinkPatch, LinkResultField, LinkSnapshotRead,
    LinkSubtypePatch, OrificeUpdate, OutletHeadBasis, OutletRating, OutletUpdate,
    PumpConfiguration, PumpUpdate, WeirUpdate,
};
pub(super) use swmmrs::simulation::nodes::{
    CanonicalStorageShape, DividerMode, DividerRuleValue, DividerUpdate, NodeConfigurationField,
    NodeConfigurationValue, NodePatch, NodeResultField, NodeSnapshotRead, NodeSubtypePatch,
    OutfallBoundary, OutfallBoundaryValue, OutfallUpdate, StorageExfiltration, StorageUpdate,
};
pub(super) use swmmrs::simulation::options::{
    SimulationOptionField, SimulationOptionValue, SimulationOptionsPatch, SimulationSchedulePatch,
    SimulationTimeField,
};
pub(super) use swmmrs::simulation::quality::{
    LinkQualityRead, LinkQualitySnapshotRead, NodeQualityRead, NodeQualitySnapshotRead,
    PersistentPollutantValuesPatch, PollutantMappingRead, PollutantValuesPatch,
    SubcatchmentQualityRead, SubcatchmentQualitySnapshotRead,
};
pub(super) use swmmrs::simulation::reads::{
    ObjectIdentityRead, ObjectSubtypeRead, SimulationOptionsRead,
};
pub(super) use swmmrs::simulation::statistics::{
    LinkStatisticsRead, LinkStatisticsSnapshotRead, NodeStatisticsRead, NodeStatisticsSnapshotRead,
    OutfallStatisticsRead, PumpStatisticsRead, QualityBalanceRead, RoutingDiagnosticsRead,
    RoutingTotalsRead, RunoffTotalsRead, SimulationStatisticsRead, StorageStatisticsRead,
    SubcatchmentStatisticsRead, SubcatchmentStatisticsSnapshotRead,
};
pub(super) use swmmrs::simulation::subcatchments::{
    AquiferConfigurationField, AquiferConfigurationValue, AquiferPatch, SnowmeltConfigurationField,
    SnowmeltConfigurationValue, SnowmeltPatch, SnowmeltSurface, SnowmeltSurfaceConfiguration,
    SnowmeltSurfaceField, SubcatchmentConfigurationField, SubcatchmentConfigurationValue,
    SubcatchmentCoveragePatch, SubcatchmentGroundwaterConfiguration,
    SubcatchmentGroundwaterConfigurationRead, SubcatchmentInfiltrationConfiguration,
    SubcatchmentLoadingPatch, SubcatchmentNamedSettings, SubcatchmentOutletRead, SubcatchmentPatch,
    SubcatchmentPrecipitationScaleFactors, SubcatchmentResultField, SubcatchmentSnapshotRead,
    SubcatchmentSnowpackConfiguration,
};
pub(super) use swmmrs::simulation::{
    AmmAssignmentConfiguration, AmmComponentConfiguration, AmmModelPatch, ConfigurationDiagnostic,
    RdiiAssignmentConfiguration, SimulationLifecycle, SwmmSimulation, UnitHydrographPatch,
    UnitHydrographResponseConfiguration,
};

mod amm;
mod lids;
mod lifecycle;
mod links;
mod marshal;
mod nodes;
mod output;
mod owner;
mod records;
mod rtk;
mod statistics;
mod subcatchments;
mod support;

use marshal::*;
pub(crate) use output::NativeOutputReader;
pub(crate) use owner::NativeSimulation;
use owner::*;
pub(crate) use records::register_records;
use records::*;
pub(crate) use support::bind_public_types;
use support::*;
