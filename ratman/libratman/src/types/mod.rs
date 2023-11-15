// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! API encoding types for Ratman

mod api_util;
mod envelope;
pub mod error;
mod identifiers;
mod letterhead;
mod recipient;
mod sequence_id;
mod status;
mod timepair;

pub use api_util::*;
pub use envelope::InMemoryEnvelope;
pub use identifiers::{address::Address, id::Id, subnet::Subnet, target::Neighbour, ID_LEN};
pub use letterhead::Letterhead;
pub use recipient::Recipient;
pub use sequence_id::SequenceIdV1;
pub use status::CurrentStatus;
pub use timepair::TimePair;
