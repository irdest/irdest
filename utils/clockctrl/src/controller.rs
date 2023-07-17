// SPDX-FileCopyrightText: 2020,2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{ClockPoint, Error, PolicyMap};
use async_std::sync::{Arc, Barrier};
use std::{collections::BTreeMap, hash::Hash};

/// A clock identifier type
///
/// This type identifies a clockable point in a runtime.  A clockable
/// point is any point in the code that is frequently returned to by a
/// loop, or scheduling mechanism and that should be controlled either
/// by time, or external signals.
///
/// A clock point can either be: a simple name (`"foo"`), a namespaced
/// name (`"test" "foo"` stored as two strings), or a concrete type
/// defined by the user (either an enum or struct, as long as it is
/// `Hash` and `Ord`).
///
/// Each clockable point is then assigned a `Behavior`, which determines
/// the behaviour of the clock point.
#[derive(Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum ClockType<K>
where
    K: Hash + Ord,
{
    /// A simple named clock activity
    Named(String),
    /// A named clock activity with a namespace
    Namespaced(String, String),
    /// A fully typed clock activity
    User(K),
}

/// A control object for different clocking scopes
///
/// Each clock target can be configured individually via the [`Behavior`]
/// type, returned by [`setup()`].  Additionally you need to provide
/// some type that implements `Hash`.  It's recomended to just use an
/// enum that can be mapped onto each of your reactors internal tasks.
///
/// [`Behavior`]: struct.Behavior.html
/// [`setup()`]: struct.ClockCtrl.html#method.setup
pub struct ClockCtrl<K>
where
    K: Hash + Ord,
{
    clocks: BTreeMap<ClockType<K>, PolicyMap>,
}

/// A wrapper type around different clocking strategies
///
/// This type is returned by the `ClockCtl::start` function, to
/// provide an easy hook for any consumer of this API to regulate
/// their internal scheduling.  For details on what the two run modes
/// are, consult the variant docs.
pub enum Scheduler {
    /// The clocking schedule is constrained internally
    ///
    /// This corresponds to a clock type that was configured via the
    /// builder API, and can internally to the `ClockCtl` controller
    /// regulate the schedule of the selected task.  The only thing
    /// for you to do is poll the provided Barrier.
    Internal(Arc<Barrier>),
    /// The clock needs to be controlled externally
    ///
    /// This corresponds to not setting any additional constraints on
    /// the `Clock` builder, and instead letting the consumer of this
    /// API regulate itself: the clock control is only used as a
    /// toggle to determine it's runtime behaviour.
    External {
        /// A delay factor that can be added to any low-power timing
        /// inside the reactor
        delay: f32,
        /// One half of the barrier (give to task)
        a: Arc<Barrier>,
        /// Other half of the barrier (give to scheduler)
        b: Arc<Barrier>,
    },
}

impl<K> ClockCtrl<K>
where
    K: Hash + Ord,
{
    /// Create a new, empty clock controller
    pub fn new() -> Self {
        Self {
            clocks: Default::default(),
        }
    }

    /// Override the default clocking scheme for a particular target
    ///
    /// It's already possible to constrain clock settings witout
    /// setting custom bounds, just because the consumer of the
    /// `ClockCtl` type can fall back to some defaults when this
    /// builder returns an object filled with `None`.
    ///
    /// Canonically, the default constraints could be used to enable a
    /// low battery mode, whereas more low power embedded platforms
    /// can be further optimised.
    pub fn setup(&mut self, target: K) -> &mut PolicyMap {
        self.clocks
            .entry(ClockType::User(target))
            .or_insert(PolicyMap::default())
    }

    /// Start clock scheduler for a given task
    ///
    /// This function returns a Barrier which can be used in the
    /// corresponding task.
    pub fn start(&mut self, target: &ClockType<K>) -> Result<ClockPoint, Error> {
        let _b = Arc::new(Barrier::new(2));
        match self.clocks.remove(target) {
            Some(behavior_map) => Ok(ClockPoint::new(behavior_map)),
            None => Err(Error::NoBehavior),
        }

        // FIXME: ???????????????????????

        //     Some(Behavior { interval, fence }) => match (interval, fence) {
        //         // A raw external scheduler
        //         (None, None) => Ok(Scheduler::External {
        //             delay: 1.0,
        //             a: Arc::clone(&b),
        //             b,
        //         }),

        //         // An external scheduler, with a delay modifier
        //         (Some(Interval::Delay(d)), None) => Ok(Scheduler::External {
        //             delay: d,
        //             a: Arc::clone(&b),
        //             b,
        //         }),

        //         // A linearly timed internal scheduler
        //         (Some(Interval::Timed(dur)), None) => {
        //             let a = Arc::clone(&b);
        //             task::spawn(async move {
        //                 loop {
        //                     task::sleep(dur).await;
        //                     a.wait().await;
        //                 }
        //             });

        //             Ok(Scheduler::Internal(b))
        //         }

        //         // A manually clock stepped fence scheduler
        //         (Some(Interval::Stepped), Some(fence)) => {
        //             let a = Arc::clone(&b);
        //             task::spawn(async move {
        //                 fence(a);
        //             });

        //             Ok(Scheduler::Internal(b))
        //         }

        //         (_, _) => panic!("Invalid scheduler setup"),
        //     },
        //     None => Err(Error::NoBehavior),
        // }
    }
}
