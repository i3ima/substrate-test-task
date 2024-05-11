#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Erc;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

const SEED: u32 = 0;

#[instance_benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn transfer()  {
        let caller: T::AccountId = whitelisted_caller();

        // How much to transfer from it
        let transfer_amount = <T as Config<I>>::Balance::from(500u32);

        let recipient: T::AccountId = account("recipient", 1, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());

        // Give source some balance
        // let _ = <Erc<T, I>>::transfer(&source_lookup, &recipient, transfer_amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount);

        // Check if successfully transferred
        assert_eq!(Erc::<T, I>::balance_of(caller), <T as Config<I>>::Balance::from(1500u32));
        assert_eq!(Erc::<T, I>::balance_of(recipient), transfer_amount);
    }

    impl_benchmark_test_suite! {
		Erc,
		tests::ExtBuilder::default().build(),
		tests::Test,
	}
}
