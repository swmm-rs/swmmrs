use super::*;

/// Lock-free iterator ownership and termination-request state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IteratorState {
    Idle,
    Iterating,
    Requested,
}

impl IteratorState {
    /// Converts the private atomic representation into a typed iterator state.
    ///
    /// # Arguments
    /// * `value` - Byte loaded from the owner-local atomic state.
    ///
    /// # Returns
    /// The recognized iterator state, or `None` for an invalid internal value.
    ///
    /// # Errors
    /// Returns `None` when `value` is not a recognized iterator state.
    pub(super) fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Iterating),
            2 => Some(Self::Requested),
            _ => None,
        }
    }

    /// Converts a typed iterator state to its private atomic representation.
    ///
    /// # Returns
    /// Byte stored only inside `AtomicIteratorState`.
    pub(super) fn byte(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Iterating => 1,
            Self::Requested => 2,
        }
    }
}

/// Atomic representation boundary that exposes only typed iterator states.
///
/// `std` has no atomic enum; the private byte never escapes this wrapper.
pub(super) struct AtomicIteratorState(AtomicU8);

impl AtomicIteratorState {
    /// Creates an idle owner-local iterator state.
    ///
    /// # Returns
    /// A new atomic iterator state initialized to `Idle`.
    pub(super) fn idle() -> Self {
        Self(AtomicU8::new(IteratorState::Idle.byte()))
    }

    /// Loads the current typed iterator state.
    ///
    /// # Returns
    /// The current state, or `None` if the internal representation is invalid.
    ///
    /// # Errors
    /// Returns `None` when the internal byte is not a recognized iterator state.
    pub(super) fn load(&self) -> Option<IteratorState> {
        IteratorState::from_byte(self.0.load(Ordering::SeqCst))
    }

    /// Publishes a typed iterator state.
    ///
    /// # Arguments
    /// * `state` - Iterator state to publish.
    pub(super) fn store(&self, state: IteratorState) {
        self.0.store(state.byte(), Ordering::SeqCst);
    }

    /// Atomically replaces one expected typed iterator state.
    ///
    /// # Arguments
    /// * `current` - State that must currently be published.
    /// * `new` - Replacement state to publish.
    ///
    /// # Returns
    /// `true` when the comparison succeeds.
    pub(super) fn replace(&self, current: IteratorState, new: IteratorState) -> bool {
        self.0
            .compare_exchange(
                current.byte(),
                new.byte(),
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
    }
}

/// Checked identity of one successfully opened project.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ProjectGeneration(pub(super) u64);

/// Mutable state serialized behind one package-private owner mutex.
pub(super) struct HandleState {
    pub(super) simulation: SwmmSimulation,
    pub(super) generation_counter: u64,
    pub(super) active_generation: Option<ProjectGeneration>,
    pub(super) failure_operation: Option<&'static str>,
    pub(super) paths: Option<OwnerPaths>,
    pub(super) advance_stride: Option<(i32, bool)>,
    pub(super) iterator_exhausted: bool,
}

impl Default for HandleState {
    /// Creates one closed native handle with no Project Generation.
    ///
    /// # Returns
    /// A handle ready to open its first project.
    fn default() -> Self {
        Self {
            simulation: SwmmSimulation::default(),
            generation_counter: 0,
            active_generation: None,
            failure_operation: None,
            paths: None,
            advance_stride: None,
            iterator_exhausted: false,
        }
    }
}

impl HandleState {
    /// Opens a project and installs its next checked generation identity.
    ///
    /// # Arguments
    /// * `paths` - Validated absolute owner paths attempted by the facade.
    ///
    /// # Returns
    /// `Ok(())` after the project opens and its generation is installed.
    ///
    /// # Errors
    /// Returns a solver open failure with attempted paths, or counter exhaustion.
    pub(super) fn open_project(&mut self, paths: OwnerPaths) -> Result<(), SwmmError> {
        let input = paths.input.as_str();
        let report = paths.report.as_str();
        let output = paths.output.as_deref().unwrap_or("");
        let generation = self.generation_counter.checked_add(1).ok_or_else(|| {
            SwmmError::with_detail(
                ErrorCode::System,
                format!(
                    "project generation exhausted; attempted input {input}, report {report}, output {}",
                    if output.is_empty() { "<scratch>" } else { output }
                ),
            )
        })?;
        if let Err(mut error) = self.simulation.open(input, report, output) {
            let attempted = format!(
                "attempted input {input}, report {report}, output {}",
                if output.is_empty() {
                    "<scratch>"
                } else {
                    output
                }
            );
            error.detail = Some(match error.detail.take() {
                Some(detail) => format!("{detail}; {attempted}"),
                None => attempted,
            });
            return Err(error);
        }
        self.generation_counter = generation;
        self.failure_operation = None;
        self.active_generation = Some(ProjectGeneration(generation));
        self.paths = Some(paths);
        self.advance_stride = None;
        self.iterator_exhausted = false;
        Ok(())
    }

