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
    pub const MinimumChallengeAmount: u64 = 1000;
    pub const FinalizeApplicationPeriod: u64 = 100;
    pub const FinalizeChallengePeriod: u64 = 101; // Happens later to ease unit tests
    pub const LoosersSlash: Perbill = Perbill::from_percent(50);
}
impl Trait for Test {
    type Event = ();
    type Currency = pallet_balances::Module<Self>;
    type MinimumApplicationAmount = MinimumApplicationAmount;
    type MinimumChallengeAmount = MinimumChallengeAmount;
    type FinalizeApplicationPeriod = FinalizeApplicationPeriod;
    type FinalizeChallengePeriod = FinalizeChallengePeriod;
    type LoosersSlash = LoosersSlash;
}

type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

const CANDIDATE: u64 = 1;
const CHALLENGER: u64 = 2;
const VOTER_FOR: u64 = 3;
const VOTER_AGAINST: u64 = 4;

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

fn allocate_balances() {
    let mut total_imbalance = <PositiveImbalanceOf<Test>>::zero();
    let r_candidate =
        <Test as Trait>::Currency::deposit_creating(&CANDIDATE, MinimumApplicationAmount::get());
    let r_challenger =
        <Test as Trait>::Currency::deposit_creating(&CHALLENGER, MinimumChallengeAmount::get());
    let r_voter_for = <Test as Trait>::Currency::deposit_creating(&VOTER_FOR, 1000);
    let r_voter_against = <Test as Trait>::Currency::deposit_creating(&VOTER_AGAINST, 1000);
    total_imbalance.subsume(r_candidate);
    total_imbalance.subsume(r_challenger);
    total_imbalance.subsume(r_voter_for);
    total_imbalance.subsume(r_voter_against);
}

#[test]
fn lock_unlock_works() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_eq!(
            BalancesModule::usable_balance(CANDIDATE),
            MinimumApplicationAmount::get()
        );

        assert_ok!(TestModule::lock_for(
            CANDIDATE,
            MinimumApplicationAmount::get() / 2
        ));
        assert_eq!(
            BalancesModule::usable_balance(CANDIDATE),
            MinimumApplicationAmount::get() / 2
        );
        assert_eq!(
            <AmountLocked<Test>>::get(CANDIDATE),
            MinimumApplicationAmount::get() / 2
        );
        assert_ok!(TestModule::lock_for(
            CANDIDATE,
            MinimumApplicationAmount::get() / 2
        ));
        assert_eq!(BalancesModule::usable_balance(CANDIDATE), 0);
        assert_eq!(
            <AmountLocked<Test>>::get(CANDIDATE),
            MinimumApplicationAmount::get()
        );
        assert_noop!(
            TestModule::lock_for(CANDIDATE, 1),
            Error::<Test>::NotEnoughFunds
        );

        assert_ok!(TestModule::unlock_for(
            CANDIDATE,
            MinimumApplicationAmount::get() / 2
        ));
        assert_eq!(
            BalancesModule::usable_balance(CANDIDATE),
            MinimumApplicationAmount::get() / 2
        );
        assert_eq!(
            <AmountLocked<Test>>::get(CANDIDATE),
            MinimumApplicationAmount::get() / 2
        );
        assert_ok!(TestModule::unlock_for(
            CANDIDATE,
            MinimumApplicationAmount::get() / 2
        ));
        assert_eq!(
            BalancesModule::usable_balance(CANDIDATE),
            MinimumApplicationAmount::get()
        );
        assert_eq!(<AmountLocked<Test>>::get(CANDIDATE), 0);
    })
}

#[test]
fn apply_works() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get()
        ));
        assert_eq!(
            TestModule::applications(CANDIDATE).candidate_deposit,
            MinimumApplicationAmount::get()
        );
        assert_eq!(
            BalancesModule::free_balance(CANDIDATE) - BalancesModule::usable_balance(CANDIDATE),
            MinimumApplicationAmount::get()
        );
    })
}

#[test]
fn can_not_apply_twice() {
    new_test_ext().execute_with(|| {
        allocate_balances();

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

#[test]
fn counter_works() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get()
        ));

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), true);

        assert_eq!(
            BalancesModule::free_balance(CHALLENGER) - BalancesModule::usable_balance(CHALLENGER),
            MinimumChallengeAmount::get()
        );
    })
}

#[test]
fn can_not_counter_unexisting_application() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TestModule::counter(
                Origin::signed(CHALLENGER),
                CANDIDATE,
                MinimumChallengeAmount::get()
            ),
            Error::<Test>::ApplicationNotFound
        );
    })
}

#[test]
fn can_not_counter_application_if_deposit_too_low() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TestModule::counter(
                Origin::signed(CHALLENGER),
                CANDIDATE,
                MinimumChallengeAmount::get() - 1
            ),
            Error::<Test>::DepositTooSmall
        );
    })
}

#[test]
fn can_not_counter_application_if_not_enough_funds() {
    new_test_ext().execute_with(|| {
        <Applications<Test>>::insert(
            CANDIDATE,
            Application {
                candidate: CANDIDATE,
                candidate_deposit: 0,
                metadata: vec![],
                challenger: None,
                challenger_deposit: None,
                votes_for: None,
                voters_for: vec![],
                votes_against: None,
                voters_against: vec![],
                created_block: <system::Module<Test>>::block_number(),
                challenged_block: <system::Module<Test>>::block_number(),
            },
        );

        assert_noop!(
            TestModule::counter(
                Origin::signed(CHALLENGER),
                CANDIDATE,
                MinimumChallengeAmount::get()
            ),
            Error::<Test>::NotEnoughFunds
        );
    })
}

