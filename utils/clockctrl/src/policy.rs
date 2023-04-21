//! Define runtime and scheduling policies

use crate::point::Behavior;
use std::collections::HashMap;

/// Specify the runtime policy for a given behaviour
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Policy {
    /// The system is running as expected, at maximum capacity
    ///
    /// Any task should scale with the amount of work and not throttle
    /// itself in any way, unless deemed vital by another component
    /// (to avoid backpressure on a given resource).
    Nominal,
    /// The system is running in a low power state
    ///
    /// Tasks in this mode MUST throttle themselves to comply with a
    /// low-power policy, which will reduce throughput.  It can also
    /// be used to select different priority strategies (to boost
    /// productivity of slow components)
    LowPower,
    /// The system is not active, only performing periodic checks
    Hibernation,
}

/// Control behaviours for different policies individually
#[derive(Default)]
pub struct PolicyMap(HashMap<Policy, Behavior>);

impl PolicyMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn for_policy(mut self, p: Policy, b: Behavior) -> Self {
        self.0.insert(p, b);
        self
    }
}