    /// Closes the project and clears its active generation even when cleanup fails.
    ///
    /// # Returns
    /// `Ok(())` after the project is closed and its active generation is cleared.
    ///
    /// # Errors
    /// Returns a newly encountered end or project-close failure.
    pub(super) fn close_project(&mut self) -> Result<(), SwmmError> {
        let result = self.simulation.close();
        self.active_generation = None;
        self.failure_operation = None;
        self.iterator_exhausted = true;
        result
    }

    /// Executes one full run and clears the generation closed by execution.
    ///
    /// # Arguments
    /// * `save_results` - Whether to save report-period results and detailed reporting.
    ///
    /// # Returns
    /// `Ok(())` after the full lifecycle executes and closes successfully.
    ///
    /// # Errors
    /// Returns the primary execution failure with any cleanup detail retained.
    pub(super) fn execute(&mut self, save_results: bool) -> Result<(), SwmmError> {
        let result = self.simulation.execute(save_results);
        if self.simulation.lifecycle_read().state == SimulationLifecycle::Closed {
            self.active_generation = None;
            self.iterator_exhausted = true;
        }
        result
    }
}

/// Pure-Rust failures returned from work detached from Python.
pub(super) enum DetachedFailure {
    Solver(&'static str, SwmmError),
    Stale,
    Poisoned,
    Panicked,
}

/// Owner-local deterministic synchronization state for concurrency tests.
#[cfg(feature = "test-support")]
#[derive(Default)]
pub(super) struct TestConcurrencyState {
    inner: Mutex<TestConcurrencyInner>,
    changed: Condvar,
}

/// Flags protected by one test-only owner-local condition variable.
#[cfg(feature = "test-support")]
#[derive(Default)]
pub(super) struct TestConcurrencyInner {
    pause_armed: bool,
    paused: bool,
    release_pause: bool,
    probe_armed: bool,
    probe_waiting: bool,
    probe_acquired: bool,
}

#[cfg(feature = "test-support")]
impl TestConcurrencyState {
    /// Arms one pause after the next owner operation finishes but before unlock.
    pub(super) fn arm_pause(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.pause_armed = true;
        inner.paused = false;
        inner.release_pause = false;
    }

    /// Waits for an armed completed operation up to `timeout`.
    ///
    /// # Arguments
    /// * `timeout` - Deadlock-guard duration; synchronization state, not elapsed work, proves overlap.
    ///
    /// # Returns
    /// `true` when the operation finishes while retaining the owner mutex.
    pub(super) fn wait_paused(&self, timeout: Duration) -> bool {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (inner, _) = self
            .changed
            .wait_timeout_while(inner, timeout, |inner| !inner.paused)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.paused
    }

    /// Releases the completed operation to unlock its owner mutex.
    pub(super) fn release_pause(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.release_pause = true;
        self.changed.notify_all();
    }

    /// Arms observation of the next production owner acquisition.
    pub(super) fn arm_probe(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.probe_armed = true;
        inner.probe_waiting = false;
        inner.probe_acquired = false;
    }

    /// Records that an armed operation is about to acquire the owner mutex.
    pub(super) fn before_lock(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.probe_armed {
            inner.probe_waiting = true;
            self.changed.notify_all();
        }
    }

    /// Records that an observed operation acquired the owner mutex.
    pub(super) fn after_lock(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.probe_armed {
            inner.probe_armed = false;
            inner.probe_acquired = true;
            self.changed.notify_all();
        }
    }

