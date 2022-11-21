#![cfg(test)]

use crate as lottery;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, Everything, OnFinalize, OnInitialize},
	PalletId,
};
use frame_support_test::TestRandomness;
use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	Config,
};
pub use pallet_balances::Call as BalancesCall;
use sp_runtime::generic::Header;
use sp_runtime::testing::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

construct_runtime!(
 pub enum Test where
  Block = Block,
  NodeBlock = Block,
  UncheckedExtrinsic = UncheckedExtrinsic,
 {
   System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
   Lottery: lottery::{Pallet, Call, Storage, Event<T>},
	 Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
 }
);

parameter_types! {
  pub const RoulettePalletId: PalletId = PalletId(*b"roulette");
}

impl lottery::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type LotteryRandomness = TestRandomness<Self>;
	type Currency = Balances;
	type PalletId = RoulettePalletId;
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u64 = 50;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}

impl Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header<Self::BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU32<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_to_block(n: u64) {
	while u64::from(System::block_number()) < n {
		if System::block_number() > 1 {
			Lottery::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Lottery::on_initialize(System::block_number());
	}
}