#[test]
fn can_not_reapply_while_challenged() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get()
        ));

        assert_noop!(
            TestModule::apply(
                Origin::signed(CANDIDATE),
                vec![],
                MinimumApplicationAmount::get()
            ),
            Error::<Test>::ApplicationChallenged
        );
    })
}

#[test]
fn vote_positive_and_negative_works() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        assert_ok!(TestModule::vote(
            Origin::signed(VOTER_FOR),
            CANDIDATE,
            true,
            100
        ));
        assert_ok!(TestModule::vote(
            Origin::signed(VOTER_AGAINST),
            CANDIDATE,
            false,
            100
        ));

        let challenge = <Challenges<Test>>::get(CANDIDATE);
        assert_eq!(challenge.clone().votes_for, Some(100));
        assert_eq!(challenge.clone().votes_against, Some(100));
        assert_eq!(
            TestModule::get_supporting(challenge.clone()),
            100 + MinimumApplicationAmount::get()
        );
        assert_eq!(
            TestModule::get_opposing(challenge.clone()),
            100 + MinimumChallengeAmount::get()
        );

        assert_eq!(
            BalancesModule::free_balance(VOTER_FOR) - BalancesModule::usable_balance(VOTER_FOR),
            100
        );
        assert_eq!(
            BalancesModule::free_balance(VOTER_AGAINST)
                - BalancesModule::usable_balance(VOTER_AGAINST),
            100
        );
    })
}

#[test]
fn can_not_vote_if_challenge_does_not_exists() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_noop!(
            TestModule::vote(Origin::signed(VOTER_FOR), CANDIDATE, true, 100),
            Error::<Test>::ChallengeNotFound
        );
    })
}

#[test]
fn can_not_deposit_if_not_enough_funds() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        assert_noop!(
            TestModule::vote(Origin::signed(VOTER_FOR), CANDIDATE, true, 1001),
            Error::<Test>::NotEnoughFunds
        );
    })
}

#[test]
fn finalize_application_if_not_challenged_and_enough_time_elapsed() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeApplicationPeriod::get() + <system::Module<Test>>::block_number());

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), true);
    })
}

#[test]
fn does_not_finalize_challenged_application() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeApplicationPeriod::get() + <system::Module<Test>>::block_number());

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), true);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), false);
    })
}

#[test]
fn does_not_finalize_application_if_not_enough_time_elapsed() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeApplicationPeriod::get() + <system::Module<Test>>::block_number() - 1);

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), true);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), false);
    })
}

#[test]
fn finalize_challenge_if_enough_time_elapsed_drop() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        assert_ok!(TestModule::vote(
            Origin::signed(VOTER_FOR),
            CANDIDATE,
            true,
            2,
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeChallengePeriod::get() + <system::Module<Test>>::block_number());

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), false); // Voted for rejection

        // Refunded only a part of the amount paid
        assert_eq!(BalancesModule::usable_balance(CANDIDATE), LoosersSlash::get() * MinimumApplicationAmount::get());
        assert_eq!(BalancesModule::usable_balance(VOTER_FOR), 1000-LoosersSlash::get() * 2);

        assert_eq!(BalancesModule::usable_balance(CHALLENGER), MinimumChallengeAmount::get());

        assert_eq!(<AmountLocked<Test>>::get(CANDIDATE), 0);
        assert_eq!(<AmountLocked<Test>>::get(VOTER_FOR), 0);
        assert_eq!(<AmountLocked<Test>>::get(CHALLENGER), 0);
    })
}

#[test]
fn finalize_challenge_if_enough_time_elapsed_accept() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        assert_ok!(TestModule::vote(
            Origin::signed(VOTER_FOR),
            CANDIDATE,
            true,
            1000, //MinimumChallengeAmount::get(),
        ));

        assert_ok!(TestModule::vote(
            Origin::signed(VOTER_AGAINST),
            CANDIDATE,
            false,
            2,
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeChallengePeriod::get() + <system::Module<Test>>::block_number());

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), true);

        // Refunded only a part of the amount paid
        assert_eq!(BalancesModule::usable_balance(CHALLENGER), LoosersSlash::get() * MinimumChallengeAmount::get());
        assert_eq!(BalancesModule::usable_balance(VOTER_AGAINST), 1000-LoosersSlash::get() * 2);

        assert_eq!(BalancesModule::usable_balance(CANDIDATE), MinimumApplicationAmount::get());
        assert_eq!(BalancesModule::usable_balance(VOTER_FOR), 1000);

        assert_eq!(<AmountLocked<Test>>::get(CANDIDATE), 0);
        assert_eq!(<AmountLocked<Test>>::get(VOTER_FOR), 0);
        assert_eq!(<AmountLocked<Test>>::get(VOTER_AGAINST), 0);
        assert_eq!(<AmountLocked<Test>>::get(CHALLENGER), 0);
    })
}

#[test]
fn does_not_finalize_challenge_if_not_enough_time_elapsed() {
    new_test_ext().execute_with(|| {
        allocate_balances();

        assert_ok!(TestModule::apply(
            Origin::signed(CANDIDATE),
            vec![],
            MinimumApplicationAmount::get(),
        ));

        assert_ok!(TestModule::counter(
            Origin::signed(CHALLENGER),
            CANDIDATE,
            MinimumChallengeAmount::get(),
        ));

        <TestModule as sp_runtime::traits::OnFinalize<<Test as system::Trait>::BlockNumber>>::on_finalize(FinalizeChallengePeriod::get() + <system::Module<Test>>::block_number() - 1);

        assert_eq!(<Applications<Test>>::contains_key(CANDIDATE), false);
        assert_eq!(<Challenges<Test>>::contains_key(CANDIDATE), true);
        assert_eq!(<Members<Test>>::contains_key(CANDIDATE), false);
    })
}