    /// Applies an armed deterministic pause before releasing the owner mutex.
    pub(super) fn before_unlock(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.pause_armed {
            inner.pause_armed = false;
            inner.paused = true;
            self.changed.notify_all();
            while !inner.release_pause {
                inner = self
                    .changed
                    .wait(inner)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
            inner.paused = false;
            inner.release_pause = false;
        }
    }

    /// Waits for an operation to reach owner acquisition up to `timeout`.
    ///
    /// # Arguments
    /// * `timeout` - Deadlock-guard duration; synchronization state, not elapsed work, proves ordering.
    ///
    /// # Returns
    /// `true` when the operation reaches owner acquisition.
    pub(super) fn wait_probe_waiting(&self, timeout: Duration) -> bool {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (inner, _) = self
            .changed
            .wait_timeout_while(inner, timeout, |inner| !inner.probe_waiting)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.probe_waiting
    }

    /// Returns whether the armed operation acquired the owner mutex.
    ///
    /// # Returns
    /// `true` after the observed operation acquires the owner mutex.
    pub(super) fn probe_acquired(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .probe_acquired
    }
}

/// Package-private synchronized Simulation Owner.
#[pyclass(frozen)]
pub(crate) struct NativeSimulation {
    pub(super) handle: Mutex<HandleState>,
    pub(super) iterator_state: AtomicIteratorState,
    #[cfg(feature = "test-support")]
    pub(super) test_concurrency: TestConcurrencyState,
}

impl NativeSimulation {
    /// Wraps a reconstructed solver owner in a fresh synchronized Python owner.
    ///
    /// # Arguments
    /// * `simulation` - Reconstructed Simulation Owner.
    /// * `paths` - Validated path metadata owned by the reconstructed project.
    ///
    /// # Returns
    /// A fresh native owner with one active Project Generation.
    pub(super) fn from_simulation(simulation: SwmmSimulation, paths: OwnerPaths) -> Self {
        Self {
            handle: Mutex::new(HandleState {
                simulation,
                generation_counter: 1,
                active_generation: Some(ProjectGeneration(1)),
                failure_operation: None,
                paths: Some(paths),
                advance_stride: None,
                iterator_exhausted: false,
            }),
            iterator_state: AtomicIteratorState::idle(),
            #[cfg(feature = "test-support")]
            test_concurrency: TestConcurrencyState::default(),
        }
    }

    /// Runs one scalar owner read with panic containment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `operation` - Stable operation name for structured failures.
    /// * `action` - Rust-only owner read to execute while locked.
    ///
    /// # Returns
    /// Owned operation result after the owner mutex is released.
    ///
    /// # Errors
    /// Returns a structured native error for owner, mutex, or panic failure.
    pub(super) fn attached<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        action: F,
    ) -> PyResult<T>
    where
        F: FnOnce(&HandleState) -> Result<T, SwmmError>,
    {
        let result = match catch_unwind(AssertUnwindSafe(|| {
            #[cfg(feature = "test-support")]
            self.test_concurrency.before_lock();
            let handle = self
                .handle
                .lock_py_attached(py)
                .map_err(|_| DetachedFailure::Poisoned)?;
            #[cfg(feature = "test-support")]
            self.test_concurrency.after_lock();
            let result = action(&handle).map_err(|error| DetachedFailure::Solver(operation, error));
            drop(handle);
            result
        })) {
            Ok(result) => result,
            Err(_) => Err(DetachedFailure::Panicked),
        };
        result.map_err(|failure| detached_error(operation, failure))
    }

    /// Runs one generation-bound read while holding the owner lock.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `operation` - Stable operation name for structured failures.
    /// * `generation` - Captured Project Generation to revalidate.
    /// * `action` - Rust-only owner read to execute while locked.
    ///
    /// # Returns
    /// Owned operation result after the owner mutex is released.
    ///
    /// # Errors
    /// Returns a stale, poisoned-mutex, action, or panic failure.
    pub(super) fn checked_generation<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        generation: u64,
        action: F,
    ) -> PyResult<T>
    where
        F: FnOnce(&HandleState) -> Result<T, DetachedFailure>,
    {
        let result = match catch_unwind(AssertUnwindSafe(|| {
            #[cfg(feature = "test-support")]
            self.test_concurrency.before_lock();
            let handle = self
                .handle
                .lock_py_attached(py)
                .map_err(|_| DetachedFailure::Poisoned)?;
            #[cfg(feature = "test-support")]
            self.test_concurrency.after_lock();
            if handle.active_generation != Some(ProjectGeneration(generation)) {
                return Err(DetachedFailure::Stale);
            }
            let result = action(&handle);
            drop(handle);
            result
        })) {
            Ok(result) => result,
            Err(_) => Err(DetachedFailure::Panicked),
        };
        result.map_err(|failure| detached_error(operation, failure))
    }

