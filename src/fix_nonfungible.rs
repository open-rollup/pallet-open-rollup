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

//! Add constraints traits for type ItemId and CollectionId for nonfungibles' Transfer trait.
//! No the constraints traits, it cannot be saved to pallet's storage.
//! The solution is ugly, it must include uniques pallet as dependency.
//! Maybe add the traits constraints for the ItemId and CollectionId types of the Inspect trait is
//! better, or seek other solution.
//! <https://github.com/paritytech/substrate/blob/polkadot-v0.9.31/frame/support/src/traits/tokens/nonfungibles.rs#L36>

use super::*;
use pallet_uniques::{Config, Pallet};

/// Trait for providing a non-fungible sets of items which can only be transferred.
pub trait Transfer<AccountId> {
	/// Type for identifying an item.
	type ItemId: Member + Parameter + MaxEncodedLen + Copy;

	/// Type for identifying a collection (an identifier for an independent collection of
	/// items).
	type CollectionId: Member + Parameter + MaxEncodedLen + Copy;

	/// Transfer `item` of `collection` into `destination` account.
	fn fix_transfer(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		destination: &AccountId,
	) -> (Option<AccountId>, DispatchResult);

	/// Mint `item` of `collection` into `destination` account.
	fn force_mint(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		destination: &AccountId,
	) -> DispatchResult;

	/// Create one collection.
	fn force_create(
		collection: &Self::CollectionId,
		who: &AccountId,
		admin: &AccountId,
	) -> DispatchResult;
}

impl<T: Config<I>, I: 'static> Transfer<<T as frame_system::Config>::AccountId> for Pallet<T, I> {
	type ItemId = T::ItemId;
	type CollectionId = T::CollectionId;

	fn fix_transfer(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		destination: &T::AccountId,
	) -> (Option<T::AccountId>, DispatchResult) {
		let owner = Self::owner(*collection, *item);
		(owner, Self::do_transfer(*collection, *item, destination.clone(), |_, _| Ok(())))
	}

	fn force_mint(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		destination: &T::AccountId,
	) -> DispatchResult {
		Self::mint_into(collection, item, destination)
	}

	fn force_create(
		collection: &Self::CollectionId,
		who: &T::AccountId,
		admin: &T::AccountId,
	) -> DispatchResult {
		Self::do_create_collection(
			*collection,
			who.clone(),
			admin.clone(),
			T::CollectionDeposit::get(),
			false,
			pallet_uniques::Event::Created {
				collection: *collection,
				creator: who.clone(),
				owner: admin.clone(),
			},
		)
	}
}
