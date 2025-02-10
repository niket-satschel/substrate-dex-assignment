//! A custom Substrate pallet that supports liquidity deposits and withdrawals
//! It also integrates the `pallet-assets` to manage two custom tokens

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_asset_conversion;
use pallet_assets::{self as assets};
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedSub, StaticLookup};
use sp_std::vec::Vec;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{traits::tokens::Balance, PalletId};
	use sp_runtime::traits::Saturating;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LiquidityDeposited {
			who: T::AccountId, // Ensure this matches function usage
			asset_1: T::AssetIdParameter,
			amount_1: T::Balance,
			asset_2: T::AssetIdParameter,
			amount_2: T::Balance,
		},
		LiquidityWithdrawn {
			// ✅ Add this event
			who: T::AccountId,
			asset_1: T::AssetIdParameter,
			amount_1: T::Balance,
			asset_2: T::AssetIdParameter,
			amount_2: T::Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidDepositRatio,
		NoLiquidity,
		PoolNotFound,
		InsufficientLiquidity,
	}

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPool<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (T::Balance, T::Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_reserves)]
	pub type PoolReserves<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetIdParameter, // First asset ID
		Blake2_128Concat,
		T::AssetIdParameter,                  // Second asset ID
		(T::Balance, T::Balance, T::Balance), // (TOKEN_A reserve, TOKEN_B reserve, k)
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn mint_tokens(
			origin: OriginFor<T>,
			asset_id: T::AssetIdParameter,
			amount: T::Balance,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			pallet_assets::Pallet::<T>::mint(
				frame_system::RawOrigin::Signed(who.clone()).into(),
				asset_id,
				beneficiary,
				amount,
			)?;
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_tokens(
			origin: OriginFor<T>,
			asset_id: T::AssetIdParameter,
			to: AccountIdLookupOf<T>,
			amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(who.clone()).into(),
				asset_id,
				to,
				amount,
			)?;
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn deposit_liquidity(
			origin: OriginFor<T>,
			asset_1: T::AssetIdParameter,
			asset_2: T::AssetIdParameter,
			amount_1: T::Balance,
			amount_2: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// let pool_account = Self::account_id();
			// let asset_1: T::AssetId = asset_1.clone().into();
			// let asset_2: T::AssetId = asset_2.clone().into();

			// Transfer tokens to pool account
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(who.clone()).into(),
				asset_1.clone(),
				T::Lookup::unlookup(Self::account_id()),
				amount_1,
			)?;
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(who.clone()).into(),
				asset_2.clone(),
				T::Lookup::unlookup(Self::account_id()),
				amount_2,
			)?;

			// Check if pool exists
			let pool_exists = PoolReserves::<T>::contains_key(asset_1.clone(), asset_2.clone());

			if !pool_exists {
				// First-time deposit: Initialize the pool with k = x * y
				let k = amount_1.saturating_mul(amount_2);
				PoolReserves::<T>::insert(
					asset_1.clone(),
					asset_2.clone(),
					(amount_1, amount_2, k),
				);
			} else {
				// Pool already exists: Check deposit ratio
				let (reserve_1, reserve_2, k) =
					PoolReserves::<T>::get(asset_1.clone(), asset_2.clone()).unwrap_or_default();

				let expected_amount_2 = reserve_2.saturating_mul(amount_1) / reserve_1;
				ensure!(amount_2 >= expected_amount_2, Error::<T>::InvalidDepositRatio);

				// Update reserves
				let new_reserve_1 = reserve_1.saturating_add(amount_1);
				let new_reserve_2 = reserve_2.saturating_add(amount_2);
				let new_k = new_reserve_1.saturating_mul(new_reserve_2); // Recalculate k

				PoolReserves::<T>::insert(
					asset_1.clone(),
					asset_2.clone(),
					(new_reserve_1, new_reserve_2, new_k),
				);
			}

			// Store user's liquidity contribution
			LiquidityPool::<T>::mutate(&who, |balance| {
				balance.0 = balance.0.saturating_add(amount_1);
				balance.1 = balance.1.saturating_add(amount_2);
			});

			// Emit event
			Self::deposit_event(Event::LiquidityDeposited {
				who: who.clone(),
				asset_1,
				amount_1,
				asset_2,
				amount_2,
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn withdraw_liquidity(
			origin: OriginFor<T>,
			asset_1: T::AssetIdParameter,
			asset_2: T::AssetIdParameter,
			withdraw_amount_1: T::Balance,
			withdraw_amount_2: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pool_account = Self::account_id();

			// Ensure user has liquidity
			ensure!(LiquidityPool::<T>::contains_key(&who), Error::<T>::NoLiquidity);

			// Get user liquidity balance
			let (user_liquidity_1, user_liquidity_2) = LiquidityPool::<T>::get(&who);

			// Ensure the requested amounts are within user's available liquidity
			ensure!(withdraw_amount_1 <= user_liquidity_1, Error::<T>::InsufficientLiquidity);
			ensure!(withdraw_amount_2 <= user_liquidity_2, Error::<T>::InsufficientLiquidity);

			// Get current pool reserves
			ensure!(
				PoolReserves::<T>::contains_key(asset_1.clone(), asset_2.clone()),
				Error::<T>::PoolNotFound
			);
			let (reserve_1, reserve_2, _) =
				PoolReserves::<T>::get(asset_1.clone(), asset_2.clone()).unwrap_or_default();

			// Ensure pool has enough liquidity to fulfill the request
			ensure!(withdraw_amount_1 <= reserve_1, Error::<T>::InsufficientLiquidity);
			ensure!(withdraw_amount_2 <= reserve_2, Error::<T>::InsufficientLiquidity);

			// Update user's liquidity balance
			LiquidityPool::<T>::mutate(&who, |balance| {
				balance.0 = balance.0.saturating_sub(withdraw_amount_1);
				balance.1 = balance.1.saturating_sub(withdraw_amount_2);
			});

			// Update pool reserves
			let new_reserve_1 = reserve_1.saturating_sub(withdraw_amount_1);
			let new_reserve_2 = reserve_2.saturating_sub(withdraw_amount_2);
			let new_k = new_reserve_1.saturating_mul(new_reserve_2); // Recalculate k

			PoolReserves::<T>::insert(
				asset_1.clone(),
				asset_2.clone(),
				(new_reserve_1, new_reserve_2, new_k),
			);

			// Transfer assets back to user
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(pool_account.clone()).into(),
				asset_1.clone(),
				T::Lookup::unlookup(who.clone()),
				withdraw_amount_1,
			)?;
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(pool_account.clone()).into(),
				asset_2.clone(),
				T::Lookup::unlookup(who.clone()),
				withdraw_amount_2,
			)?;

			// Emit event
			Self::deposit_event(Event::LiquidityWithdrawn {
				who: who.clone(),
				asset_1,
				amount_1: withdraw_amount_1,
				asset_2,
				amount_2: withdraw_amount_2,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
