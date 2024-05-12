// All pallets have to be like this because we're compiling for WebAssembly target
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use weights::*;

use sp_runtime::traits::{CheckedAdd, StaticLookup, Zero};

const LOG_TARGET: &str = "runtime::erc";

/// Utility type that defines RawOrigin conversion to reference accounts in transactions
pub(crate) type AccountIdLookupOf<T> =
	<<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::Codec;
	use core::fmt::Debug;
	use frame_support::{ensure, pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::{OriginFor, *};
	use scale_info::prelude::{string::String, vec::Vec};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Bounded, CheckedSub},
		FixedPointOperand, Saturating,
	};

	// I decided to make pallet instantiable so multiple instances of ERC20 can exist in one network
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_sudo::Config {
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		/// Defines total supply of a particular token. [`issue()`](Pallet::issue()) extrinsic does
		/// subtraction from it
		#[pallet::constant]
		type Supply: Get<u32>;

		/// Max possible length of a token name.
		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		/// Max possible length of a token symbol
		#[pallet::constant]
		type MaxSymbolLength: Get<u32>;

		/// The origin that's allowed to make privileged calls and, therefore, issue tokens from
		/// total supply. In real situation this will be either Root or Sudo call
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// This associated type is straight copied from [`Balances`](pallet_balances) to not deal
		/// with a lot of problems when you tightly couple it
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo
			+ FixedPointOperand;
	}

	/// Token name, encoded as bytes, UTF-8. Whatever utility is querying storage should do custom
	/// decoding since we can't stora actual strings in Substrate runtime
	#[pallet::storage]
	pub type Name<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<u8, T::MaxNameLength>, ValueQuery>;

	#[pallet::storage]
	pub type Symbol<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<u8, T::MaxSymbolLength>, ValueQuery>;

	// TODO: Maybe total supply should use associated type that differs from balance one?
	// It may create some problems of conversion but with proper `AtLeast` bounds it'll give pallet more robustness and flexibility
	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub type TotalSupply<T: Config<I>, I: 'static = ()> = StorageValue<_, T::Balance, ValueQuery>;

	/// A mapping of accounts to corresponding balances
	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub type Balances<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/*
		TODO: Maybe this can be simplified?
		I definitely can use structure like AccountId (one who gives permissions) -> (AccountId, Balance)[] or
		AccountId (one who receives permissions) -> AccountId (one who gave) -> Balance, both of which have more benefits
	*/
	/// Storage for allowances mechanism. Mapping is `AccountId -> AccountId -> Balance`. Where
	/// first account is who gives permissions to transfer one's own funds to another user, second
	/// is the one who can transfer.
	#[pallet::storage]
	#[pallet::getter(fn allowance_of)]
	pub type Allowances<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery,
	>;

	// Runtime events
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		Transfer { from: T::AccountId, to: T::AccountId, value: T::Balance },
		Approval { from: T::AccountId, to: T::AccountId, value: T::Balance },
		Issuance { to: T::AccountId, value: T::Balance },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		NoAllowance,
		SenderUnderflow,
		ReceiverOverflow,
		NotEnoughSupply,
		NotEnoughFunds,
		NotEnoughAllowance,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub balances: Vec<(T::AccountId, T::Balance)>,
		pub allowances: Vec<(T::AccountId, (T::AccountId, T::Balance))>,
		pub total_supply: T::Balance,
		pub name: String,
		pub symbol: String,
		// Eh... rust
		pub _ignore: PhantomData<I>,
	}

	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				balances: Default::default(),
				allowances: Default::default(),
				total_supply: T::Balance::max_value(),
				name: String::from("DEFAULT"),
				symbol: String::from("DEF"),
				_ignore: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {
			<Name<T, I>>::put(BoundedVec::truncate_from(self.name.encode()));
			<Symbol<T, I>>::put(BoundedVec::truncate_from(self.symbol.encode()));

			<TotalSupply<T, I>>::put(self.total_supply);

			for (a, b) in &self.balances {
				<Balances<T, I>>::insert(a, b);
			}

			for &(ref a, ref b) in self.allowances.iter() {
				<Allowances<T, I>>::insert(a, b.0.clone(), b.1);
			}
		}
	}

	// Functions that are callable
	#[pallet::call(weight(<T as Config<I>>::WeightInfo))]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: AccountIdLookupOf<T>,
			#[pallet::compact] value: T::Balance,
		) -> DispatchResult {
			// Since this transfer extrinsic is performed on behalf of whoever calls it we need to
			// check that it's signed by whoever called it
			let from = ensure_signed(origin)?;

			// Don't do nothing if value is zero.
			// TODO: Maybe return error or produce event instead?
			if value.is_zero() {
				return Ok(());
			}

			// Convert AccountId
			let dest = T::Lookup::lookup(dest)?;

			// Get the sender balance. Will return 0 by default (i.e when no value)
			let sender_funds = Self::balance_of(&from);

			// Check if sender has enough funds
			ensure!(sender_funds >= value, Error::<T, I>::NotEnoughFunds);

			// Get current funds amount of receiver
			let receiver_funds: T::Balance = Self::balance_of(&dest);

			// Try to update funds
			let remaining: T::Balance = sender_funds.saturating_sub(value);

			// Do a checked add to make sure we'll not overflow
			let new: T::Balance =
				receiver_funds.checked_add(&value).ok_or(Error::<T, I>::ReceiverOverflow)?;

			// Update sender
			Self::update_balance(&from, remaining);

			// Update receiver
			Self::update_balance(&dest, new);

			// Produce event if successful
			Self::deposit_event(Event::Transfer { from, to: dest, value });

			Ok(())
		}

		#[pallet::call_index(1)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: AccountIdLookupOf<T>,
			to: AccountIdLookupOf<T>,
			value: T::Balance,
		) -> DispatchResult {
			if value.is_zero() {
				return Ok(())
			}

			// Since we're doing transfer of tokens on behalf of other user we need to make sure
			// origin of transaction is signed
			let origin = ensure_signed(origin)?;

			let from = T::Lookup::lookup(from)?;
			let to = T::Lookup::lookup(to)?;

			// Update sender
			<Balances<T, I>>::mutate(&from, |balance| {
				// If there's no balance for such account there's no point in continuing
				ensure!(!balance.is_zero(), Error::<T, I>::NotEnoughFunds);

				ensure!(*balance >= value, Error::<T, I>::NotEnoughFunds);

				let remaining = balance.saturating_sub(value);
				*balance = remaining;

				Ok::<_, Error<T, I>>(())
			})?;

			// Update allowance
			<Allowances<T, I>>::mutate(&from, origin, |allowance| {
				let current = *allowance;

				// Make sure there's allowance for origin. Will fail if there's none
				ensure!(current > T::Balance::zero(), Error::<T, I>::NoAllowance);

				// Check if allowance is more or equal to the amount to be transferred and more than
				// zero
				ensure!(
					current >= value && current > T::Balance::from(0u32),
					Error::<T, I>::NotEnoughAllowance
				);

				*allowance = current - value;
				Ok::<_, Error<T, I>>(())
			})?;

			// Finally, update sender
			<Balances<T, I>>::mutate(&to, |balance| {
				*balance = balance.checked_add(&value).ok_or(Error::<T, I>::ReceiverOverflow)?;

				Ok::<_, Error<T, I>>(())
			})?;

			Self::deposit_event(Event::<T, I>::Transfer { from, to, value });

			Ok(())
		}

		#[pallet::call_index(2)]
		pub fn approve(
			origin: OriginFor<T>,
			who: AccountIdLookupOf<T>,
			value: T::Balance,
		) -> DispatchResult {
			if value.is_zero() {
				return Ok(())
			}

			// Because this method approves transfer of funds on behalf of other account we need to
			// make sure it's signed
			let from = ensure_signed(origin)?;

			let who = T::Lookup::lookup(who)?;

			Self::update_approve(&from, &who, value);

			Self::deposit_event(Event::<T, I>::Approval { from, to: who, value });

			Ok(())
		}

		#[pallet::call_index(9)]
		pub fn issue(
			origin: OriginFor<T>,
			to: AccountIdLookupOf<T>,
			#[pallet::compact] value: T::Balance,
		) -> DispatchResult {
			// Make sure only root can call this extrinsic or call dispatched by pallet_sudo
			T::ForceOrigin::ensure_origin(origin)?;

			// TODO: Should I move this before ensure_origin?
			// Don't do anything if value is zero
			if value.is_zero() {
				return Ok(())
			}

			let dest = T::Lookup::lookup(to)?;

			// Get current supply and make sure no overflow will occur
			let current_supply = Self::total_supply();
			let remaining =
				current_supply.checked_sub(&value).ok_or(Error::<T, I>::NotEnoughSupply)?;


			// Try to update balance
			<Balances<T, I>>::mutate(&dest, |balance| {
				let new_balance = balance.checked_add(&value).ok_or(Error::<T, I>::ReceiverOverflow)?;
				*balance = new_balance;

				Ok::<(), Error<T, I>>(())
			})?;

			// If balance got updated successfully we can update supply
			<TotalSupply<T, I>>::set(remaining);

			Self::deposit_event(Event::Issuance { to: dest, value });

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		// TODO: Maybe it's better to facilitate `frame_support::traits::tokens::fungible`?
		pub fn update_balance(account: &T::AccountId, value: T::Balance) {
			<Balances<T, I>>::set(account, value);
		}

		pub fn update_approve(from: &T::AccountId, who: &T::AccountId, value: T::Balance) {
			<Allowances<T, I>>::mutate(from, who, |allowance| {
				*allowance = allowance.saturating_add(value);
			});
		}
	}
}
