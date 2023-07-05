use chrono::{DateTime, Utc};

/// Indicate different states the router context can exist in
///
/// Each type contains a UTC timestamp indicating the last state
/// transition.  This is included for log-tracing purposes.
pub enum RuntimeState {
    /// The router exists, but hasn't started normal operation yet
    Startup(DateTime<Utc>),
    /// The router is running normally
    Running(DateTime<Utc>),
    /// The router is performing an ordered shutdown
    Terminating(DateTime<Utc>),
    /// The router is being killed
    Dying(DateTime<Utc>),
    /// The router is no longer running but hasn't been de-allocated yet.
    ///
    /// In this state it is actually possible to re-start the router.
    /// This may be done if a configuration or driver change is
    /// applied in-memory.  This allows routing and key tables to
    /// remain in cache.
    Idle(DateTime<Utc>),
}

impl RuntimeState {
    /// Call this function when first creating a router (or
    /// re-creating it)
    pub fn start_initialising() -> Self {
        Self::Startup(Utc::now())
    }

    /// Call this function at the end of router initialisation
    pub fn finished_initialising(&mut self) {
        *self = Self::Running(Utc::now())
    }

    /// Call this function when ordering the router to shut down
    pub fn terminate(&mut self) {
        *self = Self::Terminating(Utc::now())
    }

    /// Call this function when forcibly killing the router
    pub fn kill(&mut self) {
        *self = Self::Dying(Utc::now())
    }

    /// Call this function when the router reached the 'idle' state
    pub fn set_idle(&mut self) {
        *self = Self::Idle(Utc::now())
    }
}
