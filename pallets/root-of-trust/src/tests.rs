use super::*;

use frame_support::{
    assert_noop, assert_ok, impl_outer_origin, parameter_types, traits::Imbalance, weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use std::cell::RefCell;

impl_outer_origin! {
    pub enum Origin for Test {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}
parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
}
impl pallet_balances::Trait for Test {
    type Balance = u64;
    type Event = ();
    type DustRemoval = ();
    type AccountStore = system::Module<Test>;
    type ExistentialDeposit = ();
}
parameter_types! {
    pub const MinimumApplicationAmount: u64 = 100;
    pub const MinimumCounterAmount: u64 = 1000;
    pub const MinimumChallengeAmount: u64 = 10000;
    pub const FinalizeApplicationPeriod: u64 = 100;
    pub const FinalizeChallengePeriod: u64 = 101; // Happens later to ease unit tests
    pub const LoosersSlash: Perbill = Perbill::from_percent(50);
}
impl pallet_tcr::Trait for Test {
    type Event = ();
    type Currency = pallet_balances::Module<Self>;
    type MinimumApplicationAmount = MinimumApplicationAmount;
    type MinimumCounterAmount = MinimumCounterAmount;
    type MinimumChallengeAmount = MinimumChallengeAmount;
    type FinalizeApplicationPeriod = FinalizeApplicationPeriod;
    type FinalizeChallengePeriod = FinalizeChallengePeriod;
    type LoosersSlash = LoosersSlash;
    type ChangeMembers = TestModule;
}
impl Trait for Test {
    type Event = ();
    type Currency = pallet_balances::Module<Self>;
    type CertificateId = <Test as system::Trait>::AccountId;
}

type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

type BalancesModule = pallet_balances::Module<Test>;
type TcrModule = pallet_tcr::Module<Test>;
type TestModule = Module<Test>;

const ROOT_MANAGER: u64 = 1;
const OFFCHAIN_CERTIFICATE_SIGNER: u64 = 2;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

fn allocate_balances() {
    let mut total_imbalance = <PositiveImbalanceOf<Test>>::zero();
    let r_manager =
        <Test as Trait>::Currency::deposit_creating(&ROOT_MANAGER, MinimumApplicationAmount::get());
    total_imbalance.subsume(r_manager);
}

fn do_register() {
    assert_ok!(TcrModule::apply(
        Origin::signed(ROOT_MANAGER),
        vec![],
        MinimumApplicationAmount::get(),
    ));
    <TcrModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeApplicationPeriod::get() + <system::Module<Test>>::block_number());
}

#[test]
fn tcr_membership_propagate() {
    new_test_ext().execute_with(|| {
        allocate_balances();
        do_register();

        assert_eq!(TestModule::is_member(&ROOT_MANAGER), true);
        assert_eq!(TestModule::is_member(&OFFCHAIN_CERTIFICATE_SIGNER), false);
    })
}

#[test]
fn non_member_can_not_buy_slots() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_noop!(
            TestModule::book_slot(Origin::signed(ROOT_MANAGER), OFFCHAIN_CERTIFICATE_SIGNER),
            Error::<Test>::NotAMember
        );
    })
}

#[test]
fn can_not_buy_slot_twice() {
    new_test_ext().execute_with(|| {
        allocate_balances();
        do_register();

        assert_ok!(TestModule::book_slot(
            Origin::signed(ROOT_MANAGER),
            OFFCHAIN_CERTIFICATE_SIGNER
        ));
        assert_noop!(
            TestModule::book_slot(Origin::signed(ROOT_MANAGER), OFFCHAIN_CERTIFICATE_SIGNER),
            Error::<Test>::SlotTaken
        );
    })
}

#[test]
fn member_can_buy_slots() {
    new_test_ext().execute_with(|| {
        allocate_balances();
        do_register();

        assert_ok!(TestModule::book_slot(
            Origin::signed(ROOT_MANAGER),
            OFFCHAIN_CERTIFICATE_SIGNER
        ));
        assert_eq!(
            TestModule::slots(OFFCHAIN_CERTIFICATE_SIGNER).key,
            OFFCHAIN_CERTIFICATE_SIGNER
        );
        assert_eq!(
            TestModule::slots(OFFCHAIN_CERTIFICATE_SIGNER).owner,
            ROOT_MANAGER
        );
        assert_eq!(
            TestModule::slots(OFFCHAIN_CERTIFICATE_SIGNER).created,
            <system::Module<Test>>::block_number()
        );
        assert_eq!(
            TestModule::slots(OFFCHAIN_CERTIFICATE_SIGNER).renewed,
            <system::Module<Test>>::block_number()
        );
    })
}
