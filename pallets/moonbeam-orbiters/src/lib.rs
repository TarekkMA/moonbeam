// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;

/*#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;*/

pub use pallet::*;
pub use types::*;

use frame_support::pallet;
use nimbus_primitives::{AccountLookup, NimbusId};

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Imbalance};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{One, Saturating, StaticLookup};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// A type to convert between AuthorId and AccountId. This pallet wrap the lookup to allow
		/// orbiters authoring.
		type AccountLookup: AccountLookup<Self::AccountId>;

		/// Origin that is allowed to add a collator in orbiters program
		type AddCollatorOrigin: EnsureOrigin<Self::Origin>;

		/// The currency type
		type Currency: Currency<Self::AccountId>;

		/// Origin that is allowed to remove a collator from orbiters program
		type DelCollatorOrigin: EnsureOrigin<Self::Origin>;

		/// Maximum number of orbiters per collator
		type MaxPoolSize: Get<u32>;

		/// Maximum number of round to keep on storage
		type MaxRoundArchive: Get<Self::RoundIndex>;

		/// Round index type.
		type RoundIndex: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ sp_runtime::traits::MaybeDisplay
			+ sp_runtime::traits::AtLeast32Bit
			+ Copy;
	}

	#[pallet::storage]
	#[pallet::getter(fn collators_pool)]
	/// Current orbiters, with their "parent" collator
	pub type CollatorsPool<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, CollatorPoolInfo<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn account_lookup_override)]
	/// Account lookup override
	pub type AccountLookupOverride<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Option<T::AccountId>>;

	#[pallet::storage]
	/// Store active orbiter per round and per parent collator
	pub(crate) type OrbiterPerRound<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::RoundIndex,
		Blake2_128Concat,
		T::AccountId,
		T::AccountId,
		OptionQuery,
	>;

	#[pallet::storage]
	/// Current round index
	pub(crate) type CurrentRound<T: Config> = StorageValue<_, T::RoundIndex, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Prune old OrbiterPerRound entries
			let current_round = CurrentRound::<T>::get();
			if current_round > T::MaxRoundArchive::get() {
				let round_to_prune = current_round - T::MaxRoundArchive::get();
				OrbiterPerRound::<T>::remove_prefix(round_to_prune, None);
			}

			0
		}
	}

	/// An error that can occur while executing this pallet's extrinsics.
	#[pallet::error]
	pub enum Error<T> {
		/// The collator is already added in orbiters program.
		CollatorAlreadyAdded,
		/// This collator is not in orbiters program.
		CollatorNotFound,
		/// There are already too many orbiters associated with this collator.
		CollatorPoolTooLarge,
		/// This orbiter is already associated with this collator.
		OrbiterAlreadyInPool,
		/// This orbiter is not found
		OrbiterNotFound,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Paid the orbiter account the balance as liquid rewards.
		OrbiterRewarded {
			account: T::AccountId,
			rewards: BalanceOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a collator to orbiters program.
		#[pallet::weight(0)]
		pub fn add_collator(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::AddCollatorOrigin::ensure_origin(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			ensure!(
				CollatorsPool::<T>::get(&collator).is_none(),
				Error::<T>::CollatorAlreadyAdded
			);

			CollatorsPool::<T>::insert(collator, CollatorPoolInfo::default());

			Ok(())
		}
		/// Add an orbiter in a collator pool
		#[pallet::weight(0)]
		pub fn add_orbiter(
			origin: OriginFor<T>,
			orbiter: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let collator = ensure_signed(origin)?;
			let orbiter = T::Lookup::lookup(orbiter)?;

			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;
			let orbiters = collator_pool.get_orbiters();
			ensure!(
				(orbiters.len() as u32) < T::MaxPoolSize::get(),
				Error::<T>::CollatorPoolTooLarge
			);
			for orbiter_ in orbiters {
				if orbiter_ == &orbiter {
					return Err(Error::<T>::OrbiterAlreadyInPool.into());
				}
			}

			collator_pool.add_orbiter(orbiter);
			CollatorsPool::<T>::insert(collator, collator_pool);

			Ok(())
		}
		/// Remove a collator from orbiters program.
		#[pallet::weight(0)]
		pub fn remove_collator(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::DelCollatorOrigin::ensure_origin(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			let collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			// Remove all AccountLookupOverride entries related to this collator
			for orbiter in collator_pool.get_orbiters() {
				AccountLookupOverride::<T>::remove(&orbiter);
			}
			AccountLookupOverride::<T>::remove(&collator);

			// Remove the pool associated to this collator
			CollatorsPool::<T>::remove(collator);

			Ok(())
		}
		/// Add an orbiter in a collator pool
		#[pallet::weight(0)]
		pub fn remove_orbiter(
			origin: OriginFor<T>,
			orbiter: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let collator = ensure_signed(origin)?;
			let orbiter = T::Lookup::lookup(orbiter)?;

			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			if !collator_pool.remove_orbiter(&orbiter) {
				Err(Error::<T>::OrbiterNotFound.into())
			} else {
				Ok(())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Notify this pallet that a new round begin
		pub fn on_new_round() -> Weight {
			let current_round = CurrentRound::<T>::mutate(|current_round| {
				*current_round = current_round.saturating_add(One::one());
				*current_round
			});

			// Update current orbiter for each pool and edit AccountLookupOverride accordingly.
			CollatorsPool::<T>::translate::<CollatorPoolInfo<T::AccountId>, _>(
				|collator, mut pool| {
					// remove current orbiter, if any.
					if let Some(current_orbiter) = pool.get_current_orbiter() {
						AccountLookupOverride::<T>::remove(current_orbiter);
					}
					if let Some(next_orbiter) = pool.next_orbiter() {
						// Forbidding the collator to write blocks, it is now up to its orbiters to do it.
						AccountLookupOverride::<T>::insert(
							collator.clone(),
							Option::<T::AccountId>::None,
						);
						// Insert new current orbiter
						AccountLookupOverride::<T>::insert(
							next_orbiter.clone(),
							Some(collator.clone()),
						);
						OrbiterPerRound::<T>::insert(current_round, collator, next_orbiter);
					} else {
						// If there is no more active orbiter, you have to remove the collator override.
						AccountLookupOverride::<T>::remove(collator);
					}
					Some(pool)
				},
			);

			0
		}
		/// Notify this pallet that a collator received rewards
		pub fn distribute_rewards(
			pay_for_round: T::RoundIndex,
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> Weight {
			if let Some(orbiter) = OrbiterPerRound::<T>::take(pay_for_round, &collator) {
				if let Ok(amount_to_transfer) = T::Currency::withdraw(
					&collator,
					amount,
					frame_support::traits::WithdrawReasons::TRANSFER,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				) {
					let real_reward = amount_to_transfer.peek();
					if T::Currency::resolve_into_existing(&orbiter, amount_to_transfer).is_ok() {
						Self::deposit_event(Event::OrbiterRewarded {
							account: orbiter,
							rewards: real_reward,
						});
					}
				}
			}

			0
		}
	}
}

impl<T: Config> AccountLookup<T::AccountId> for Pallet<T> {
	fn lookup_account(nimbus_id: &NimbusId) -> Option<T::AccountId> {
		let account_id = T::AccountLookup::lookup_account(nimbus_id)?;
		match AccountLookupOverride::<T>::get(&account_id) {
			Some(override_) => override_,
			None => Some(account_id),
		}
	}
}
