use frame_benchmarking::whitelisted_caller;
use crate::{self as pallet_erc20, Config};
use frame_support::{derive_impl, parameter_types};
use frame_support::traits::ConstU32;
use sp_runtime::{BuildStorage};
// Const that are needed for this pallet Config
pub const MAX_NAME_LENGTH: u8 = 50;
pub const MAX_SYMBOL_LENGTH: u8 = 50;
pub const TOTAL_SUPPLY: u32 = u32::MAX;


// Types that are needed for Config's of this pallet and other that are coupled
type Block = frame_system::mocking::MockBlock<Test>;
pub type Balance = u32;


parameter_types! {
	pub const MaxNameLength: u8 = MAX_NAME_LENGTH;
	pub const MaxSymbolLength: u8 = MAX_SYMBOL_LENGTH;
	pub static ExistentialDeposit: u64 = 1;
}


// Configure a mock runtime to test.rs the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
        Sudo: pallet_sudo,
		Erc: pallet_erc20,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}



#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Supply = ConstU32<TOTAL_SUPPLY>;
	type MaxNameLength = MaxNameLength;
	type MaxSymbolLength = MaxSymbolLength;
	type ForceOrigin = frame_system::EnsureRoot<<Test as frame_system::Config>::AccountId>;
	
	type Balance = Balance;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

#[derive(Clone)]
pub struct ExtBuilder {}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_erc20::GenesisConfig::<Test> {
            balances: vec![
				(whitelisted_caller(), 2000),
				(1, 0)
			],
            allowances: vec![],
            total_supply: Balance::MAX,
            name: "Ethereum".to_string(),
            symbol: "ETH".to_string(),
            _ignore: Default::default()
        }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {}
	}
}

// TODO: Add tests that will be checking for possibly failing scenarios
// #[test]
// fn transfer_works_for_basic_case() {
// 	new_test_ext().execute_with(|| {
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);
//
// 		// Provide some basic storage
//
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(Erc::transfer(RuntimeOrigin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(Balances::<Test>::get(), Some(42));
// 		// Assert that the correct event was deposited
// 		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
// 	});
// }
//
// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(RuntimeOrigin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
