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

//! Various basic types for use in the open rollup pallet.

use super::*;
use frame_support::{pallet_prelude::*, BoundedVec};

// type alias
pub(super) type AccountIdLookupOf<T> =
	<<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type ProgramHashOf<T, I = ()> = <T as Config<I>>::ProgramHash;
pub(super) type StateRootOf<T, I = ()> = <T as Config<I>>::StateRoot;

pub(super) type CurrencyBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub(super) type FungibleBalanceOf<T, I=()> = <<T as Config<I>>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;
pub(super) type AssetIdOf<T, I = ()> = <<T as Config<I>>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;
pub(super) type CollectionIdOf<T, I = ()> =
	<<T as Config<I>>::Nonfungibles as fix_nonfungible::Transfer<
		<T as frame_system::Config>::AccountId,
	>>::CollectionId;
pub(super) type ItemIdOf<T, I = ()> =
	<<T as Config<I>>::Nonfungibles as fix_nonfungible::Transfer<
		<T as frame_system::Config>::AccountId,
	>>::ItemId;

pub(super) type AssetsLimitOf<T, I = ()> = <T as Config<I>>::AssetsLimit;
pub(super) type AssetsItemLimitOf<T, I = ()> = <T as Config<I>>::AssetsItemLimit;
pub(super) type L1OperationLimitOf<T, I = ()> = <T as Config<I>>::L1OperationLimit;

pub(super) type ZkappOf<T, I> = Zkapp<
	StateRootOf<T, I>,
	AccountIdOf<T>,
	AssetIdOf<T, I>,
	CollectionIdOf<T, I>,
	AssetValueOf<T, I>,
	ProgramHashOf<T, I>,
	AssetsLimitOf<T, I>,
	L1OperationLimitOf<T, I>,
>;

pub(super) type AssetOf<T, I = ()> = Asset<AssetIdOf<T, I>, CollectionIdOf<T, I>>;
pub(super) type SupportedAssetsOf<T, I> = BoundedVec<AssetOf<T, I>, AssetsLimitOf<T, I>>;
pub(super) type AssetValueOf<T, I> = AssetValue<
	CurrencyBalanceOf<T, I>,
	AssetIdOf<T, I>,
	FungibleBalanceOf<T, I>,
	CollectionIdOf<T, I>,
	ItemIdOf<T, I>,
>;

pub(super) type OperationOf<T, I> =
	Operation<AccountIdOf<T>, AssetValueOf<T, I>, ProgramHashOf<T, I>>;

pub(super) type AccountOf<T, I> = Account<
	AccountIdOf<T>,
	CurrencyBalanceOf<T, I>,
	AssetIdOf<T, I>,
	FungibleBalanceOf<T, I>,
	CollectionIdOf<T, I>,
	ItemIdOf<T, I>,
	AssetsItemLimitOf<T, I>,
>;

/// Asset types supported by open rollup pallet.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Asset<AssetId, CollectionId> {
	Currency,
	Fungible(AssetId),
	Nonfungible(CollectionId),
}

/// One specific asset, include amount.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId> {
	Currency(CurrencyBalance),
	Fungible(AssetId, FungibleBalance),
	Nonfungible(CollectionId, BoundedVec<ItemId, ConstU32<100>>),
}

/// Implement From trait, from AssetValue to Asset Enum.
impl<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId>
	From<AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId>>
	for Asset<AssetId, CollectionId>
{
	fn from(
		asset_value: AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId>,
	) -> Asset<AssetId, CollectionId> {
		match asset_value {
			AssetValue::Currency(_) => Asset::Currency,
			AssetValue::Fungible(asset_id, _) => Asset::Fungible(asset_id),
			AssetValue::Nonfungible(collection_id, _) => Asset::Nonfungible(collection_id),
		}
	}
}

/// Supported operations of L1 and L2 operations.
///
/// User's L1 Txs can trigger Deposit, Withdraw, Move operations,
/// L2 Txs can trigger all operations.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Operation<AccountId, AssetValue, ProgramHash> {
	Deposit(AccountId, AssetValue),
	Withdraw(AccountId, AssetValue),
	Move(AccountId, ProgramHash, AssetValue),
	Transfer(AccountId, AccountId, AssetValue),
	Swap(AccountId, AssetValue, AccountId, AssetValue),
}

/// Supported zkvm types.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ZkvmType {
	/// Fake verifier, doesn't verify the proof, for test.
	Fake,
	/// Miden verifier type.
	Miden,
}

/// One zkapp's saved data.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(AssetsLimit, L1OperationLimit))]
pub struct Zkapp<
	StateRoot,
	AccountId,
	AssetId,
	CollectionId,
	AssetValue,
	ProgramHash,
	AssetsLimit: Get<u32>,
	L1OperationLimit: Get<u32>,
> {
	/// The zkapp's zkvm type
	pub(super) zkvm_type: ZkvmType,
	/// The Zkapp's owner, who can change `submitter`, `is_inactive`.
	pub(super) owner: AccountId,
	/// The account who can submit one batch.
	pub(super) submitter: AccountId,
	/// Whether the zkapp is inactive.
	pub(super) is_inactive: bool,
	/// Root of the state (e.g. off-chain's users tree) of the zkapp.
	pub(super) state_root: StateRoot,
	/// supported Assets of the zkapp.
	pub(super) supported_assets: BoundedVec<Asset<AssetId, CollectionId>, AssetsLimit>,
	/// L1 operation queue trigger by L1 Txs.
	pub(super) l1_operations:
		BoundedVec<Operation<AccountId, AssetValue, ProgramHash>, L1OperationLimit>,
}

/// User data of one zkapp.
///
/// Include user's assets in on zkapp.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(AssetsItemLimit))]
pub struct Account<
	AccountId,
	CurrencyBalance,
	AssetId,
	FungibleBalance,
	CollectionId,
	ItemId,
	AssetsItemLimit: Get<u32>,
> {
	/// User AccountId.
	pub(super) user: AccountId,

	/// User's Assets in one zkapp.
	pub(super) assets: BoundedVec<
		AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId>,
		AssetsItemLimit,
	>,
}

/// The output of zk-program's execution
#[derive(Clone, Encode, Decode, Eq, PartialEq)]
pub struct ProofOutput<Operation, StateRoot> {
	/// Operations triggered by one batch's txs.
	pub operations: Vec<Operation>,
	/// The new root of the state.
	pub state_root: StateRoot,
	/// The number of L1 operations the execution include.
	pub l1_operations_pos: u32,
}
