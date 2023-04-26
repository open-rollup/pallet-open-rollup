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

//! Functions for the Open Rollup pallet.

use super::*;

/// Contains all pallet-facing functions.
impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Returns the account of the pallet.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Transfer asset from user to pallet account.
	pub fn user_deposit(user: T::AccountId, asset_value: AssetValueOf<T, I>) -> DispatchResult {
		let account_id = Self::account_id();

		match asset_value {
			AssetValue::Currency(value) => {
				T::Currency::transfer(&user, &account_id, value, ExistenceRequirement::KeepAlive)?;
			},
			AssetValue::Fungible(asset_id, value) => {
				T::Fungibles::transfer(asset_id, &user, &account_id, value, false)?;
			},
			AssetValue::Nonfungible(collection_id, item_ids) =>
				for item_id in item_ids {
					let (asset_owner, result) =
						T::Nonfungibles::fix_transfer(&collection_id, &item_id, &account_id);
					let asset_owner = asset_owner.ok_or(Error::<T, I>::NotAssetOwner)?;
					ensure!(asset_owner == user, Error::<T, I>::NotAssetOwner);
					result?;
				},
		}
		Ok(())
	}

	/// Transfer asset from pallet account to user.
	pub fn user_withdraw(user: T::AccountId, asset_value: AssetValueOf<T, I>) -> DispatchResult {
		let account_id = Self::account_id();

		match asset_value {
			AssetValue::Currency(value) => {
				T::Currency::transfer(&account_id, &user, value, ExistenceRequirement::KeepAlive)?;
			},
			AssetValue::Fungible(asset_id, value) => {
				T::Fungibles::transfer(asset_id, &account_id, &user, value, false)?;
			},
			AssetValue::Nonfungible(collection_id, item_ids) =>
				for item_id in item_ids {
					let (asset_owner, result) =
						T::Nonfungibles::fix_transfer(&collection_id, &item_id, &user);
					let asset_owner = asset_owner.ok_or(Error::<T, I>::NotAssetOwner)?;
					ensure!(asset_owner == account_id, Error::<T, I>::NotAssetOwner);
					result?;
				},
		}
		Ok(())
	}

	/// Add user's asset balance in a zkapp.
	pub fn add_user_asset(
		account: &mut AccountOf<T, I>,
		asset_value: &AssetValueOf<T, I>,
	) -> Result<(), Error<T, I>> {
		let dest_asset: AssetOf<T, I> = <AssetOf<T, I>>::from(asset_value.clone());
		let mut has_asset = false;
		for exist_asset_value in &mut account.assets {
			let asset = Asset::from(exist_asset_value.clone());
			if asset == dest_asset {
				match exist_asset_value {
					AssetValue::Currency(ref mut value) => {
						if let AssetValue::Currency(add_value) = asset_value {
							*value += *add_value;
						}
					},
					AssetValue::Fungible(_, ref mut value) => {
						if let AssetValue::Fungible(_, add_value) = asset_value {
							*value += *add_value;
						}
					},
					AssetValue::Nonfungible(_, ref mut items) => {
						if let AssetValue::Nonfungible(_, add_items) = asset_value {
							for item_id in add_items {
								if !items.contains(item_id) {
									items
										.try_push(*item_id)
										.map_err(|_| Error::<T, I>::InvalidAssets)?;
								}
							}
						}
					},
				}
				has_asset = true;
				break
			}
		}
		if !has_asset {
			account
				.assets
				.try_push(asset_value.clone())
				.map_err(|_| Error::<T, I>::InvalidAssets)?;
		}

		Ok(())
	}

	/// Reduce user's asset balance in a zkapp.
	pub fn reduce_user_asset(
		account: &mut AccountOf<T, I>,
		asset_value: &AssetValueOf<T, I>,
	) -> Result<(), Error<T, I>> {
		let dest_asset: AssetOf<T, I> = <AssetOf<T, I>>::from(asset_value.clone());
		let mut has_asset = false;
		for exist_asset_value in &mut account.assets {
			let asset = Asset::from(exist_asset_value.clone());
			if asset == dest_asset {
				match exist_asset_value {
					AssetValue::Currency(ref mut value) => {
						if let AssetValue::Currency(reduce_value) = asset_value {
							if reduce_value > value {
								return Err(Error::<T, I>::InvalidAssets)
							}
							*value -= *reduce_value;
						}
					},
					AssetValue::Fungible(_, ref mut value) => {
						if let AssetValue::Fungible(_, reduce_value) = asset_value {
							if reduce_value > value {
								return Err(Error::<T, I>::InvalidAssets)
							}
							*value -= *reduce_value;
						}
					},
					AssetValue::Nonfungible(_, ref mut items) => {
						if let AssetValue::Nonfungible(_, reduce_items) = asset_value {
							for item_id in reduce_items {
								if !items.contains(item_id) {
									return Err(Error::<T, I>::InvalidAssets)
								}
							}
							items.retain(|item_id| !reduce_items.contains(item_id));
						}
					},
				}
				has_asset = true;
				break
			}
		}

		if !has_asset {
			return Err(Error::<T, I>::InvalidAssets)
		}

		Ok(())
	}

	/// Check whether user has enough asset for withdraw or move.
	pub fn check_has_enough_asset(
		account: &AccountOf<T, I>,
		asset_value: &AssetValueOf<T, I>,
	) -> bool {
		let dest_asset: AssetOf<T, I> = <AssetOf<T, I>>::from(asset_value.clone());
		for exist_asset_value in &account.assets {
			let asset = Asset::from(exist_asset_value.clone());
			if asset == dest_asset {
				match exist_asset_value {
					AssetValue::Currency(value) => {
						if let AssetValue::Currency(reduce_value) = asset_value {
							if reduce_value > value {
								return false
							}
							return true
						}
					},
					AssetValue::Fungible(_, value) => {
						if let AssetValue::Fungible(_, reduce_value) = asset_value {
							if reduce_value > value {
								return false
							}
							return true
						}
					},
					AssetValue::Nonfungible(_, items) => {
						if let AssetValue::Nonfungible(_, reduce_items) = asset_value {
							for item_id in reduce_items {
								if !items.contains(item_id) {
									return false
								}
							}
							return true
						}
					},
				}
			}
		}
		false
	}

	/// Add user's asset balance in a zkapp.
	pub fn add_zkapp_user_asset(
		program_hash: ProgramHashOf<T, I>,
		user: AccountIdOf<T>,
		asset_value: &AssetValueOf<T, I>,
	) -> Result<(), Error<T, I>> {
		let mut account: AccountOf<T, I>;
		if let Ok(_account) = ZkappsAccounts::<T, I>::try_get(program_hash, user.clone()) {
			account = _account;
		} else {
			account = Account { user: user.clone(), assets: Default::default() };
		}
		Self::add_user_asset(&mut account, asset_value)?;
		ZkappsAccounts::<T, I>::insert(program_hash, user, account);

		Ok(())
	}
}
