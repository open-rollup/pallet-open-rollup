//  Copyright 2022 Open Rollup Lab
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

//! Autogenerated weights for `pallet_open_rollup`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `debian`, CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/node-template
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_open_rollup
// --extrinsic=*
// --steps
// 50
// --repeat
// 20
// --output
// weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_assets.
pub trait WeightInfo {
    fn zkapp_register() -> Weight;
    fn add_asset_support() -> Weight;
    fn change_submitter() -> Weight;
    fn set_inactive() -> Weight;
    fn deposit() -> Weight;
    fn withdraw() -> Weight;
    fn move_asset() -> Weight;
    fn exit() -> Weight;
    fn submit_batch(ops_len: u32) -> Weight;
}

/// Weight functions for `pallet_open_rollup`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn zkapp_register() -> Weight {
		// Minimum execution time: 26_576 nanoseconds.
		Weight::from_ref_time(27_203_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn add_asset_support() -> Weight {
		// Minimum execution time: 39_073 nanoseconds.
		Weight::from_ref_time(59_438_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn change_submitter() -> Weight {
		// Minimum execution time: 23_980 nanoseconds.
		Weight::from_ref_time(24_816_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn set_inactive() -> Weight {
		// Minimum execution time: 24_124 nanoseconds.
		Weight::from_ref_time(27_062_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn deposit() -> Weight {
		// Minimum execution time: 67_083 nanoseconds.
		Weight::from_ref_time(68_999_000_u64)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	fn withdraw() -> Weight {
		// Minimum execution time: 29_619 nanoseconds.
		Weight::from_ref_time(30_187_000_u64)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:2 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	fn move_asset() -> Weight {
		// Minimum execution time: 33_833 nanoseconds.
		Weight::from_ref_time(37_397_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:0)
	// Storage: OpenRollup ZkappsExit (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn exit() -> Weight {
		// Minimum execution time: 69_844 nanoseconds.
		Weight::from_ref_time(77_761_000_u64)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:2 w:2)
	// Storage: Uniques Asset (r:1 w:1)
	// Storage: Uniques Class (r:1 w:0)
	// Storage: Uniques Account (r:0 w:2)
	// Storage: Uniques ItemPriceOf (r:0 w:1)
	fn submit_batch(ops_len: u32) -> Weight {
		// Minimum execution time: 85_676 nanoseconds.
		Weight::from_ref_time(87_229_000_u64)
            .saturating_add(Weight::from_ref_time(5_000_000_u64).saturating_mul(ops_len as u64))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn zkapp_register() -> Weight {
		// Minimum execution time: 26_576 nanoseconds.
		Weight::from_ref_time(27_203_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn add_asset_support() -> Weight {
		// Minimum execution time: 39_073 nanoseconds.
		Weight::from_ref_time(59_438_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn change_submitter() -> Weight {
		// Minimum execution time: 23_980 nanoseconds.
		Weight::from_ref_time(24_816_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	fn set_inactive() -> Weight {
		// Minimum execution time: 24_124 nanoseconds.
		Weight::from_ref_time(27_062_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn deposit() -> Weight {
		// Minimum execution time: 67_083 nanoseconds.
		Weight::from_ref_time(68_999_000_u64)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	fn withdraw() -> Weight {
		// Minimum execution time: 29_619 nanoseconds.
		Weight::from_ref_time(30_187_000_u64)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:2 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	fn move_asset() -> Weight {
		// Minimum execution time: 33_833 nanoseconds.
		Weight::from_ref_time(37_397_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:0)
	// Storage: OpenRollup ZkappsExit (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn exit() -> Weight {
		// Minimum execution time: 69_844 nanoseconds.
		Weight::from_ref_time(77_761_000_u64)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	// Storage: OpenRollup Zkapps (r:1 w:1)
	// Storage: OpenRollup ZkappsAccounts (r:2 w:2)
	// Storage: Uniques Asset (r:1 w:1)
	// Storage: Uniques Class (r:1 w:0)
	// Storage: Uniques Account (r:0 w:2)
	// Storage: Uniques ItemPriceOf (r:0 w:1)
	fn submit_batch(ops_len: u32) -> Weight {
		// Minimum execution time: 85_676 nanoseconds.
		Weight::from_ref_time(87_229_000_u64)
            .saturating_add(Weight::from_ref_time(5_000_000_u64).saturating_mul(ops_len as u64))
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
}
