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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks_instance_pallet, whitelisted_caller};
use frame_system::RawOrigin as SystemOrigin;
use sp_std::prelude::*;

use crate::Pallet as OpenRollup;

const SEED: u32 = 0;
const PROGRAM_HASH: [u8; 32] = *b"0000000000000001ad428e4906aE43D8";
const STATE_ROOT_1: [u8; 32] = *b"00000000000000000000000000000000";
const STATE_ROOT_2: [u8; 32] = *b"0059b62bc53ad4150a3e712d6273956f";

/// Check last event.
fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

/// Register one zkapp use default program hash.
fn register_default_zkapp<T: Config<I>, I: 'static>() -> (T::AccountId, AccountIdLookupOf<T>)
where
	CurrencyBalanceOf<T, I>: From<u64>,
{
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());

	T::Currency::make_free_balance_be(&caller, 8888888888u64.into());

	<<T as Config<I>>::Fungibles>::create(T::Helper::asset(11), caller.clone(), true, 1u32.into())
		.unwrap();
	<<T as Config<I>>::Nonfungibles>::force_create(
		&T::Helper::collection(11).into(),
		&caller,
		&caller,
	)
	.unwrap();

	assert!(OpenRollup::<T, I>::zkapp_register(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		ZkvmType::Fake,
		caller_lookup.clone(),
		T::Helper::state_root(STATE_ROOT_1),
	)
	.is_ok());

	(caller, caller_lookup)
}

/// Register one zkapp use another program hash.
fn register_other_zkapp<T: Config<I>, I: 'static>() -> (T::AccountId, AccountIdLookupOf<T>) {
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());

	assert!(OpenRollup::<T, I>::zkapp_register(
		SystemOrigin::Signed(caller.clone()).into(),
		T::Helper::program_hash(PROGRAM_HASH),
		ZkvmType::Fake,
		caller_lookup.clone(),
		T::Helper::state_root(STATE_ROOT_1),
	)
	.is_ok());

	(caller, caller_lookup)
}

/// Add assets support for zkapp of `PROGRAM_HASH` program hash
fn add_other_assets_support<T: Config<I>, I: 'static>() -> (T::AccountId, AccountIdLookupOf<T>) {
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());

	let asset_id = T::Helper::asset(11);
	let collection_id = T::Helper::collection(11);

	assert!(OpenRollup::<T, I>::add_asset_support(
		SystemOrigin::Signed(caller.clone()).into(),
		T::Helper::program_hash(PROGRAM_HASH),
		Asset::Fungible(asset_id),
	)
	.is_ok());

	assert!(OpenRollup::<T, I>::add_asset_support(
		SystemOrigin::Signed(caller.clone()).into(),
		T::Helper::program_hash(PROGRAM_HASH),
		Asset::Nonfungible(collection_id),
	)
	.is_ok());

	(caller, caller_lookup)
}

/// Add assets support for zkapp of default program hash
fn add_default_assets_support<T: Config<I>, I: 'static>() -> (T::AccountId, AccountIdLookupOf<T>) {
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());

	let asset_id = T::Helper::asset(11);
	let collection_id = T::Helper::collection(11);

	assert!(OpenRollup::<T, I>::add_asset_support(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		Asset::Fungible(asset_id),
	)
	.is_ok());

	assert!(OpenRollup::<T, I>::add_asset_support(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		Asset::Nonfungible(collection_id),
	)
	.is_ok());

	(caller, caller_lookup)
}

