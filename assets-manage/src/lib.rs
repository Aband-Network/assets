#![cfg_attr(not(feature = "std"), no_std)]

pub mod asset_id;
pub mod traits;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use traits::GetMultiAssetInfo;
use codec::{Encode, Decode, MaxEncodedLen, Codec};
use sp_runtime::{RuntimeDebug, traits::{Zero, AccountIdConversion}};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use orml_traits::{
	arithmetic::{Signed, SimpleArithmetic},
	currency::TransferAll,
	BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency, BasicReservableCurrency,
	LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
	NamedBasicReservableCurrency, NamedMultiReservableCurrency,
};
use frame_support::{
	BoundedVec,
	dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AssetDetails<AccountId, Metadata> {
	pub creator: Option<AccountId>,
	pub owner: Option<AccountId>,
	pub metadata: Metadata,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AssetMetadata<AccountId, Balance, String> {
	pub name: String,
	pub token_symbol: String,
	pub decimal: u16,
	pub deposit: Balance,
	pub token_account_id: AccountId,
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type StringOf<T> = BoundedVec<u8, <T as Config>::MaxStringLen>;
	pub(crate) type ReserveIdentifierOf<T> = <<T as Config>::MultiCurrency as NamedMultiReservableCurrency<
		<T as frame_system::Config>::AccountId,
	>>::ReserveIdentifier;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MultiCurrency: TransferAll<Self::AccountId>
			+ MultiCurrencyExtended<Self::AccountId>
			+ MultiLockableCurrency<Self::AccountId>
			+ MultiReservableCurrency<Self::AccountId>
			+ NamedMultiReservableCurrency<Self::AccountId>;
		type CurrencyIdConvertToAccountId: From<CurrencyIdOf<Self>> + AccountIdConversion<Self::AccountId>;
		#[pallet::constant]
		type CreateReserveBalance: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type MaxStringLen: Get<u32>;
		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;
		// #[pallet::constant]
		type CreateAssetReserveIdentifier: Get<ReserveIdentifierOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn assets_metadata)]
	pub type AssetsDetails<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, AssetDetails<T::AccountId, AssetMetadata<T::AccountId, BalanceOf<T>, StringOf<T>>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn create_asset_reserve_of)]
	pub type CreateAssetReserveOf<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		CreateAsset{
			creator: Option<T::AccountId>,
			asset_id: CurrencyIdOf<T>,
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		BalanceNotZero,
		AssetAlreadyCreated,
		NativeAsset,
		MaxStringExceeded,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn create_asset(origin: OriginFor<T>, asset_id: CurrencyIdOf<T>, owner: Option<T::AccountId>, name: Vec<u8>, token_symbol: Vec<u8>, decimal: u16, total_issuance: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			T::MultiCurrency::reserve_named(&T::CreateAssetReserveIdentifier::get(), T::GetNativeCurrencyId::get(), &creator.clone(), T::CreateReserveBalance::get())?;
			CreateAssetReserveOf::<T>::insert(asset_id, T::CreateReserveBalance::get());
			Self::do_create(asset_id, Some(creator.clone()), owner.clone(), name.clone(), token_symbol.clone(), decimal, total_issuance)?;
			Ok(().into())
		}

	}

	impl<T: Config> Pallet<T> {
		pub fn do_create(asset_id: CurrencyIdOf<T>, creator: Option<T::AccountId>, owner: Option<T::AccountId>, name: Vec<u8>, token_symbol: Vec<u8>, decimal: u16, total_issuance: BalanceOf<T>) -> DispatchResult{
			ensure!(T::MultiCurrency::minimum_balance(asset_id).is_zero(), Error::<T>::BalanceNotZero);
			ensure!(asset_id != T::GetNativeCurrencyId::get(), Error::<T>::NativeAsset);
			ensure!(!AssetsDetails::<T>::contains_key(asset_id), Error::<T>::AssetAlreadyCreated);
			if let Some(c) = creator.clone() {
				T::MultiCurrency::deposit(asset_id, &c, total_issuance)?;
			}
			let token_account_id = T::CurrencyIdConvertToAccountId::from(asset_id).into_account_truncating();

			let bounded_name: BoundedVec<u8, T::MaxStringLen> =
				name.try_into().map_err(|_| Error::<T>::MaxStringExceeded)?;
			let bounded_token_symbol: BoundedVec<u8, T::MaxStringLen> =
				token_symbol.try_into().map_err(|_| Error::<T>::MaxStringExceeded)?;

			AssetsDetails::<T>::insert(asset_id, AssetDetails {
				creator: creator.clone(),
				owner,
				metadata: AssetMetadata {
					name: bounded_name,
					token_symbol: bounded_token_symbol,
					decimal,
					deposit: total_issuance,
					token_account_id,
				},
			});
			Self::deposit_event(Event::CreateAsset{
				creator,
				asset_id,
			});
			Ok(())
		}

	}

	impl <T: Config>GetMultiAssetInfo<CurrencyIdOf<T>, AssetMetadata<T::AccountId, BalanceOf<T>, StringOf<T>>, AssetDetails<T::AccountId, AssetMetadata<T::AccountId, BalanceOf<T>, StringOf<T>>>> for Pallet<T> {
		fn get_asset_details(asset_id: CurrencyIdOf<T>) -> Option<AssetDetails<T::AccountId, AssetMetadata<T::AccountId, BalanceOf<T>, StringOf<T>>>> {
			AssetsDetails::<T>::get(asset_id)
		}
	}
}
