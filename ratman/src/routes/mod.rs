//! Provide utilities and types to keep track of routes in the network
//!
//! At its core a route consists of an `Address` and a link that has been known
//! to deliver Announcements from that address.  There are different states of
//! routes: `active`, `idle`, and `lost`.
//!
//! - `active`: The peer is online and we are receiving announcements via a
//! particular link.  Multiple links can be referenced in an active route at the
//! same time, since loops and cliques in the network can exist.
//!
//! - `idle`: The peer is offline, but the link that has been sending
//! announcements from this peer is still active.  When a peer goes offline it
//! needs to have been seen at least twice before transitioning to the `idle`
//! phase.
//!
//! - `lost`: The peer is offline and we don't know if it will be back.  When we
//! first meet a peer and it goes offline, it will transition into the `lost`
//! state.  When it comes online a second time via the same link, when it then
//! goes offline it will be marked as `idle`, since we can assume that it will
//! probably come back via the same link again.
//!
//! Route data is additive, meaning that old link associations with a peer will
//! be kept around for some time, even if the peer has re-introduced itself from
//! a new link.  Stale link associations will only be pruned after a timeout or
//! when a peer has more than 5 link associations, since that implies that it
//! keeps connecting from a new remote link every time.
//!
//! Note: a "link" in this lingo not only refers to the physical connection
//! channel used to transmit data but also the `Neighbour` discriminat.
//!
//! A route also contains metadata on the quality of the link, estimated delay
//! time, maximum transfer unit, a stream size hint, etc.
//!
//! This module consists of two main parts: the main routing table type
//! utilities, and route scoring, which is how Ratman decides on a route if
//! multiple active options exist.

mod scoring;
