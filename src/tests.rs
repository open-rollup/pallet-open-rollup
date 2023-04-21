//! Tests for Open Rollup pallet.

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, bounded_vec};
use sp_runtime::testing::H256;

#[test]
fn zkapp_register_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		));
	});
}

#[test]
fn zkapp_add_asset_support_should_work() {
	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		assert_ok!(OpenRollup::add_asset_support(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			Asset::Fungible(1)
		));
		assert_ok!(OpenRollup::add_asset_support(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			Asset::Nonfungible(2)
		));
	});
}

#[test]
fn zkapp_change_submitter_should_work() {
	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		assert_ok!(OpenRollup::change_submitter(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			3
		));
	});
}

#[test]
fn zkapp_set_inactive_should_work() {
	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		assert_ok!(OpenRollup::set_inactive(RuntimeOrigin::signed(1), H256::from_low_u64_be(123)));
	});
}

#[test]
fn zkapp_deposit_should_work() {
	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		assert_ok!(OpenRollup::add_asset_support(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			Asset::Fungible(1)
		));
		assert_ok!(OpenRollup::add_asset_support(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			Asset::Nonfungible(1)
		));
		assert_ok!(OpenRollup::deposit(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			AssetValue::Nonfungible(1, bounded_vec![1])
		));
		assert_ok!(OpenRollup::deposit(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			AssetValue::Fungible(1, 10)
		));
	});
}

#[test]
fn zkapp_withdraw_should_work() {
	let program_hash = H256::from_low_u64_be(123);
	let account_id = 1;
	let signed_user = RuntimeOrigin::signed(account_id);

	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			signed_user.clone(),
			program_hash,
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		OpenRollup::add_asset_support(signed_user.clone(), program_hash, Asset::Fungible(1))
			.unwrap();

		OpenRollup::add_zkapp_user_asset(program_hash, account_id, &AssetValue::Fungible(1, 10))
			.unwrap();

		assert_ok!(OpenRollup::withdraw(signed_user, program_hash, AssetValue::Fungible(1, 10)));
	});
}

#[test]
fn zkapp_move_asset_should_work() {
	let program_hash = H256::from_low_u64_be(123);
	let account_id = 1;
	let signed_user = RuntimeOrigin::signed(account_id);
	let program_hash_2 = H256::from_low_u64_be(234);

	new_test_ext().execute_with(|| {
		// zkapps register
		OpenRollup::zkapp_register(
			signed_user.clone(),
			program_hash,
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		OpenRollup::zkapp_register(
			signed_user.clone(),
			program_hash_2,
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();

		// add asset support
		OpenRollup::add_asset_support(signed_user.clone(), program_hash, Asset::Fungible(1))
			.unwrap();
		OpenRollup::add_asset_support(signed_user.clone(), program_hash_2, Asset::Fungible(1))
			.unwrap();

		OpenRollup::add_zkapp_user_asset(program_hash, account_id, &AssetValue::Fungible(1, 10))
			.unwrap();

		assert_ok!(OpenRollup::move_asset(
			signed_user.clone(),
			program_hash,
			program_hash_2,
			AssetValue::Fungible(1, 10)
		));
	});
}

#[test]
fn zkapp_exit_should_work() {
	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			ZkvmType::Fake,
			2,
			H256::from_low_u64_be(0),
		)
		.unwrap();
		OpenRollup::add_asset_support(
			RuntimeOrigin::signed(1),
			H256::from_low_u64_be(123),
			Asset::Fungible(1),
		)
		.unwrap();

		OpenRollup::deposit(
			RuntimeOrigin::signed(2),
			H256::from_low_u64_be(123),
			AssetValue::Currency(10),
		)
		.unwrap();
		OpenRollup::set_inactive(RuntimeOrigin::signed(1), H256::from_low_u64_be(123)).unwrap();
		assert_ok!(OpenRollup::exit(RuntimeOrigin::signed(2), H256::from_low_u64_be(123)));
	});
}

#[test]
fn zkapp_submit_batch_should_work() {
	let program_hash = H256::from_low_u64_be(123);
	let account_id = 1;
	let signed_user = RuntimeOrigin::signed(account_id);
	let submitter = 2;
	let signed_submiter = RuntimeOrigin::signed(submitter);
	let empty_state_root = H256::from_low_u64_be(0);
	let state_root_1 = H256::from_low_u64_be(1);

	new_test_ext().execute_with(|| {
		OpenRollup::zkapp_register(
			signed_user.clone(),
			program_hash,
			ZkvmType::Fake,
			submitter,
			empty_state_root,
		)
		.unwrap();

		// add_asset_support
		OpenRollup::add_asset_support(signed_user.clone(), program_hash, Asset::Fungible(1))
			.unwrap();
		OpenRollup::add_asset_support(signed_user.clone(), program_hash, Asset::Nonfungible(1))
			.unwrap();

		// deposit
		OpenRollup::deposit(signed_user.clone(), program_hash, AssetValue::Currency(10)).unwrap();
		OpenRollup::deposit(signed_user.clone(), program_hash, AssetValue::Fungible(1, 10))
			.unwrap();
		OpenRollup::deposit(
			signed_user.clone(),
			program_hash,
			AssetValue::Nonfungible(1, bounded_vec![1, 2]),
		)
		.unwrap();

		// submit_batch
		let zk_proof = vec![1, 2, 3];
		assert_ok!(OpenRollup::submit_batch(
			signed_submiter.clone(),
			program_hash,
			empty_state_root,
			state_root_1,
			3,
			vec![
				Operation::Deposit(1, AssetValue::Currency(10)),
				Operation::Deposit(1, AssetValue::Fungible(1, 10)),
				Operation::Deposit(1, AssetValue::Nonfungible(1, bounded_vec![1, 2])),
				Operation::Transfer(1, 2, AssetValue::Fungible(1, 2)),
				Operation::Transfer(1, 2, AssetValue::Nonfungible(1, bounded_vec![2])),
			],
			zk_proof,
		));

		let account = ZkappsAccounts::<Test>::try_get(program_hash, account_id).unwrap();

		assert_eq!(account.assets.len(), 3);
		assert_eq!(account.assets.first().unwrap(), &AssetValue::Currency(10));
		assert_eq!(account.assets.get(1).unwrap(), &AssetValue::Fungible(1, 8));
		assert_eq!(account.assets.last().unwrap(), &AssetValue::Nonfungible(1, bounded_vec![1]));
	});
}
