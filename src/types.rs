//! Various basic types for use in the open rollup pallet.

use super::*;
use frame_support::{pallet_prelude::*, BoundedVec};

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

pub(super) type AssetOf<T, I> = Asset<AssetIdOf<T, I>, CollectionIdOf<T, I>>;
pub(super) type SupportedAssetsOf<T, I> = BoundedVec<AssetOf<T, I>, AssetsLimitOf<T, I>>;
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

pub(super) type AssetValueOf<T, I> = AssetValue<
	CurrencyBalanceOf<T, I>,
	AssetIdOf<T, I>,
	FungibleBalanceOf<T, I>,
	CollectionIdOf<T, I>,
	ItemIdOf<T, I>,
>;

pub(super) type OperationOf<T, I> =
	Operation<AccountIdOf<T>, AssetValueOf<T, I>, ProgramHashOf<T, I>>;

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

pub(super) type AccountOf<T, I> = Account<
	AccountIdOf<T>,
	CurrencyBalanceOf<T, I>,
	AssetIdOf<T, I>,
	FungibleBalanceOf<T, I>,
	CollectionIdOf<T, I>,
	ItemIdOf<T, I>,
	AssetsItemLimitOf<T, I>,
>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Asset<AssetId, CollectionId> {
	Currency,
	Fungible(AssetId),
	Nonfungible(CollectionId),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId> {
	Currency(CurrencyBalance),
	Fungible(AssetId, FungibleBalance),
	Nonfungible(CollectionId, BoundedVec<ItemId, ConstU32<100>>),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Operation<AccountId, AssetValue, ProgramHash> {
	Deposit(AccountId, AssetValue),
	Withdraw(AccountId, AssetValue),
	Move(AccountId, ProgramHash, AssetValue),
	Transfer(AccountId, AccountId, AssetValue),
	Swap(AccountId, AssetValue, AccountId, AssetValue),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ZkvmType {
	/// doesn't verify the proof
	Fake,
	/// use Miden verifier
	Miden,
}

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
	/// zkvm_type: miden, fake
	pub(super) zkvm_type: ZkvmType,
	/// Can change `submitter`, `is_inactive` accounts.
	pub(super) owner: AccountId,
	/// Can submitBatch for the zkapp accounts.
	pub(super) submitter: AccountId,
	/// Whether the zkapp is inactive.
	pub(super) is_inactive: bool,
	/// State root of the root of the zkapp.
	pub(super) state_root: StateRoot,
	/// supported Assets
	pub(super) supported_assets: BoundedVec<Asset<AssetId, CollectionId>, AssetsLimit>,
	/// L1 txs queue
	pub(super) l1_operations:
		BoundedVec<Operation<AccountId, AssetValue, ProgramHash>, L1OperationLimit>,
}

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
	/// user AccountId
	pub(super) user: AccountId,

	///  user's Assets in one zkapp
	pub(super) assets: BoundedVec<
		AssetValue<CurrencyBalance, AssetId, FungibleBalance, CollectionId, ItemId>,
		AssetsItemLimit,
	>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq)]
pub struct ProofOutput<Operation, StateRoot> {
	pub operations: Vec<Operation>,
	pub state_root: StateRoot,
	pub l1_operations_pos: u32,
}