benchmarks_instance_pallet! {
	where_clause{
		where
			CurrencyBalanceOf<T, I>: From<u64>,
	}

	zkapp_register {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), ZkvmType::Fake, caller_lookup, Default::default())
	verify {
		assert_last_event::<T, I>(Event::ZkappRegister(ZkvmType::Fake, Default::default()).into());
	}

	add_asset_support {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		let asset = Asset::Fungible(T::Helper::asset(11));
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), asset.clone())
	verify {
		assert_last_event::<T, I>(Event::AddAssetSupport(Default::default(), asset).into());
	}

	change_submitter {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), caller_lookup)
	verify {
		assert_last_event::<T, I>(Event::ChangeSubmitter(Default::default(), caller).into());
	}

	set_inactive {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
	}: _(SystemOrigin::Signed(caller.clone()), Default::default())
	verify {
		assert_last_event::<T, I>(Event::SetInactive(Default::default()).into());
	}

	deposit {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		add_default_assets_support::<T, I>();
		<<T as Config<I>>::Fungibles as fungibles::Mutate<T::AccountId>>::mint_into(T::Helper::asset(11), &caller, 10u32.into()).unwrap();
		let asset_value = AssetValueOf::<T, I>::Fungible(T::Helper::asset(11), 10u32.into());
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), asset_value.clone())
	verify {
		assert_last_event::<T, I>(Event::Deposited(Default::default(), caller, asset_value).into());
	}

	withdraw {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		add_default_assets_support::<T, I>();
		let asset_value = AssetValueOf::<T, I>::Fungible(T::Helper::asset(11), 10u32.into());
		OpenRollup::<T, I>::add_zkapp_user_asset(Default::default(), caller.clone(), &asset_value).unwrap();
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), asset_value.clone())
	verify {
		assert_last_event::<T, I>(Event::Withdrawed(Default::default(), caller, asset_value).into());
	}

	move_asset {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		add_default_assets_support::<T, I>();
		register_other_zkapp::<T, I>();
		add_other_assets_support::<T, I>();
		let asset_value = AssetValueOf::<T, I>::Fungible(T::Helper::asset(11), 10u32.into());
		OpenRollup::<T, I>::add_zkapp_user_asset(Default::default(), caller.clone(), &asset_value).unwrap();
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), T::Helper::program_hash(PROGRAM_HASH), asset_value.clone())
	verify {
		assert_last_event::<T, I>(Event::MoveAsset(Default::default(), T::Helper::program_hash(PROGRAM_HASH), caller, asset_value).into());
	}

	exit {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		add_default_assets_support::<T, I>();

		<<T as Config<I>>::Fungibles as fungibles::Mutate<T::AccountId>>::mint_into(T::Helper::asset(11), &caller, 10000u32.into()).unwrap();

		let origin = SystemOrigin::Signed(caller.clone());
		let asset_value = AssetValueOf::<T, I>::Fungible(T::Helper::asset(11), 10u32.into());

		OpenRollup::<T, I>::deposit(origin.clone().into(), Default::default(), asset_value.clone()).unwrap();
		OpenRollup::<T, I>::set_inactive(origin.clone().into(), Default::default()).unwrap();

	}: _(SystemOrigin::Signed(caller.clone()), Default::default())
	verify {
		assert_last_event::<T, I>(Event::Exit(Default::default(), caller).into());
	}

	submit_batch {
		let (caller, caller_lookup) = register_default_zkapp::<T, I>();
		add_default_assets_support::<T, I>();

		// Generate state roots and users.
		let old_state_root = T::Helper::state_root(STATE_ROOT_1);
		let new_state_root = T::Helper::state_root(STATE_ROOT_2);
		let zk_proof = vec![1, 2, 3];
		let user_1: T::AccountId = account("user_1", 0, SEED);
		let user_2: T::AccountId = account("user_2", 0, SEED);
		let user_1_signed = SystemOrigin::Signed(user_1.clone());
		let user_2_signed = SystemOrigin::Signed(user_2.clone());

		// Add currency for users.
		T::Currency::make_free_balance_be(&user_1, 8888888888u64.into());
		T::Currency::make_free_balance_be(&user_2, 8888888888u64.into());

		// Add fungible asset for users.
		<<T as Config<I>>::Fungibles as fungibles::Mutate<T::AccountId>>::mint_into(T::Helper::asset(11), &user_1, 10000u32.into()).unwrap();

		// Add nonfungible asset for users.
		<<T as Config<I>>::Nonfungibles>::force_mint(&T::Helper::collection(11), &T::Helper::item(11), &user_2).unwrap();

		// Currency `AssetValue`.
		let asset_value_1 = AssetValueOf::<T, I>::Currency(1000u64.into());
		OpenRollup::<T, I>::deposit(user_1_signed.clone().into(), Default::default(), asset_value_1.clone()).unwrap();

		// Fungible `AssetValue`.
		let asset_value_2 = AssetValueOf::<T, I>::Fungible(T::Helper::asset(11), 10u32.into());
		OpenRollup::<T, I>::deposit(user_1_signed.clone().into(), Default::default(), asset_value_2.clone()).unwrap();

		// NonFungible `AssetValue`.
		let mut items = BoundedVec::default();
		items.try_push(T::Helper::item(11)).unwrap();
		let asset_value_3 = AssetValueOf::<T, I>::Nonfungible(T::Helper::collection(11), items);
		OpenRollup::<T, I>::deposit(user_2_signed.clone().into(), Default::default(), asset_value_3.clone()).unwrap();

		// Operations included in the batch submission.
		let operations = vec![
			Operation::Deposit(user_1.clone(), asset_value_1.clone()),
			Operation::Deposit(user_1.clone(), asset_value_2.clone()),
			Operation::Deposit(user_2.clone(), asset_value_3.clone()),

			Operation::Transfer(user_1.clone(), user_2.clone(), asset_value_1.clone()),
			Operation::Withdraw(user_2.clone(), asset_value_3.clone()),
		];

	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), old_state_root, new_state_root, 3, operations.clone(), zk_proof, None)
	verify {
		// Check SubmitBatch event.
		assert_last_event::<T, I>(Event::SubmitBatch(Default::default(), old_state_root, new_state_root, operations).into());

		let zkapp = Zkapps::<T, I>::try_get::<T::ProgramHash>(Default::default()).unwrap();
		// Check `l1_operations`data.
		assert_eq!(zkapp.l1_operations.len(), 0);
		// Check `new_state_root` has saved.
		assert_eq!(zkapp.state_root, new_state_root);

		// Check users's assets is correct.
		let account_1 = ZkappsAccounts::<T, I>::try_get::<T::ProgramHash, T::AccountId>(Default::default(), user_1.clone()).unwrap();
		assert_eq!(account_1.assets.last().unwrap(), &asset_value_2);
		let account_2 = ZkappsAccounts::<T, I>::try_get::<T::ProgramHash, T::AccountId>(Default::default(), user_2.clone()).unwrap();
		assert_eq!(account_2.assets.last().unwrap(), &asset_value_1);
	}


	impl_benchmark_test_suite!(OpenRollup, crate::mock::new_test_ext(), crate::mock::Test)
}
