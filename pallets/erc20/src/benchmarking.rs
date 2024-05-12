#![cfg(feature = "runtime-benchmarks")]
use super::*;

use super::Pallet as Erc;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

const SEED: u32 = 0;

#[instance_benchmarks]
mod benchmarks {
	use super::*;
	use sp_runtime::{
		traits::{Bounded, StaticLookup},
		Saturating,
	};

	#[benchmark]
	fn transfer() {
		// Benchmarks of Substrate pallets typically have 3 distinct stages: setup, call and
		// verification Setup is all that happens before #[extrinsic_call] and verification it's
		// what goes after

		// Setup
		let caller: T::AccountId = whitelisted_caller();

		// How much to transfer from it
		let transfer_amount = T::Balance::from(500u32);

		let recipient: T::AccountId = account("recipient", 1, SEED);
		let recipient_lookup = T::Lookup::unlookup(recipient.clone());

		//Call
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount);

		// Verification
		assert_eq!(Erc::<T, I>::balance_of(caller), T::Balance::from(1500u32));
		assert_eq!(Erc::<T, I>::balance_of(recipient), transfer_amount);
	}

	#[benchmark]
	fn issue() {
		// How much to issue
		let issue_amount = T::Balance::from(500u32);

		// And who will receive it
		let recipient: T::AccountId = account("recipient", 1, SEED);
		let recipient_lookup = T::Lookup::unlookup(recipient.clone());

		#[extrinsic_call]
		_(RawOrigin::Root, recipient_lookup, issue_amount);

		// Verify it
		assert_eq!(
			Erc::<T, I>::total_supply(),
			<T as Config<I>>::Balance::max_value() - issue_amount
		);
		assert_eq!(Erc::<T, I>::balance_of(recipient), issue_amount);
	}

	#[benchmark]
	fn transfer_from() {
		// Account that allows transfer of funds
		let owner: T::AccountId = whitelisted_caller();
		let owner_lookup = T::Lookup::unlookup(owner.clone());
		let original_owner_balance = Erc::<T, I>::balance_of(&owner);

		// How much will be allowed
		let allowance_amount = T::Balance::from(500u32);

		// This account will be allowed to transfer funds
		let allowed: T::AccountId = account("allowed", 1, SEED);

		// To this account
		let recipient: T::AccountId = account("recipient", 2, SEED);
		let recipient_lookup = T::Lookup::unlookup(recipient.clone());

		Erc::<T, I>::update_approve(&owner, &allowed, allowance_amount);

		#[extrinsic_call]
		_(RawOrigin::Signed(allowed.clone()), owner_lookup, recipient_lookup, allowance_amount);

		// Check that allowance is now zero
		assert_eq!(Erc::<T, I>::allowance_of(&owner, allowed), T::Balance::zero());
		// Check that owner balance is now less
		assert_eq!(
			Erc::<T, I>::balance_of(owner),
			original_owner_balance.saturating_sub(allowance_amount)
		);
		// Check that recipient received funds
		assert_eq!(Erc::<T, I>::balance_of(recipient), allowance_amount);
	}

	#[benchmark]
	fn approve() {
		let owner: T::AccountId = whitelisted_caller();

		let allowance_amount = T::Balance::from(500u32);

		let recipient: T::AccountId = account("recipient", 2, SEED);
		let recipient_lookup = T::Lookup::unlookup(recipient.clone());

		#[extrinsic_call]
		_(RawOrigin::Signed(owner.clone()), recipient_lookup, allowance_amount);

		assert_eq!(Erc::<T, I>::allowance_of(owner, recipient), allowance_amount);
	}

	impl_benchmark_test_suite! {
		Erc,
		tests::ExtBuilder::default().build(),
		tests::Test,
	}
}