    /// Runs one owned Live View read with generation and typed-identity validation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `operation` - Stable operation name used in native diagnostics.
    /// * `generation` - Captured Project Generation.
    /// * `object_type` - Expected configured object family.
    /// * `index` - Captured native object index.
    /// * `id` - Captured canonical object ID.
    /// * `subtype` - Captured private subtype label.
    /// * `action` - Rust-only owner read to run under the same lock.
    ///
    /// # Returns
    /// Owned read result after the owner lock is released.
    ///
    /// # Errors
    /// Returns a stale, poisoned-mutex, owner-read, or panic failure.
    pub(super) fn checked_view<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        generation: u64,
        object_type: ObjectType,
        index: usize,
        id: &str,
        subtype: &str,
        action: F,
    ) -> PyResult<T>
    where
        F: FnOnce(&SwmmSimulation) -> Result<T, SwmmError>,
    {
        let subtype = object_subtype(object_type, subtype)
            .ok_or_else(|| internal_error(operation, "invalid private subtype"))?;
        self.checked_generation(py, operation, generation, |handle| {
            if !handle
                .simulation
                .object_identity_matches(object_type, index, id, subtype)
                .map_err(|error| DetachedFailure::Solver(operation, error))?
            {
                return Err(DetachedFailure::Stale);
            }
            action(&handle.simulation).map_err(|error| DetachedFailure::Solver(operation, error))
        })
    }

