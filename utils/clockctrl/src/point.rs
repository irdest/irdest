// SPDX-FileCopyrightText: 2020,2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::sync::{Arc, Barrier};
use atomptr::AtomPtr;
use std::time::Duration;

use crate::{Policy, PolicyMap, Scheduler};

/// The type of clocking mechanism to use
#[derive(Clone, Debug)]
pub enum Interval {
    /// Indicates that the parameter should be manually stepped
    Stepped,
    /// Adds a relative delay to the default clock times
    Delay(f32),
    /// Schedules an event in a fixed interval
    Timed(Duration),
}

/// Represents a single clock behaviour
///
/// Don't construct this object manually, get a mutable builder
/// reference from [`Clockctrl::setup()`]!
///
/// [`Clockctrl::setup()`]: struct.ClockCtrl.html#method.setup
#[derive(Default)]
pub struct Behavior {
    /// Specify an interval which dictates scheduling of a task
    pub interval: Option<Interval>,
    /// If Interval::Stepped, provide a fence to step the clock with
    pub fence: Option<Box<dyn Fn(Arc<Barrier>) + Send + 'static>>,
}

impl Behavior {
    /// Set the interval at which this clock will be controlled
    pub fn set(&mut self, iv: Interval) -> &mut Self {
        self.interval = Some(iv);
        self
    }

    /// Setup a fence which will clock control the associated task
    ///
    /// The provided function is called once, on a detached task, and
    /// is expected to block the task with block_on, which can then do
    /// async operation.
    pub fn fence<F: 'static>(&mut self, f: F) -> &mut Self
    where
        F: Fn(Arc<Barrier>) + Send,
    {
        self.fence = Some(Box::new(f));
        self
    }
}

pub struct ClockPoint {
    current_policy: AtomPtr<Policy>,
    current_schedule: AtomPtr<Option<Scheduler>>,
    map: PolicyMap,
}

impl ClockPoint {
    /// Create a new clock point
    pub fn new(map: PolicyMap) -> Self {
        Self {
            current_policy: AtomPtr::new(Policy::Nominal),
            current_schedule: AtomPtr::new(None),
            map,
        }
    }

    /// Adjust the schedule for this clock point to a new policy
    ///
    /// This configures this clock point to be internally scheduled,
    /// meaning that internally tasks are spawned to control the
    /// schedule behaviour.
    ///
    /// If you want direct (or raw) control over a clock point, use
    /// `switch_policy_external` instead.
    pub fn switch_policy_internal(&self, new_policy: &Policy) {
        self.current_policy.swap(new_policy.clone());
    }

    /// Hang the current task until the next clock point is reached
    pub async fn wait_for_clock(&self) {}
}
