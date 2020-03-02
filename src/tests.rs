use super::*;

use frame_support::{
    assert_noop, assert_ok, impl_outer_origin, parameter_types, traits::Imbalance, weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
    KeyTypeId, Perbill,
};

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
}
impl Trait for Test {
    type Event = ();
    type Currency = pallet_balances::Module<Self>;
    type MinimumApplicationAmount = MinimumApplicationAmount;
}

type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

const CANDIDATE: u64 = 1;

type BalancesModule = pallet_balances::Module<Test>;
type TestModule = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

#[test]
fn apply_works() {
    new_test_ext().execute_with(|| {
        let mut total_imbalance = <PositiveImbalanceOf<Test>>::zero();
        let r = <Test as Trait>::Currency::deposit_creating(
            &CANDIDATE,
            MinimumApplicationAmount::get(),
        );
        total_imbalance.subsume(r);

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get()
        ));
        assert_eq!(
            TestModule::applications(CANDIDATE).candidate_deposit,
            MinimumApplicationAmount::get()
        );
    })
}

#[test]
fn can_not_apply_twice() {
    new_test_ext().execute_with(|| {
        let mut total_imbalance = <PositiveImbalanceOf<Test>>::zero();
        let r = <Test as Trait>::Currency::deposit_creating(
            &CANDIDATE,
            MinimumApplicationAmount::get(),
        );
        total_imbalance.subsume(r);

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get()
        ));
        assert_noop!(
            TestModule::apply(
                Origin::signed(CANDIDATE),
                vec![],
                MinimumApplicationAmount::get()
            ),
            Error::<Test>::ApplicationPending
        );
    })
}

#[test]
fn can_not_apply_if_not_enough_tokens() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TestModule::apply(
                Origin::signed(CANDIDATE),
                vec![],
                MinimumApplicationAmount::get()
            ),
            Error::<Test>::NotEnoughFunds
        );
    })
}

#[test]
fn can_not_apply_if_deposit_is_too_low() {
    new_test_ext().execute_with(|| {
        let mut total_imbalance = <PositiveImbalanceOf<Test>>::zero();
        let r = <Test as Trait>::Currency::deposit_creating(
            &CANDIDATE,
            MinimumApplicationAmount::get(),
        );
        total_imbalance.subsume(r);

        assert_noop!(
            TestModule::apply(
                Origin::signed(CANDIDATE),
                vec![],
                MinimumApplicationAmount::get() - 1
            ),
            Error::<Test>::DepositTooSmall
        );
    })
}