    /// Runs one detached Live View mutation with generation and typed-identity validation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during the mutation.
    /// * `operation` - Stable operation name used in native diagnostics.
    /// * `generation` - Captured Project Generation.
    /// * `object_type` - Expected configured object family.
    /// * `index` - Captured native object index.
    /// * `id` - Captured canonical object ID.
    /// * `subtype` - Captured private subtype label.
    /// * `action` - Rust-only owner mutation to run under the same lock.
    ///
    /// # Returns
    /// Owned mutation result after Python is reattached.
    ///
    /// # Errors
    /// Returns a stale, poisoned-mutex, owner-mutation, or panic failure.
    pub(super) fn detached_checked_view<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        generation: u64,
        object_type: ObjectType,
        index: usize,
        id: String,
        subtype: String,
        action: F,
    ) -> PyResult<T>
    where
        T: Send,
        F: FnOnce(&mut SwmmSimulation) -> Result<T, SwmmError> + Send,
    {
        self.detached_checked_view_result(
            py,
            operation,
            generation,
            object_type,
            index,
            id,
            subtype,
            move |simulation| {
                action(simulation).map_err(|error| DetachedFailure::Solver(operation, error))
            },
        )
    }

    /// Runs one detached Live View operation that can report component staleness.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during the operation.
    /// * `operation` - Stable operation name used in native diagnostics.
    /// * `generation` - Captured Project Generation.
    /// * `object_type` - Expected configured object family.
    /// * `index` - Captured native object index.
    /// * `id` - Captured canonical object ID.
    /// * `subtype` - Captured private subtype label.
    /// * `action` - Rust-only owner operation that can return `DetachedFailure::Stale`.
    ///
    /// # Returns
    /// Owned operation result after Python is reattached.
    ///
    /// # Errors
    /// Returns a stale, poisoned-mutex, owner-operation, or panic failure.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn detached_checked_view_result<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        generation: u64,
        object_type: ObjectType,
        index: usize,
        id: String,
        subtype: String,
        action: F,
    ) -> PyResult<T>
    where
        T: Send,
        F: FnOnce(&mut SwmmSimulation) -> Result<T, DetachedFailure> + Send,
    {
        let subtype = object_subtype(object_type, &subtype)
            .ok_or_else(|| internal_error(operation, "invalid private subtype"))?;
        let result = py.detach(|| {
            match catch_unwind(AssertUnwindSafe(|| {
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_lock();
                let mut handle = self.handle.lock().map_err(|_| DetachedFailure::Poisoned)?;
                #[cfg(feature = "test-support")]
                self.test_concurrency.after_lock();
                if handle.active_generation != Some(ProjectGeneration(generation)) {
                    return Err(DetachedFailure::Stale);
                }
                if !handle
                    .simulation
                    .object_identity_matches(object_type, index, &id, subtype)
                    .map_err(|error| DetachedFailure::Solver(operation, error))?
                {
                    return Err(DetachedFailure::Stale);
                }
                let result = action(&mut handle.simulation);
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_unlock();
                drop(handle);
                result
            })) {
                Ok(result) => result,
                Err(_) => Err(DetachedFailure::Panicked),
            }
        });
        result.map_err(|failure| detached_error(operation, failure))
    }

    /// Runs one generation-bound mutation detached from Python.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during the mutation.
    /// * `operation` - Stable operation name used in native diagnostics.
    /// * `generation` - Captured Project Generation to revalidate while locked.
    /// * `action` - Rust-only owner mutation to run while locked.
    ///
    /// # Returns
    /// Mutation result after Python is reattached.
    ///
    /// # Errors
    /// Returns a native exception for stale, poisoned, panicked, or solver failure.
    pub(super) fn detached_checked_generation<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        generation: u64,
        action: F,
    ) -> PyResult<T>
    where
        T: Send,
        F: FnOnce(&mut HandleState) -> Result<T, SwmmError> + Send,
    {
        let result = py.detach(|| {
            match catch_unwind(AssertUnwindSafe(|| {
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_lock();
                let mut handle = self.handle.lock().map_err(|_| DetachedFailure::Poisoned)?;
                #[cfg(feature = "test-support")]
                self.test_concurrency.after_lock();
                if handle.active_generation != Some(ProjectGeneration(generation)) {
                    return Err(DetachedFailure::Stale);
                }
                let result =
                    action(&mut handle).map_err(|error| DetachedFailure::Solver(operation, error));
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_unlock();
                drop(handle);
                result
            })) {
                Ok(result) => result,
                Err(_) => Err(DetachedFailure::Panicked),
            }
        });
        result.map_err(|failure| detached_error(operation, failure))
    }

    /// Rejects operations that cannot overlap exclusive iterator advancement.
    ///
    /// # Arguments
    /// * `operation` - Operation rejected while iterator ownership is active.
    ///
    /// # Returns
    /// `Ok(())` when no iterator owns advancement.
    ///
    /// # Errors
    /// Returns a lifecycle error while iterator ownership or termination is active.
    pub(super) fn ensure_not_iterating(&self, operation: &'static str) -> PyResult<()> {
        if self.iterator_state.load() == Some(IteratorState::Idle) {
            Ok(())
        } else {
            Err(lifecycle_error(operation, "iterator owns advancement"))
        }
    }

    /// Runs one mutating owner operation detached from Python with panic containment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `operation` - Stable operation name for structured failures.
    /// * `action` - Rust-only owner operation to execute while locked.
    ///
    /// # Returns
    /// Owned operation result after the owner mutex is released.
    ///
    /// # Errors
    /// Returns a structured native error for solver failure, mutex poison, or panic.
    pub(super) fn detached<T, F>(
        &self,
        py: Python<'_>,
        operation: &'static str,
        action: F,
    ) -> PyResult<T>
    where
        T: Send,
        F: FnOnce(&mut HandleState) -> Result<T, SwmmError> + Send,
    {
        let result = py.detach(|| {
            match catch_unwind(AssertUnwindSafe(|| {
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_lock();
                let mut handle = self.handle.lock().map_err(|_| DetachedFailure::Poisoned)?;
                #[cfg(feature = "test-support")]
                self.test_concurrency.after_lock();
                let result = action(&mut handle).map_err(|error| {
                    let failure_operation = if handle.simulation.lifecycle_read().state
                        == SimulationLifecycle::Failed
                    {
                        *handle.failure_operation.get_or_insert(operation)
                    } else {
                        operation
                    };
                    DetachedFailure::Solver(failure_operation, error)
                });
                #[cfg(feature = "test-support")]
                self.test_concurrency.before_unlock();
                drop(handle);
                result
            })) {
                Ok(result) => result,
                Err(_) => Err(DetachedFailure::Panicked),
            }
        });
        result.map_err(|failure| detached_error(operation, failure))
    }
}

impl Drop for NativeSimulation {
    /// Performs panic-contained best-effort cleanup without surfacing destructor errors.
    fn drop(&mut self) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let handle = self
                .handle
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let _ = handle.close_project();
        }));
    }
}
