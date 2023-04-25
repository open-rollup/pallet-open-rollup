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

//! Tests for Open Rollup pallet.

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, bounded_vec};
use sp_runtime::testing::H256;

const OWNER_ID: u64 = 1;
const SUBMITTER_ID: u64 = 2;
const USER_ID: u64 = 3;
const ASSET_ID: u32 = 1;
const COLLECTION_ID: u32 = 1;
const PROGRAM_HASH_64: u64 = 123;

type RuntimeEvent = <Test as Config>::RuntimeEvent;

/// Check last event.
fn assert_last_event(event: RuntimeEvent) {
	frame_system::Pallet::<Test>::assert_last_event(event);
}

/// Setup a zkapp, add fungible and nonfungible assets supports.
fn setup_app(program_hash_u64: u64) -> (H256, RuntimeOrigin, RuntimeOrigin) {
	let program_hash = H256::from_low_u64_be(program_hash_u64);
	let owner = RuntimeOrigin::signed(OWNER_ID);
	let empty_state_root = H256::from_low_u64_be(0);

	let user = RuntimeOrigin::signed(USER_ID);

	OpenRollup::zkapp_register(
		owner.clone(),
		program_hash,
		ZkvmType::Fake,
		SUBMITTER_ID,
		empty_state_root,
	)
	.unwrap();
	// check event
	assert_last_event(Event::ZkappRegister(ZkvmType::Fake, program_hash).into());

	OpenRollup::add_asset_support(owner.clone(), program_hash, Asset::Fungible(ASSET_ID)).unwrap();
	// check event
	assert_last_event(Event::AddAssetSupport(program_hash, Asset::Fungible(ASSET_ID)).into());

	OpenRollup::add_asset_support(owner.clone(), program_hash, Asset::Nonfungible(COLLECTION_ID))
		.unwrap();
	// check event
	assert_last_event(
		Event::AddAssetSupport(program_hash, Asset::Nonfungible(COLLECTION_ID)).into(),
	);

	// check zkapp data
	let zkapp = Zkapps::<Test>::try_get(program_hash).unwrap();
	assert_eq!(zkapp.owner, OWNER_ID);
	assert_eq!(zkapp.state_root, empty_state_root);
	assert_eq!(zkapp.supported_assets.first().unwrap(), &Asset::Currency);

	(program_hash, owner, user)
}

/// Register the same app duplicately.
#[test]
fn duplicate_register_zkapp() {
	new_test_ext().execute_with(|| {
		let (program_hash, owner, _user) = setup_app(PROGRAM_HASH_64);
		assert_noop!(
			OpenRollup::zkapp_register(
				owner,
				program_hash,
				ZkvmType::Miden,
				2,
				H256::from_low_u64_be(0),
			),
			Error::<Test>::DuplicateApp
		);
	});
}

/// Add support asset duplicately.
#[test]
fn duplicate_add_asset_support() {
	new_test_ext().execute_with(|| {
		let (program_hash, owner, _) = setup_app(PROGRAM_HASH_64);
		// Currency
		assert_noop!(
			OpenRollup::add_asset_support(owner.clone(), program_hash, Asset::Currency),
			Error::<Test>::DuplicateSupportAsset
		);
		// Fungible
		assert_noop!(
			OpenRollup::add_asset_support(owner.clone(), program_hash, Asset::Fungible(ASSET_ID)),
			Error::<Test>::DuplicateSupportAsset
		);
		// Nonfungible
		assert_noop!(
			OpenRollup::add_asset_support(
				owner.clone(),
				program_hash,
				Asset::Nonfungible(COLLECTION_ID)
			),
			Error::<Test>::DuplicateSupportAsset
		);
	});
}

/// Called by not-owner
#[test]
fn called_by_not_owner() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);
		let new_submitter = 5;

		assert_noop!(
			OpenRollup::add_asset_support(user.clone(), program_hash, Asset::Fungible(ASSET_ID)),
			Error::<Test>::NotOwner
		);
		assert_noop!(
			OpenRollup::change_submitter(user.clone(), program_hash, new_submitter),
			Error::<Test>::NotOwner
		);
		assert_noop!(OpenRollup::set_inactive(user.clone(), program_hash), Error::<Test>::NotOwner);
	});
}

/// Called by not-submitter
#[test]
fn called_by_not_submitter() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);

		assert_noop!(
			OpenRollup::submit_batch(
				user.clone(),
				program_hash,
				H256::from_low_u64_be(0),
				H256::from_low_u64_be(1),
				0,
				vec![],
				vec![1, 2, 3]
			),
			Error::<Test>::NotSubmitter
		);
	});
}

