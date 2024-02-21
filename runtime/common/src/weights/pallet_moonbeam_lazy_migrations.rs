// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.
//! Autogenerated weights for `pallet_moonbeam_lazy_migrations`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-13, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `srv589657`, CPU: `AMD Ryzen 9 7950X 16-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("moonbase-dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/moonbeam
// benchmark
// pallet
// --chain=moonbase-dev
// --steps=50
// --repeat=20
// --pallet=pallet_moonbeam_lazy_migrations
// --extrinsic=*
// --wasm-execution=compiled
// --header=./file_header.txt
// --output=./runtime/common/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_moonbeam_lazy_migrations`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_moonbeam_lazy_migrations::WeightInfo for WeightInfo<T> {
	/// Storage: `EVM::AccountCodes` (r:1000 w:0)
	/// Proof: `EVM::AccountCodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EVM::AccountStorages` (r:33000 w:32000)
	/// Proof: `EVM::AccountStorages` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[0, 1000]`.
	/// The range of component `l` is `[0, 32500]`.
	fn clear_suicided_storage(a: u32, l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + a * (2 ±0) + l * (87 ±0)`
		//  Estimated: `7953 + a * (2352 ±35) + l * (2564 ±1)`
		// Minimum execution time: 1_770_000 picoseconds.
		Weight::from_parts(1_860_000, 0)
			.saturating_add(Weight::from_parts(0, 7953))
			// Standard Error: 1_291_244
			.saturating_add(Weight::from_parts(18_976_954, 0).saturating_mul(a.into()))
			// Standard Error: 39_723
			.saturating_add(Weight::from_parts(5_597_489, 0).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(l.into())))
			.saturating_add(Weight::from_parts(0, 2352).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 2564).saturating_mul(l.into()))
	}
}