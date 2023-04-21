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
