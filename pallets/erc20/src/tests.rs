use crate::{self as pallet_erc20, Config};
use frame_support::{derive_impl, parameter_types, traits::{ConstU16, ConstU64}};
use frame_support::traits::ConstU32;
use frame_system::{RawOrigin};
use sp_core::H256;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, BuildStorage, MultiSignature};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub const MAX_NAME_LENGTH: u8 = 50;
pub const MAX_SYMBOL_LENGTH: u8 = 50;

type Block = frame_system::mocking::MockBlock<Test>;

type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

pub(crate) type Balance = u128;

parameter_types! {
	pub const MaxNameLength: u8 = MAX_NAME_LENGTH;
	pub const MaxSymbolLength: u8 = MAX_SYMBOL_LENGTH;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
        Sudo: pallet_sudo,
        Balances: pallet_balances,
		Erc: pallet_erc20,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
}

#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Supply = ConstU32<u32::MAX>;
	type MaxNameLength = MaxNameLength;
	type MaxSymbolLength = MaxSymbolLength;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
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
				(0, 2000),
				(1, 0)
			],
            allowances: vec![],
            total_supply: u32::MAX,
            name: "Bitcoin".to_string(),
            symbol: "BTC".to_string(),
            _ignore: Default::default()
        }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

// TODO: This pallet is not yet ready
//
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
