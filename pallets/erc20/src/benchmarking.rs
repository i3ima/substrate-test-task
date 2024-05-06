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
        let source: T::AccountId = account("source", 0, SEED);
        let source_lookup = T::Lookup::unlookup(source.clone());

        // How much to transfer from it
        let transfer_amount = Erc::u32_to_balance(500);

        let recipient: T::AccountId = account("recipient", 1, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());

        // Give source some balance
        // let _ = <Erc<T, I>>::transfer(&source_lookup, &recipient, transfer_amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(&source_lookup), recipient_lookup, transfer_amount);

        // Check if successfully transferred
        assert_eq!(Erc::<T, I>::balance_of(recipient), Erc::u32_to_balance(500));
    }

    impl_benchmark_test_suite! {
		Erc,
		crate::tests::ExtBuilder::default().build(),
		tests::Test,
	}
}
