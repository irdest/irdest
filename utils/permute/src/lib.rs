// SPDX-FileCopyrightText: 2020 Leonora Tindall <nora@nora.codes>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! # permute
//! Generate permutations of a slice in a memory-efficient and deterministic manner, using
//! [Heap's algorithm](https://en.wikipedia.org/wiki/Heap%27s_algorithm).
pub mod arbitrary_tandem_control_iter;
mod permutations;
mod permute_iter;

pub use permutations::permute;
pub use permute_iter::permutations_of;
