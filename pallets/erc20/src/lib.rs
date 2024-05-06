// All pallets have to be like this because we're compiling for WebAssembly target
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use weights::*;

use sp_runtime::traits::{CheckedAdd, SaturatedConversion, StaticLookup, Zero};

const LOG_TARGET: &str = "runtime::erc";

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{ensure, pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::{OriginFor, *};
	use scale_info::prelude::{string::String, vec::Vec};
	use sp_runtime::Saturating;

	// I decided to make pallet instantiable so multiple instances of ERC20 can exist in one network
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_balances::Config + pallet_sudo::Config
	{
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type Supply: Get<u32>;

		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		#[pallet::constant]
		type MaxSymbolLength: Get<u32>;

		/// The origin that's allowed to make privileged calls and issue tokens from total supply
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		// TODO: Weights for this pallet
		// type WeightInfo;
	}

	#[pallet::storage]
	pub type Name<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<u8, T::MaxNameLength>, ValueQuery>;

	#[pallet::storage]
	pub type Symbol<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<u8, T::MaxSymbolLength>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub type TotalSupply<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub type Balances<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance>;

	#[pallet::storage]
	#[pallet::getter(fn allowance_of)]
	pub type Allowances<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
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
		NoSenderAccount,
		NoReceiverAccount,
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
		pub total_supply: u32,
		pub name: String,
		pub symbol: String,
		pub _ignore: PhantomData<I>,
	}

	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				balances: Default::default(),
				allowances: Default::default(),
				total_supply: u32::MAX,
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
	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: AccountIdLookupOf<T>,
			#[pallet::compact] value: T::Balance,
		) -> DispatchResult {
			// Don't do nothing if value is zero.
			// TODO: Maybe return error or produce event instead?
			if value.is_zero() {
				return Ok(());
			}

			// Since this transfer extrinsic is performed on behalf of whoever calls it we need to
			// check that it's signed by whoever called it
			let from = ensure_signed(origin)?;

			// Convert AccountId
			let dest = T::Lookup::lookup(dest)?;

			// Make sure that sender account exists in storage
			Self::ensure_balance_mapping(&from);

			// We can safely do unwrap because of previous check
			let sender_funds = Self::balance_of(&from).unwrap();

			// Check if sender has enough funds
			ensure!(sender_funds >= value, Error::<T, I>::NotEnoughFunds);

			// If there's no sender account in balances -- insert it first
			Self::ensure_balance_mapping(&dest);

			// Get current funds amount of receiver
			let receiver_funds: T::Balance = Self::balance_of(&dest).unwrap();

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

		#[pallet::weight(Weight::default())]
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
			// We can safely do such conversion because Balance has AtLeast32BitUnsigned trait bound
			let value: u32 = value.saturated_into();

			let origin = ensure_signed(origin)?;
			let from = T::Lookup::lookup(from)?;
			let to = T::Lookup::lookup(to)?;

			// Make sure there's allowance for origin. Will fail if there's none

			let current_allowance: u32 = Self::allowance_of(&from, &origin)
				.map(|b| b.saturated_into())
				.ok_or(Error::<T, I>::NoAllowance)?;

			log::debug!(target: LOG_TARGET, "Current allowance {:?}", &current_allowance);

			// Update sender
			let remaining_allowance = <Balances<T, I>>::mutate(&from, |balance| {
				// If there's no balance for such account there's no point in continuing
				let current_balance: u32 =
					balance.ok_or(Error::<T, I>::NotEnoughFunds).map(|b| b.saturated_into())?;

				ensure!(
					current_allowance >= value && current_allowance > 0,
					Error::<T, I>::NotEnoughAllowance
				);

				ensure!(current_balance >= value.saturated_into(), Error::<T, I>::NotEnoughFunds);

				let remaining = current_balance.saturating_sub(value);

				*balance = Some(Self::u32_to_balance(remaining));

				Ok::<_, Error<T, I>>(current_allowance - value)
			})?;

			// Update allowance
			<Allowances<T, I>>::mutate(&from, origin, |allowance| {
				*allowance = Some(Self::u32_to_balance(remaining_allowance));
				Ok::<_, Error<T, I>>(())
			})?;

			Self::ensure_balance_mapping(&to);
			// Finally, update sender
			<Balances<T, I>>::mutate(&to, |balance| {
				*balance = Some(
					balance
						.unwrap()
						.checked_add(&Self::u32_to_balance(value))
						.ok_or(Error::<T, I>::ReceiverOverflow)?,
				);

				Ok::<(), Error<T, I>>(())
			})?;

			Self::deposit_event(Event::<T, I>::Transfer {
				from,
				to,
				value: Self::u32_to_balance(value),
			});

			Ok(())
		}

		#[pallet::weight(Weight::default())]
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


			match Self::allowance_of(&from, &who) {
				Some(_) => {
					// Otherwise perform mutate with saturation add which protects us from overflow
					// of allowed funds
					<Allowances<T, I>>::mutate(&from, &who, |allowance| {
						*allowance = Some(allowance.unwrap().saturating_add(value));
					});
				},
				None => {
					// If there's yet no mapping of AccountId -> AccountId -> Balance just insert
					<Allowances<T, I>>::insert(&from, &who, value);
				},
			}

			Self::deposit_event(Event::<T, I>::Approval { from, to: who, value });

			Ok(())
		}

		#[pallet::weight(Weight::default())]
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
			let remaining = current_supply
				.checked_sub(value.saturated_into())
				.ok_or(Error::<T, I>::NotEnoughSupply)?;

			Self::ensure_balance_mapping(&dest);

			// Try to update balance
			<Balances<T, I>>::mutate(&dest, |balance| {
				let new_balance =
					balance.unwrap().checked_add(&value).ok_or(Error::<T, I>::ReceiverOverflow)?;
				*balance = Some(new_balance);

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
			<Balances<T, I>>::set(account, Some(value));
		}

		/// Utility function that checks account presence in Balances storage and inserts if needed
		pub fn ensure_balance_mapping(account: &T::AccountId) {
			Self::balance_of(&account).is_none().then(|| {
				<Balances<T, I>>::insert(account, Self::u32_to_balance(0));
			});
		}

		pub fn u32_to_balance(value: u32) -> T::Balance {
			Into::<T::Balance>::into(value)
		}
	}
}
