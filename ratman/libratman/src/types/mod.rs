// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! API encoding types for Ratman

mod envelope;
pub mod error;
mod identifiers;
mod letterhead;
mod recipient;
mod sequence_id;
mod status;
mod target;
mod timepair;

pub use envelope::InMemoryEnvelope;
pub use identifiers::{address::Address, id::Id, ID_LEN};
pub use letterhead::Letterhead;
pub use recipient::Recipient;
pub use sequence_id::SequenceIdV1;
pub use status::CurrentStatus;
pub use target::EpTarget;
pub use timepair::TimePair;
