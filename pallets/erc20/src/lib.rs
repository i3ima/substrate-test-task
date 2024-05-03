// All pallets have to be like this because we're compiling for WebAssembly target
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::{OriginFor, *};
	use scale_info::prelude::{string::String, vec::Vec};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_sudo::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type Supply: Get<u32>;

		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		#[pallet::constant]
		type MaxSymbolLength: Get<u32>;

		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		// TODO: Weights for this pallet
		// type WeightInfo;
	}

	// Since most tools like polkadot.js automatically converts Vec<u8> to string we can store them
	// like that
	#[pallet::storage]
	pub type Name<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxNameLength>, ValueQuery>;

	#[pallet::storage]
	pub type Symbol<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxSymbolLength>>;

	#[pallet::storage]
	pub type TotalSupply<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type Balances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance>;

	#[pallet::storage]
	pub type Allowances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		OptionQuery,
	>;

	// Runtime events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Transfer { from: T::AccountId, to: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub balances: Vec<(T::AccountId, T::Balance)>,
		pub allowances: Vec<(T::AccountId, (T::AccountId, T::Balance))>,
		pub total_supply: u32,
		pub name: String,
		pub symbol: String,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				balances: Default::default(),
				allowances: Default::default(),
				total_supply: u32::MAX,
				name: String::from("DEFAULT"),
				symbol: String::from("DEF"),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			<Name<T>>::put(BoundedVec::truncate_from(self.name.encode()));
			<Symbol<T>>::put(BoundedVec::truncate_from(self.symbol.encode()));

			<TotalSupply<T>>::put(self.total_supply);

			for (a, b) in &self.balances {
				<Balances<T>>::insert(a, b);
			}

			for &(ref a, ref b) in self.allowances.iter() {
				<Allowances<T>>::insert(a, b.0.clone(), b.1);
			}
		}
	}

	// Functions that are callable
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(2)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::deposit_event(Event::Transfer { from: who, to });

			Ok(())
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(9)]
		pub fn issue(origin: OriginFor<T>, to: T::AccountId, value: T::Balance) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			Ok(())
		}
	}
}

pub mod weights {
	// TODO: Calculate/benchmark actual weights
	pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
}
