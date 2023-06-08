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

		/// The zkapp's program hash.
		///
		/// Any zkapp's program shoud be reduced to a single 32-byte value, called program-hash,
		/// it ensures that the verifier verifies the execution of a specific program.
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

		/// The output root of zkapp's state tree.
		///
		/// The state tree maintained off-chain by the zkapp.
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

		/// The currency asset mechanism.
		type Currency: Currency<Self::AccountId>;

		/// The fungibles assets mechanism.
		type Fungibles: fungibles::Transfer<Self::AccountId>
			+ FungibleMutate<Self::AccountId>
			+ FungibleCreate<Self::AccountId>;

		/// The nonfungibles assets mechanism.
		type Nonfungibles: fix_nonfungible::Transfer<Self::AccountId>
			+ NonfungibleCreate<Self::AccountId>
			+ NonfungibleMutate<Self::AccountId>
			+ NonfungibleInspect<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

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
	/// Map of `program_hash` to `Zkapp`.
	///
	/// Used to retrieve the zkapp's information.
	pub(super) type Zkapps<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::ProgramHash, ZkappOf<T, I>>;

	#[pallet::storage]
	/// Map of `program_hash` and `accountId` to Account.
	///
	/// Used to retrieve one user's assets's balances in one zkapp.
	/// TODO: If the zkapp's program can provide EXIT sub-program, this Map is not necessary.
	pub(super) type ZkappsAccounts<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::ProgramHash,
		Blake2_128Concat,
		T::AccountId,
		AccountOf<T, I>,
	>;

	#[pallet::storage]
	/// Map of `program_hash` and `accountId` to Exit status.
	///
	/// Used to retrieve whether one user has exited the zkapp.
	pub(super) type ZkappsExit<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Blake2_128Concat, T::ProgramHash, Blake2_128Concat, T::AccountId, bool>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	/// All events that can be emitted by Pallet function.
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A zkapp registered in the pallet.
		/// \[zkvm_type, program_hash\]
		ZkappRegister(ZkvmType, T::ProgramHash),

		/// A supported asset added into a zkapp.
		/// \[program_hash, asset\]
		AddAssetSupport(T::ProgramHash, AssetOf<T, I>),

		/// The submitter changed of o zkapp.
		/// \[program_hash, submitter\]
		ChangeSubmitter(T::ProgramHash, T::AccountId),

		/// One zkapp's status has been setted to inactive.
		/// \[program_hash\]
		SetInactive(T::ProgramHash),

		/// A user deposited asset into a zkapp.
		/// \[program_hash, account_id, asset_value\]
		Deposited(T::ProgramHash, T::AccountId, AssetValueOf<T, I>),

		/// A user withdrawed asset from a zkapp.
		/// \[program_hash, account_id, asset_value\]
		Withdrawed(T::ProgramHash, T::AccountId, AssetValueOf<T, I>),

		/// A user move asset from a zkapp to another zkapp.
		/// \[from_program_hash, to_program_hash, account_id, asset_value\]
		MoveAsset(T::ProgramHash, T::ProgramHash, T::AccountId, AssetValueOf<T, I>),

		/// A user exited from a zkapp.
		/// \[program_hash, account_id\]
		Exit(T::ProgramHash, T::AccountId),

		/// A batch of a zkapp submited into the pallet.
		/// \[program_hash, old_state_root, new_state_root, operations\]
		SubmitBatch(T::ProgramHash, T::StateRoot, T::StateRoot, Vec<OperationOf<T, I>>),
	}

	#[pallet::error]
	/// All errors that can be returned by Pallet functions.
	pub enum Error<T, I = ()> {
		/// The zkapp has been registered before.
		DuplicateApp,
		/// BoundedVec error.
		BoundedVecInvalid,
		/// No zkapp registered for program_hash.
		NoProgram,
		/// Move asset to the same zkapp.
		SameZkapp,
		/// The supported asset has been added before.
		DuplicateSupportAsset,
		/// The number of supported assets exceed.
		AssetsLimitExceed,
		/// The user sent the Tx is not the zkapp's owner
		NotOwner,
		/// The tx is not allowed as zkapp is inactive.
		Inactive,
		/// The exit tx is not allowed as zkapp is not inactive.
		NotInactive,
		/// The asset is not supported by the zkapp.
		NotSupportAsset,
		/// The asset is not owned by the user.
		NotAssetOwner,
		/// Only submitter of the zkapp can submit batch.
		NotSubmitter,
		/// The number of L1 operations exceed.
		L1OperationLimitExceed,
		/// A user can only exit once from a zkapp.
		HasExit,
		/// The old_state_root of the batch submited should equal the current state_root of the
		/// zkapp.
		InvalidStateRoot,
		/// A proof could not be verified.
		InvalidProof,
		/// The l1_operations or l1_operations_pos is invalid.
		InvalidBatchParams,
		/// No enough assets when user withdraw or move assets.
		NoEnoughAssets,
		/// The operations of the batch submited include unknowned account by the zkapp.
		NoAccount,
		/// The operations of the batch submited include invalid assets.
		InvalidAssets,
	}

	#[pallet::call]
	/// Contains all user-facing functions.
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(T::WeightInfo::zkapp_register())]
		/// Register a zkapp onchain.
		///
		/// - `origin`: the sender as the zkapp's owner who can change submitter and inactive
		/// status
		/// - `program_hash`: zkapp's program hash, a zk-program should be reduced to a single
		///   32-byte value,
		/// called program hash.  This ensures that the verifier verifies execution of a specific
		/// program.
		/// - `submitter`: who can submit one batch for the zkapp.
		/// - `empty_state_root`: the root (hash) of empty state of the zkapp.
		///
		/// Emits `ZkappRegister` event when successful.
		///
		/// Weight: `O(1)`
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
				.map_err(|_| Error::<T, I>::BoundedVecInvalid)?;

			Zkapps::<T, I>::insert(
				program_hash,
				Zkapp {
					zkvm_type: zkvm_type.clone(),
					owner,
					submitter,
					is_inactive: false,
					state_root: empty_state_root,
					supported_assets,
					l1_operations: Vec::new()
						.try_into()
						.map_err(|_| Error::<T, I>::BoundedVecInvalid)?,
				},
			);
			Self::deposit_event(Event::ZkappRegister(zkvm_type, program_hash));
			Ok(())
		}

		/// Add a asset supported by a zkapp, can only be called by owner of the zkapp.
		///
		/// - `origin`: the sender who is the zkapp' owner.
		/// - `program_hash`: the program hash of the zkapp's program.
		/// - `asset`: the asset the zkapp supported.
		///
		/// Emits `AddAssetSupport` event when successful.
		///
		/// Weight: `O(1)`
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

		/// Change the submitter of one zkapp, can only be called by owner of the zkapp.
		///
		/// Emits `ChangeSubmitter` event when successful.
		///
		/// Weight: `O(1)`
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

		/// Set the zkapp is inactive, can only be called by owner of the zkapp.
		/// If be called, only `exit` is allowed for the zkapp.
		///
		/// TODO: If a zkapp has a fraud program, bugs, or does not submit batch txs for a long
		/// time, other mechanisms are required to set this program as inactive,
		/// so that users can fully exit this program to withdraw their assets.
		///
		/// Emits `SetInactive` event when successful.
		///
		/// Weight: `O(1)`
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

		/// Deposit asset to a zkapp, it is a L1 transaction, and trigger `Deposit` operation.
		///
		/// Save `Deposit` operation into `l1_operations` queue, the zkapp's program (off-chain)
		/// should read `l1_operations` as the program's inputs when execution.
		/// The asset will be added to the user's assets of the pallet when `submit_batch` is
		/// called.
		///
		/// Emits `Deposited` event when successful.
		///
		/// Weight: `O(1)`
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

		/// Withdraw asset from a zkapp, it is a L1 transaction, and trigger `Withdraw` operation.
		///
		/// Save `Withdraw` operation into `l1_operations` queue, the zkapp's program (off-chain)
		/// should read `l1_operations` as the program's inputs when execution.
		/// The asset will be withdrawed to the user when `submit_batch` is called.
		///
		/// Emits `Withdrawed` event when successful.
		///
		/// Weight: `O(1)`
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

		/// Move asset from a zkapp to another zkapp, it is a L1 transaction, and trigger `Move`
		/// operation.
		///
		/// Save `Move` operation into `l1_operations` queue, the zkapp's program (off-chain)
		/// should read `l1_operations` as the program's inputs when execution.
		/// The asset will be moved when `submit_batch` is called.
		///
		/// Emits `Move` event when successful.
		///
		/// Weight: `O(1)`
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

		/// User exit a zkapp fully, it can be called only when the status of this program is
		/// inactive.
		///
		/// When called, user's assets (saved in ZkappsAccounts DoubleMap) are transfered to user,
		/// if L1 operations queue `l1_operations` of the zkapp has user's `Deposit` operations,
		/// they also are transfered to user.
		///
		/// Emits `Exit` event when successful.
		///
		/// Weight: `O(max(m, n))` where
		/// - `m = account.assets.len()`
		/// - `n = zkapp.l1_operations.len()`
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

			// withdraw from L1 operations
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

		/// Submit a batch for a zkapp, can only be called by submitter of the zkapp.
		///
		/// A zkapp's program (off-chain component) collects L1 operations from
		/// `zkapp.l1_operations` or events onchain and L2 transactions from users (interact with
		/// off-chain component), Every execution of a zkapp's program, use `old_state_root` as the
		/// public inputs, l1_operations, L2 transactions and state tree as secret inputs,
		/// the outputs of a execution should include new_state_root,
		/// all operations and the number of the l1_operations included.
		///
		/// - `origin`: submitter of the zkapp.
		/// - `program_hash`: program hash of the zkapp.
		/// - `old_state_root`: state root of state tree before execution.
		/// - `new_state_root`: state root of state tree after execution.
		/// - `l1_operations_pos`: the number of the L1 operations included in the execution.
		/// - `operations`: all operations generated by the execution of zkapp's program this time.
		/// - `zk_proof`: the proof generated during program execution.
		///
		/// Emits `SubmitBatch` event when successful.
		///
		/// Weight: `O(operations.len())`
        #[allow(clippy::too_many_arguments)]
		#[pallet::weight(T::WeightInfo::submit_batch(operations.len() as u32))]
		pub fn submit_batch(
			origin: OriginFor<T>,
			program_hash: T::ProgramHash,
			old_state_root: T::StateRoot,
			new_state_root: T::StateRoot,
			l1_operations_pos: u32,
			operations: Vec<OperationOf<T, I>>,
			zk_proof: Vec<u8>,
            zk_outputs: Option<Vec<u8>>,
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

			// verify the zk proof
			let zk_inputs = old_state_root.as_ref();
            let zk_outputs = match zk_outputs {
                Some(outs) => outs,
                None => ProofOutput {
                    operations: operations.clone(),
                    state_root: new_state_root,
                    l1_operations_pos,
                }.encode()
            };
			let zk_outputs = zk_outputs.as_ref();

            // println!("pallet zk_outputs: {:?}", zk_outputs);

			match zkapp.zkvm_type {
				ZkvmType::Fake => {
					FakeVerifier::verify(program_hash.as_ref(), zk_inputs, &zk_proof, zk_outputs)
						.map_err(|_| Error::<T, I>::InvalidProof)?;
				},
				ZkvmType::Miden => {
					MidenVerifier::verify(program_hash.as_ref(), zk_inputs, &zk_proof, zk_outputs)
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
						Self::add_user_asset(&mut account, asset_value)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user, account);
					},
					Operation::Withdraw(user, asset_value) => {
						let mut account = ZkappsAccounts::<T, I>::try_get(program_hash, user)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::user_withdraw(user.clone(), asset_value.clone())?;
						Self::reduce_user_asset(&mut account, asset_value)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user, account);
					},
					Operation::Move(user, to_program_hash, asset_value) => {
						// reduce user asset_value
						let mut account = ZkappsAccounts::<T, I>::try_get(program_hash, user)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account, asset_value)?;
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
						Self::reduce_user_asset(&mut from_account, asset_value)?;
						ZkappsAccounts::<T, I>::insert(program_hash, from_user, from_account);

						Self::add_user_asset(&mut to_account, asset_value)?;
						ZkappsAccounts::<T, I>::insert(program_hash, to_user, to_account);
					},
					Operation::Swap(user_1, asset_value_1, user_2, asset_value_2) => {
						// modify user_1 assets
						let mut account_1 = ZkappsAccounts::<T, I>::try_get(program_hash, user_1)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account_1, asset_value_1)?;
						Self::add_user_asset(&mut account_1, asset_value_2)?;
						ZkappsAccounts::<T, I>::insert(program_hash, user_1, account_1);

						// modify user_2 assets
						let mut account_2 = ZkappsAccounts::<T, I>::try_get(program_hash, user_2)
							.map_err(|_| Error::<T, I>::NoAccount)?;
						Self::reduce_user_asset(&mut account_2, asset_value_2)?;
						Self::add_user_asset(&mut account_2, asset_value_1)?;
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
