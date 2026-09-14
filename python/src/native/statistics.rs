use super::*;

#[pymethods]
impl NativeSimulation {
    /// Copies one node statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Python-owned statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn node_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<NodeStatistics>> {
        let values = self.checked_view(
            py,
            "node_statistics",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_statistics_read(index),
        )?;
        Py::new(py, NodeStatistics::new(values))
    }

    /// Copies one storage statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured storage-node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Python-owned storage statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, mutex, or panic failure.
    fn storage_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<StorageStatistics>> {
        let values = self.checked_view(
            py,
            "storage_statistics",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.storage_statistics_read(index),
        )?;
        Py::new(py, StorageStatistics::new(values))
    }

    /// Copies one outfall statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured outfall-node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Python-owned outfall statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, mutex, or panic failure.
    fn outfall_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<OutfallStatistics>> {
        let values = self.checked_view(
            py,
            "outfall_statistics",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.outfall_statistics_read(index),
        )?;
        Py::new(py, OutfallStatistics::new(values))
    }

    /// Copies one link statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    ///
    /// # Returns
    /// Python-owned statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn link_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<LinkStatistics>> {
        let values = self.checked_view(
            py,
            "link_statistics",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_statistics_read(index),
        )?;
        Py::new(py, LinkStatistics::new(values))
    }

    /// Copies one pump statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured pump-link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    ///
    /// # Returns
    /// Python-owned pump statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, mutex, or panic failure.
    fn pump_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<PumpStatistics>> {
        let values = self.checked_view(
            py,
            "pump_statistics",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.pump_statistics_read(index),
        )?;
        Py::new(py, PumpStatistics::new(values))
    }

    /// Copies one subcatchment statistics record after validating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary subtype label.
    ///
    /// # Returns
    /// Python-owned statistics fields.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn subcatchment_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Py<SubcatchmentStatistics>> {
        let values = self.checked_view(
            py,
            "subcatchment_statistics",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_statistics_read(index),
        )?;
        Py::new(py, SubcatchmentStatistics::new(values))
    }

    /// Copies ordered node statistics while detached from Python.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native aligned statistics record.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn node_statistics_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<NodeStatisticsSnapshot>> {
        let ids = snapshot_ids(ids, "node_statistics")?;
        let values =
            self.detached_checked_generation(py, "node_statistics", generation, move |handle| {
                handle
                    .simulation
                    .node_statistics_snapshot_read(ids.as_deref())
            })?;
        Py::new(py, NodeStatisticsSnapshot::new(values))
    }

    /// Copies ordered link statistics while detached from Python.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native aligned statistics record.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn link_statistics_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<LinkStatisticsSnapshot>> {
        let ids = snapshot_ids(ids, "link_statistics")?;
        let values =
            self.detached_checked_generation(py, "link_statistics", generation, move |handle| {
                handle
                    .simulation
                    .link_statistics_snapshot_read(ids.as_deref())
            })?;
        Py::new(py, LinkStatisticsSnapshot::new(values))
    }

    /// Copies ordered subcatchment statistics while detached from Python.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native aligned statistics record.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn subcatchment_statistics_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<SubcatchmentStatisticsSnapshot>> {
        let ids = snapshot_ids(ids, "subcatchment_statistics")?;
        let values = self.detached_checked_generation(
            py,
            "subcatchment_statistics",
            generation,
            move |handle| {
                handle
                    .simulation
                    .subcatchment_statistics_snapshot_read(ids.as_deref())
            },
        )?;
        Py::new(py, SubcatchmentStatisticsSnapshot::new(values))
    }

    /// Copies system continuity statistics while detached from Python.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    ///
    /// # Returns
    /// Immutable native coherent system statistics record.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, configured-storage, mutex, or panic failure.
    fn simulation_statistics(
        &self,
        py: Python<'_>,
        generation: u64,
    ) -> PyResult<Py<SimulationStatistics>> {
        let values =
            self.detached_checked_generation(py, "simulation_statistics", generation, |handle| {
                handle.simulation.simulation_statistics_read()
            })?;
        Py::new(py, SimulationStatistics::new(values))
    }
}
