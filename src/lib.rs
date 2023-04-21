//! # Open Rollup Pallet

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

mod fix_nonfungible;
mod functions;
mod types;
mod verifier;

use fix_nonfungible::Transfer as NonfungibleTransfer;
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungibles::{
				self, Create as FungibleCreate, Mutate as FungibleMutate,
				Transfer as FungibleTransfer,
			},
			nonfungibles::{
				Create as NonfungibleCreate, Inspect as NonfungibleInspect,
				Mutate as NonfungibleMutate,
			},
			ExistenceRequirement,
		},
		Currency,
	},
	PalletId,
};

use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::traits::{AccountIdConversion, StaticLookup};
use sp_std::vec::Vec;

pub use pallet::*;
pub use types::*;
pub use verifier::{FakeVerifier, MidenVerifier, Verifier};
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AssetId, CollectionId, ItemId, ProgramHash, StateRoot> {
		fn asset(i: u32) -> AssetId;
		fn collection(i: u32) -> CollectionId;
		fn item(i: u32) -> ItemId;
		fn program_hash(i: [u8; 32]) -> ProgramHash;
		fn state_root(i: [u8; 32]) -> StateRoot;
	}
	#[cfg(feature = "runtime-benchmarks")]
	impl<
			AssetId: From<u32>,
			CollectionId: From<u32>,
			ItemId: From<u32>,
			ProgramHash: From<[u8; 32]>,
			StateRoot: From<[u8; 32]>,
		> BenchmarkHelper<AssetId, CollectionId, ItemId, ProgramHash, StateRoot> for ()
	{
		fn asset(i: u32) -> AssetId {
			i.into()
		}
		fn collection(i: u32) -> CollectionId {
			i.into()
		}
		fn item(i: u32) -> ItemId {
			i.into()
		}
		fn program_hash(i: [u8; 32]) -> ProgramHash {
			i.into()
		}
		fn state_root(i: [u8; 32]) -> StateRoot {
			i.into()
		}
	}

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The zkapp's program_hash.
		type ProgramHash: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Ord
			+ Default
			+ Copy
			+ sp_std::hash::Hash
			+ AsRef<[u8]>
			+ AsMut<[u8]>
			+ MaxEncodedLen;

		/// The output of zkapp's state_root.
		type StateRoot: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Ord
			+ Default
			+ Copy
			+ sp_std::hash::Hash
			+ AsRef<[u8]>
			+ AsMut<[u8]>
			+ MaxEncodedLen;

		/// ID of this pallet.
		///
		/// Only used to derive the pallets account.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		/// fungibles assets
		type Fungibles: fungibles::Transfer<Self::AccountId>
			+ FungibleMutate<Self::AccountId>
			+ FungibleCreate<Self::AccountId>;

		/// nonfungibles assets
		type Nonfungibles: fix_nonfungible::Transfer<Self::AccountId>
			+ NonfungibleCreate<Self::AccountId>
			+ NonfungibleMutate<Self::AccountId>
			+ NonfungibleInspect<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The maximum allowable length in bytes for storage keys.
		type MaxStorageKeyLen: Get<u32>;

		/// The maximum of the assets one zkapp supported.
		#[pallet::constant]
		type AssetsLimit: Get<u32>;

		/// The maximum of the items one Account supported.
		#[pallet::constant]
		type AssetsItemLimit: Get<u32>;

		/// The maximum of the L1 operations one zkapp supported.
		#[pallet::constant]
		type L1OperationLimit: Get<u32>;

		/// The maximum of the items of one nonfungible asset
		#[pallet::constant]
		type NonfungibleItemLimit: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type Helper: BenchmarkHelper<
			AssetIdOf<Self, I>,
			CollectionIdOf<Self, I>,
			ItemIdOf<Self, I>,
			Self::ProgramHash,
			Self::StateRoot,
		>;
	}

	#[pallet::storage]
	pub(super) type Zkapps<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::ProgramHash, ZkappOf<T, I>>;

	#[pallet::storage]
	pub(super) type ZkappsAccounts<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::ProgramHash,
		Blake2_128Concat,
		T::AccountId,
		AccountOf<T, I>,
	>;

	#[pallet::storage]
	pub(super) type ZkappsExit<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Blake2_128Concat, T::ProgramHash, Blake2_128Concat, T::AccountId, bool>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		ZkappRegister(ZkvmType, T::ProgramHash),
		AddAssetSupport(T::ProgramHash, AssetOf<T, I>),
		ChangeSubmitter(T::ProgramHash, T::AccountId),
		SetInactive(T::ProgramHash),
		Deposited(T::ProgramHash, T::AccountId, AssetValueOf<T, I>),
		Withdrawed(T::ProgramHash, T::AccountId, AssetValueOf<T, I>),
		MoveAsset(T::ProgramHash, T::ProgramHash, T::AccountId, AssetValueOf<T, I>),
		Exit(T::ProgramHash, T::AccountId),
		SubmitBatch(T::ProgramHash, T::StateRoot, T::StateRoot, Vec<OperationOf<T, I>>),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		DuplicateApp,
		BadMetadata,
		NoProgram,
		SameZkapp,
		DuplicateSupportAsset,
		AssetsLimitExceed,
		NotOwner,
		Inactive,
		NotInactive,
		NotSupportAsset,
		NotAssetOwner,
		NotSubmitter,
		L1OperationLimitExceed,
		NoUserInProgram,
		HasExit,
		InvalidStateRoot,
		InvalidProof,
		InvalidBatchParams,
		NoEnoughAssets,
		NoAccount,
		InvalidAssets,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(T::WeightInfo::zkapp_register())]
		pub fn zkapp_register(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			zkvm_type: ZkvmType,
			submitter: AccountIdLookupOf<T>,
			empty_state_root: T::StateRoot,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let submitter = T::Lookup::lookup(submitter)?;

			ensure!(!Zkapps::<T, I>::contains_key(program_hash), Error::<T, I>::DuplicateApp);

			let mut supported_assets: SupportedAssetsOf<T, I> = Default::default();
			supported_assets
				.try_push(Asset::Currency)
				.map_err(|_| Error::<T, I>::BadMetadata)?;

			Zkapps::<T, I>::insert(
				program_hash,
				Zkapp {
					zkvm_type: zkvm_type.clone(),
					owner,
					submitter,
					is_inactive: false,
					state_root: empty_state_root,
					supported_assets,
					l1_operations: Vec::new().try_into().map_err(|_| Error::<T, I>::BadMetadata)?,
				},
			);
			Self::deposit_event(Event::ZkappRegister(zkvm_type, program_hash));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::add_asset_support())]
		pub fn add_asset_support(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			asset: AssetOf<T, I>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			ensure!(owner == zkapp.owner, Error::<T, I>::NotOwner);
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(!zkapp.supported_assets.contains(&asset), Error::<T, I>::DuplicateSupportAsset);

			zkapp
				.supported_assets
				.try_push(asset.clone())
				.map_err(|_| Error::<T, I>::AssetsLimitExceed)?;
			Zkapps::<T, I>::insert(program_hash, zkapp);
			Self::deposit_event(Event::AddAssetSupport(program_hash, asset));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::change_submitter())]
		pub fn change_submitter(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			submitter: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			ensure!(owner == zkapp.owner, Error::<T, I>::NotOwner);
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			let submitter = T::Lookup::lookup(submitter)?;
			zkapp.submitter = submitter.clone();

			Zkapps::<T, I>::insert(program_hash, zkapp);
			Self::deposit_event(Event::ChangeSubmitter(program_hash, submitter));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_inactive())]
		pub fn set_inactive(origin: OriginFor<T>, program_hash: T::ProgramHash) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			ensure!(owner == zkapp.owner, Error::<T, I>::NotOwner);
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			zkapp.is_inactive = true;
			Zkapps::<T, I>::insert(program_hash, zkapp);
			Self::deposit_event(Event::SetInactive(program_hash));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::deposit())]
		pub fn deposit(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			asset_value: AssetValueOf<T, I>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			let asset = asset_value.clone().into();
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(zkapp.supported_assets.contains(&asset), Error::<T, I>::NotSupportAsset);

			Self::user_deposit(user.clone(), asset_value.clone())?;

			zkapp
				.l1_operations
				.try_push(Operation::Deposit(user.clone(), asset_value.clone()))
				.map_err(|_| Error::<T, I>::L1OperationLimitExceed)?;
			Zkapps::<T, I>::insert(program_hash, zkapp);

			Self::deposit_event(Event::Deposited(program_hash, user, asset_value));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::withdraw())]
		pub fn withdraw(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			asset_value: AssetValueOf<T, I>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			let asset = asset_value.clone().into();
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(zkapp.supported_assets.contains(&asset), Error::<T, I>::NotSupportAsset);

			// check user balance
			let account = ZkappsAccounts::<T, I>::try_get(program_hash, user.clone())
				.map_err(|_| Error::<T, I>::NoEnoughAssets)?;
			ensure!(
				Self::check_has_enough_asset(&account, &asset_value),
				Error::<T, I>::NoEnoughAssets
			);

			zkapp
				.l1_operations
				.try_push(Operation::Withdraw(user.clone(), asset_value.clone()))
				.map_err(|_| Error::<T, I>::L1OperationLimitExceed)?;
			Zkapps::<T, I>::insert(program_hash, zkapp);

			Self::deposit_event(Event::Withdrawed(program_hash, user, asset_value));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::move_asset())]
		pub fn move_asset(
			origin: OriginFor<T>,
			from_program_hash: T::ProgramHash,
			to_program_hash: T::ProgramHash,
			asset_value: AssetValueOf<T, I>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let mut from_zkapp =
				Zkapps::<T, I>::try_get(from_program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			let to_zkapp =
				Zkapps::<T, I>::try_get(to_program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			let asset = asset_value.clone().into();
			ensure!(from_program_hash != to_program_hash, Error::<T, I>::SameZkapp);
			ensure!(!from_zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(!to_zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(from_zkapp.supported_assets.contains(&asset), Error::<T, I>::NotSupportAsset);
			ensure!(to_zkapp.supported_assets.contains(&asset), Error::<T, I>::NotSupportAsset);

			// check user balance
			let account = ZkappsAccounts::<T, I>::try_get(from_program_hash, user.clone())
				.map_err(|_| Error::<T, I>::NoEnoughAssets)?;
			ensure!(
				Self::check_has_enough_asset(&account, &asset_value),
				Error::<T, I>::NoEnoughAssets
			);

			from_zkapp
				.l1_operations
				.try_push(Operation::Move(user.clone(), to_program_hash, asset_value.clone()))
				.map_err(|_| Error::<T, I>::L1OperationLimitExceed)?;
			Zkapps::<T, I>::insert(from_program_hash, from_zkapp);

			Self::deposit_event(Event::MoveAsset(
				from_program_hash,
				to_program_hash,
				user,
				asset_value,
			));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::exit())]
		pub fn exit(origin: OriginFor<T>, program_hash: T::ProgramHash) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			ensure!(zkapp.is_inactive, Error::<T, I>::NotInactive);
			ensure!(
				ZkappsExit::<T, I>::try_get(program_hash, user.clone()).is_err(),
				<Error<T, I>>::HasExit
			);

			// withdraw from user balance
			let account = ZkappsAccounts::<T, I>::try_get(program_hash, user.clone());
			if let Ok(account) = account {
				for asset_value in account.assets {
					Self::user_withdraw(user.clone(), asset_value.clone())?;
				}
			}

			// withdraw from L1_operations
			for op in zkapp.l1_operations {
				match op {
					Operation::Deposit(op_user, asset_value) if op_user == user => {
						Self::user_withdraw(user.clone(), asset_value.clone())?;
					},
					_ => (),
				}
			}

			ZkappsExit::<T, I>::insert(program_hash, user.clone(), true);

			Self::deposit_event(Event::Exit(program_hash, user));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::submit_batch(operations.len() as u32))]
		pub fn submit_batch(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			old_state_root: T::StateRoot,
			new_state_root: T::StateRoot,
			l1_operations_pos: u32,
			operations: Vec<OperationOf<T, I>>,
			zk_proof: Vec<u8>,
		) -> DispatchResult {
			let submitter = ensure_signed(origin)?;
			let mut zkapp =
				Zkapps::<T, I>::try_get(program_hash).map_err(|_| Error::<T, I>::NoProgram)?;
			ensure!(!zkapp.is_inactive, Error::<T, I>::Inactive);
			ensure!(zkapp.submitter == submitter, Error::<T, I>::NotSubmitter);
			ensure!(zkapp.state_root == old_state_root, Error::<T, I>::InvalidStateRoot);

			// the program of the zkapp must handle the l1_operations queue
			// and the front operations should match the l1_operations queue
			ensure!(
				u32::try_from(zkapp.l1_operations.len())
					.map_err(|_| Error::<T, I>::InvalidBatchParams)? >=
					l1_operations_pos,
				Error::<T, I>::InvalidBatchParams
			);
			for op_index in 0..l1_operations_pos {
				ensure!(
					zkapp.l1_operations[op_index as usize] == operations[op_index as usize],
					Error::<T, I>::InvalidBatchParams
				);
			}

			// println!("{:?}, {:?}, {:?}", l1_operations_pos, operations, zkapp.l1_operations);

			// vefify the zk proof
			let zk_inputs = old_state_root.as_ref();
			let zk_outputs = ProofOutput {
				operations: operations.clone(),
				state_root: new_state_root,
				l1_operations_pos,
			}
			.encode();
			let zk_outputs = zk_outputs.as_ref();

			match zkapp.zkvm_type {
				ZkvmType::Fake => {
					FakeVerifier::vefify(program_hash.as_ref(), zk_inputs, &zk_proof, zk_outputs)
						.map_err(|_| Error::<T, I>::InvalidProof)?;
				},
				ZkvmType::Miden => {
					MidenVerifier::vefify(program_hash.as_ref(), zk_inputs, &zk_proof, zk_outputs)
						.map_err(|_| Error::<T, I>::InvalidProof)?;
				},
			};

			// remove the l1_operations which are executed in the batch
			zkapp.l1_operations =
				BoundedVec::try_from(zkapp.l1_operations[l1_operations_pos as usize..].to_vec())
					.map_err(|_| Error::<T, I>::InvalidBatchParams)?;

			// execute operations
			for (i, op) in operations.iter().enumerate() {
				match op {
					Operation::Deposit(user, asset_value) => {
						// only execution of L1 Deposit transaction can Deposit Operation
						ensure!(i < l1_operations_pos as usize, Error::<T, I>::InvalidBatchParams);

						let mut account: AccountOf<T, I>;
						if let Ok(_account) = ZkappsAccounts::<T, I>::try_get(program_hash, user) {
							account = _account;
						} else {
							account = Account { user: user.clone(), assets: Default::default() };
						}
						Self::add_user_asset(&mut account, asset_value)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user, account);
					},
					Operation::Withdraw(user, asset_value) => {
						let mut account = ZkappsAccounts::<T, I>::try_get(program_hash, user)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::user_withdraw(user.clone(), asset_value.clone())?;
						Self::reduce_user_asset(&mut account, asset_value)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user, account);
					},
					Operation::Move(user, to_program_hash, asset_value) => {
						// reduce user asset_value
						let mut account = ZkappsAccounts::<T, I>::try_get(program_hash, user)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account, asset_value)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user, account);

						// add deposit L1 operation and deposit event to to_program
						let mut to_zkapp = Zkapps::<T, I>::try_get(to_program_hash)
							.map_err(|_| Error::<T, I>::NoProgram)?;
						to_zkapp
							.l1_operations
							.try_push(Operation::Deposit(user.clone(), asset_value.clone()))
							.map_err(|_| Error::<T, I>::L1OperationLimitExceed)?;
						Zkapps::<T, I>::insert(to_program_hash, to_zkapp);
						Self::deposit_event(Event::Deposited(
							*to_program_hash,
							user.clone(),
							asset_value.clone(),
						));
					},
					Operation::Transfer(from_user, to_user, asset_value) => {
						let mut from_account =
							ZkappsAccounts::<T, I>::try_get(program_hash, from_user)
								.map_err(|_| Error::<T, I>::NoAccount)?;

						let mut to_account: AccountOf<T, I>;
						if let Ok(account) = ZkappsAccounts::<T, I>::try_get(program_hash, to_user)
						{
							to_account = account;
						} else {
							to_account =
								Account { user: to_user.clone(), assets: Default::default() };
						}
						Self::reduce_user_asset(&mut from_account, asset_value)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, from_user, from_account);

						Self::add_user_asset(&mut to_account, asset_value)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, to_user, to_account);
					},
					Operation::Swap(user_1, asset_value_1, user_2, asset_value_2) => {
						// modify user_1 assets
						let mut account_1 = ZkappsAccounts::<T, I>::try_get(program_hash, user_1)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account_1, asset_value_1)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						Self::add_user_asset(&mut account_1, asset_value_2)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user_1, account_1);

						// modify user_2 assets
						let mut account_2 = ZkappsAccounts::<T, I>::try_get(program_hash, user_2)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account_2, asset_value_2)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						Self::add_user_asset(&mut account_2, asset_value_1)
							.map_err(|_| Error::<T, I>::InvalidAssets)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user_2, account_2);
					},
				}
			}

			// save zkapp's new_state_root
			zkapp.state_root = new_state_root;
			Zkapps::<T, I>::insert(program_hash, zkapp);

			Self::deposit_event(Event::SubmitBatch(
				program_hash,
				old_state_root,
				new_state_root,
				operations,
			));
			Ok(())
		}
	}
}
