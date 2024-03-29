use atomptr::AtomPtr;
use chrono::{DateTime, Utc};

/// Indicate different states the router context can exist in
///
/// Each type contains a UTC timestamp indicating the last state
/// transition.  This is included for log-tracing purposes.
///
/// Internally this type uses an atomic pointer to allow internal
/// changes without external mutability.
#[derive(Clone)]
pub struct RuntimeState(AtomPtr<RuntimeStateInner>);

// FIXME: WHY THE FUCK IS THIS NEEDED ??? AtomPtr LITERALLY IS AN ARC
// WITH EXTRA STEPS AAAAAAAAAAAAAAAAAAAAAAAAAAAH
#[derive(Clone)]
pub enum RuntimeStateInner {
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
    #[allow(unused)]
    Idle(DateTime<Utc>),
}

impl RuntimeState {
    /// Call this function when first creating a router (or
    /// re-creating it)
    pub fn start_initialising() -> Self {
        Self(AtomPtr::new(RuntimeStateInner::Startup(Utc::now())))
    }

    /// Call this function at the end of router initialisation
    pub fn finished_initialising(&self) -> bool {
        self.0
            .compare_exchange(self.0.get_ref(), RuntimeStateInner::Running(Utc::now()))
            .success()
    }

    /// Call this function when ordering the router to shut down
    pub fn terminate(&self) -> bool {
        self.0
            .compare_exchange(self.0.get_ref(), RuntimeStateInner::Terminating(Utc::now()))
            .success()
    }

    /// Call this function when forcibly killing the router
    pub fn kill(&self) -> bool {
        self.0
            .compare_exchange(self.0.get_ref(), RuntimeStateInner::Dying(Utc::now()))
            .success()
    }

    /// Call this function when the router reached the 'idle' state
    #[allow(unused)]
    pub fn set_idle(&self) -> bool {
        self.0
            .compare_exchange(self.0.get_ref(), RuntimeStateInner::Idle(Utc::now()))
            .success()
    }
}