/// Unkown Zkapp
#[test]
fn called_on_unkowned_zkapp() {
	new_test_ext().execute_with(|| {
		let (_program_hash, owner, user) = setup_app(PROGRAM_HASH_64);
		let unknow_program_hash = H256::from_low_u64_be(999);
		let new_submitter = 5;

		assert_noop!(
			OpenRollup::add_asset_support(
				owner.clone(),
				unknow_program_hash,
				Asset::Fungible(ASSET_ID)
			),
			Error::<Test>::NoProgram
		);
		assert_noop!(
			OpenRollup::change_submitter(owner.clone(), unknow_program_hash, new_submitter),
			Error::<Test>::NoProgram
		);
		assert_noop!(
			OpenRollup::set_inactive(owner.clone(), unknow_program_hash),
			Error::<Test>::NoProgram
		);

		let asset_fungible = AssetValue::Fungible(ASSET_ID, 10);
		assert_noop!(
			OpenRollup::deposit(user.clone(), unknow_program_hash, asset_fungible.clone()),
			Error::<Test>::NoProgram
		);

		assert_noop!(
			OpenRollup::withdraw(user.clone(), unknow_program_hash, AssetValue::Currency(10)),
			Error::<Test>::NoProgram
		);
	});
}

/// Not supported assets
#[test]
fn called_on_not_supported_assets() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);
		let asset_fungible = AssetValue::Fungible(ASSET_ID + 10, 10);
		assert_noop!(
			OpenRollup::deposit(user.clone(), program_hash, asset_fungible.clone()),
			Error::<Test>::NotSupportAsset
		);

		assert_noop!(
			OpenRollup::withdraw(user.clone(), program_hash, asset_fungible.clone()),
			Error::<Test>::NotSupportAsset
		);
	});
}

/// Change a zkapp's submitter.
#[test]
fn change_submitter_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, owner, _user) = setup_app(PROGRAM_HASH_64);
		let new_submitter = 5;
		assert_ok!(OpenRollup::change_submitter(owner, program_hash, new_submitter));
		// check event
		assert_last_event(Event::ChangeSubmitter(program_hash, new_submitter).into());
		// check data
		let zkapp = Zkapps::<Test>::try_get(program_hash).unwrap();
		assert_eq!(zkapp.submitter, new_submitter);
	});
}

/// Set a zkapp to inactive
#[test]
fn set_inactive_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, owner, _user) = setup_app(PROGRAM_HASH_64);
		assert_ok!(OpenRollup::set_inactive(owner.clone(), program_hash));
		// check event
		assert_last_event(Event::SetInactive(program_hash).into());

		// if set inactive, only `exit` can be called
		assert_noop!(
			OpenRollup::add_asset_support(owner.clone(), program_hash, Asset::Fungible(ASSET_ID)),
			Error::<Test>::Inactive
		);
		assert_noop!(
			OpenRollup::change_submitter(owner.clone(), program_hash, 3),
			Error::<Test>::Inactive
		);
		// check data
		let zkapp = Zkapps::<Test>::try_get(program_hash).unwrap();
		assert_eq!(zkapp.is_inactive, true);
	});
}

/// User L1 deposit Tx into a zkapp
#[test]
fn zkapp_deposit_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);

		let asset_fungible = AssetValue::Fungible(ASSET_ID, 10);
		assert_ok!(OpenRollup::deposit(user.clone(), program_hash, asset_fungible.clone()));
		// check event
		assert_last_event(Event::Deposited(program_hash, USER_ID, asset_fungible.clone()).into());

		let asset_nonfungible = AssetValue::Nonfungible(COLLECTION_ID, bounded_vec![3]);
		assert_ok!(OpenRollup::deposit(user.clone(), program_hash, asset_nonfungible.clone()));
		// check event
		assert_last_event(
			Event::Deposited(program_hash, USER_ID, asset_nonfungible.clone()).into(),
		);
		// check data
		let zkapp = Zkapps::<Test>::try_get(program_hash).unwrap();
		assert_eq!(
			zkapp.l1_operations.first().unwrap(),
			&Operation::Deposit(USER_ID, asset_fungible.clone())
		);
		assert_eq!(
			zkapp.l1_operations.last().unwrap(),
			&Operation::Deposit(USER_ID, asset_nonfungible.clone())
		);
	});
}

/// User L1 withdraw Tx from a zkapp
#[test]
fn zkapp_withdraw_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);
		let asset_value = AssetValue::Fungible(ASSET_ID, 10);

		// need add asset for user befor withdraw
		OpenRollup::add_zkapp_user_asset(program_hash, USER_ID, &asset_value).unwrap();

		assert_ok!(OpenRollup::withdraw(user, program_hash, asset_value.clone()));
		// check event
		assert_last_event(Event::Withdrawed(program_hash, USER_ID, asset_value.clone()).into());
		// check data
		let zkapp = Zkapps::<Test>::try_get(program_hash).unwrap();
		assert_eq!(
			zkapp.l1_operations.first().unwrap(),
			&Operation::Withdraw(USER_ID, asset_value.clone())
		);
	});
}

/// move asset from a zkapp to another zkapp
#[test]
fn zkapp_move_asset_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash_1, _owner_2, _) = setup_app(PROGRAM_HASH_64);
		let (program_hash_2, _owner_2, user) = setup_app(456);
		let asset_value = AssetValue::Fungible(ASSET_ID, 10);

		// Add asset for user befor move
		OpenRollup::add_zkapp_user_asset(program_hash_1, USER_ID, &asset_value).unwrap();

		assert_ok!(OpenRollup::move_asset(
			user.clone(),
			program_hash_1,
			program_hash_2,
			asset_value.clone(),
		));
		// check event
		assert_last_event(
			Event::MoveAsset(program_hash_1, program_hash_2, USER_ID, asset_value.clone()).into(),
		);
		// check data
		let zkapp = Zkapps::<Test>::try_get(program_hash_1).unwrap();
		assert_eq!(
			zkapp.l1_operations.first().unwrap(),
			&Operation::Move(USER_ID, program_hash_2, asset_value.clone())
		);
	});
}

// Exit from zkapp
#[test]
fn zkapp_exit_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, owner, user) = setup_app(PROGRAM_HASH_64);

		// need add asset for user befor withdraw
		let asset_value = AssetValue::Currency(10);
		OpenRollup::deposit(user.clone(), program_hash, asset_value.clone()).unwrap();

		// check currency balance
		assert_eq!(<Test as Config>::Currency::free_balance(&OpenRollup::account_id()), 10);

		// set zkapp to inactive
		OpenRollup::set_inactive(owner, program_hash).unwrap();

		assert_ok!(OpenRollup::exit(user.clone(), program_hash));
		// check event
		assert_last_event(Event::Exit(program_hash, USER_ID).into());

		// check currency balance
		assert_eq!(<Test as Config>::Currency::free_balance(&OpenRollup::account_id()), 0);
	});
}

// submit a batch
#[test]
fn zkapp_submit_batch_should_work() {
	new_test_ext().execute_with(|| {
		let (program_hash, _owner, user) = setup_app(PROGRAM_HASH_64);

		// state_root
		let state_root_1 = H256::from_low_u64_be(0);
		let state_root_2 = H256::from_low_u64_be(1);

		// zk proof
		let zk_proof = vec![1, 2, 3];

		// assets
		let asset_value_1 = AssetValue::Currency(10);
		let asset_value_2 = AssetValue::Fungible(ASSET_ID, 10);
		let asset_value_3 = AssetValue::Nonfungible(COLLECTION_ID, bounded_vec![3, 4]);

		let l1_operations_pos = 3;

		let user_id_2 = 6;

		// deposit
		OpenRollup::deposit(user.clone(), program_hash, asset_value_1.clone()).unwrap();
		OpenRollup::deposit(user.clone(), program_hash, asset_value_2.clone()).unwrap();
		OpenRollup::deposit(user.clone(), program_hash, asset_value_3.clone()).unwrap();

		// check currency balance
		assert_eq!(<Test as Config>::Currency::free_balance(&OpenRollup::account_id()), 10);

		let operations = vec![
			Operation::Deposit(USER_ID, asset_value_1),
			Operation::Deposit(USER_ID, asset_value_2),
			Operation::Deposit(USER_ID, asset_value_3),
			Operation::Transfer(USER_ID, user_id_2, AssetValue::Fungible(ASSET_ID, 2)),
			Operation::Transfer(
				USER_ID,
				user_id_2,
				AssetValue::Nonfungible(COLLECTION_ID, bounded_vec![3]),
			),
		];

		// submit_batch
		assert_ok!(OpenRollup::submit_batch(
			RuntimeOrigin::signed(SUBMITTER_ID),
			program_hash,
			state_root_1,
			state_root_2,
			l1_operations_pos,
			operations.clone(),
			zk_proof,
		));

		// check event
		assert_last_event(
			Event::SubmitBatch(program_hash, state_root_1, state_root_2, operations).into(),
		);

		// check account data
		let account = ZkappsAccounts::<Test>::try_get(program_hash, USER_ID).unwrap();

		assert_eq!(account.assets.len(), 3);
		assert_eq!(account.assets.first().unwrap(), &AssetValue::Currency(10));
		assert_eq!(account.assets.get(1).unwrap(), &AssetValue::Fungible(ASSET_ID, 8));
		assert_eq!(
			account.assets.last().unwrap(),
			&AssetValue::Nonfungible(COLLECTION_ID, bounded_vec![4])
		);
	});
}
