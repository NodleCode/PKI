#![cfg_attr(not(feature = "std"), no_std)]

//! This module implements a Token Curated Registry where members (represented by their
//! `AccountId`) are accepted based on the number of tokens staked in support to their
//! application.

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{result::Result, DispatchError, DispatchResult},
    ensure,
    traits::{ChangeMembers, Currency, Get, Imbalance, ReservableCurrency},
};
use sp_runtime::{traits::CheckedAdd, Perbill};
use sp_std::prelude::Vec;
use system::ensure_signed;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Application<AccountId, Balance, BlockNumber> {
    candidate: AccountId,
    candidate_deposit: Balance,
    metadata: Vec<u8>, // For instance, a link or a name...

    challenger: Option<AccountId>,
    challenger_deposit: Option<Balance>,

    votes_for: Option<Balance>,
    voters_for: Vec<(AccountId, Balance)>,
    votes_against: Option<Balance>,
    voters_against: Vec<(AccountId, Balance)>,

    created_block: BlockNumber,
    challenged_block: BlockNumber,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency used to represent the voting power
    type Currency: ReservableCurrency<Self::AccountId>;
    /// Minimum amount of tokens required to apply
    type MinimumApplicationAmount: Get<BalanceOf<Self>>;
    /// Minimum amount of tokens required to counter an application
    type MinimumCounterAmount: Get<BalanceOf<Self>>;
    /// Minimum amount of tokens required to challenge a member's application
    type MinimumChallengeAmount: Get<BalanceOf<Self>>;
    /// How many blocks we need to wait for before validating an application
    type FinalizeApplicationPeriod: Get<Self::BlockNumber>;
    /// How many blocks we need to wait for before finalizing a challenge
    type FinalizeChallengePeriod: Get<Self::BlockNumber>;
    /// How do we slash loosing parties when challenges are finalized, application's
    /// member will be slashed at the same value
    type LoosersSlash: Get<Perbill>;
    /// Hook that we call whenever some members are added or removed from the TCR
    type ChangeMembers: ChangeMembers<Self::AccountId>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Someone applied to join the registry
        NewApplication(AccountId, Balance),
        /// Someone countered an application
        ApplicationCountered(AccountId, AccountId, Balance),
        /// A new vote for an application has been recorded
        VoteRecorded(AccountId, AccountId, Balance, bool),
        /// An application passed without being countered
        ApplicationPassed(AccountId),
        /// A member's application is being challenged
        ApplicationChallenged(AccountId, AccountId, Balance),
        /// A challenge killed the given application
        ChallengeRefusedApplication(AccountId),
        /// A challenge accepted the application
        ChallengeAcceptedApplication(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// An application for this Origin is already pending
        ApplicationPending,
        /// A similar application is being challenged
        ApplicationChallenged,
        /// Not enough funds to pay the deposit
        NotEnoughFunds,
        /// The deposit is too small
        DepositTooSmall,
        /// The application linked to the member was not found
        ApplicationNotFound,
        /// The challenge linked ot the member was not found
        ChallengeNotFound,
        /// The account id is not a member
        MemberNotFound,

        ReserveOverflow,
        UnreserveOverflow,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as TcrModule {
        Applications get(applications): linked_map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>, T::BlockNumber>;
        Challenges get(challenges): linked_map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>, T::BlockNumber>;
        Members get(members): linked_map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>, T::BlockNumber>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn apply(origin, metadata: Vec<u8>, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumApplicationAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(!<Applications<T>>::contains_key(sender.clone()), Error::<T>::ApplicationPending);
            ensure!(!<Challenges<T>>::contains_key(sender.clone()), Error::<T>::ApplicationChallenged);

            Self::reserve_for(sender.clone(), deposit)?;

            <Applications<T>>::insert(sender.clone(), Application {
                candidate: sender.clone(),
                candidate_deposit: deposit,
                metadata: metadata,

                challenger: None,
                challenger_deposit: None,

                votes_for: None,
                voters_for: vec![],
                votes_against: None,
                voters_against: vec![],

                created_block: <system::Module<T>>::block_number(),
                challenged_block: 0.into(),
            });

            Self::deposit_event(RawEvent::NewApplication(sender, deposit));
            Ok(())
        }

        /// Counter a pending application, this will initiate a challenge
        pub fn counter(origin, member: T::AccountId, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumCounterAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(<Applications<T>>::contains_key(member.clone()), Error::<T>::ApplicationNotFound);

            Self::reserve_for(sender.clone(), deposit)?;

            let mut application = <Applications<T>>::take(member.clone());
            application.challenger = Some(sender.clone());
            application.challenger_deposit = Some(deposit);
            application.challenged_block = <system::Module<T>>::block_number();

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::ApplicationCountered(member, sender, deposit));
            Ok(())
        }

        /// Vote in support or opposition of a given challenge
        pub fn vote(origin, member: T::AccountId, supporting: bool, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<Challenges<T>>::contains_key(member.clone()), Error::<T>::ChallengeNotFound);

            Self::reserve_for(sender.clone(), deposit)?;

            let mut application = <Challenges<T>>::take(member.clone());

            if supporting {
                application.votes_for = Some(Self::helper_vote_increment(application.votes_for, deposit)?);
                application.voters_for.push((sender.clone(), deposit));
            } else {
                application.votes_against = Some(Self::helper_vote_increment(application.votes_against, deposit)?);
                application.voters_against.push((sender.clone(), deposit));
            }

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::VoteRecorded(member, sender, deposit, supporting));
            Ok(())
        }

        /// Trigger a new challenge to remove an existing member
        pub fn challenge(origin, member: T::AccountId, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumChallengeAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(<Members<T>>::contains_key(member.clone()), Error::<T>::MemberNotFound);

            Self::reserve_for(sender.clone(), deposit)?;

            let mut application = <Members<T>>::get(member.clone());
            application.challenger = Some(sender.clone());
            application.challenger_deposit = Some(deposit);
            application.challenged_block = <system::Module<T>>::block_number();
            application.votes_for = None;
            application.voters_for = vec![];
            application.votes_against = None;
            application.voters_against = vec![];

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::ApplicationChallenged(member, sender, deposit));
            Ok(())
        }

        /// At the end of each blocks, commit applications or challenges as needed
        fn on_finalize(block: T::BlockNumber) {
            let (mut new_1, mut old_1) = Self::commit_applications(block).unwrap_or((vec![], vec![]));
            let (new_2, old_2) = Self::resolve_challenges(block).unwrap_or((vec![], vec![]));

            // TODO: optimise all those array operations

            // Should never be the same, so should not need some uniq checks
            new_1.extend(new_2.clone());
            old_1.extend(old_2.clone());

            new_1.sort();
            old_1.sort();

            Self::notify_members_change(new_1, old_1);
        }
    }
}

impl<T: Trait> Module<T> {
    /// Do not just call `set_lock`, rather increase the locked amount
    fn reserve_for(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        // Make sure we can lock has many funds
        if !T::Currency::can_reserve(&who, amount) {
            Err(Error::<T>::NotEnoughFunds)?;
        }

        T::Currency::reserve(&who, amount)
    }

    /// Decrease the locked amount of tokens
    fn unreserve_for(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        drop(T::Currency::unreserve(&who, amount));
        Ok(())
    }

    /// Takes some funds away from a looser, deposit in our own account
    fn slash_looser(who: T::AccountId, amount: BalanceOf<T>) -> NegativeImbalanceOf<T> {
        let to_be_slashed = T::LoosersSlash::get() * amount; // Sorry buddy...
        if T::Currency::can_slash(&who, to_be_slashed) {
            let (imbalance, _remaining) = T::Currency::slash(&who, to_be_slashed);
            imbalance
        } else {
            <NegativeImbalanceOf<T>>::zero()
        }
    }

    /// Number of tokens supporting a given application
    fn get_supporting(
        application: Application<T::AccountId, BalanceOf<T>, T::BlockNumber>,
    ) -> BalanceOf<T> {
        application.candidate_deposit + application.votes_for.unwrap_or(0.into())
    }

    /// Number of tokens opposing a given application
    fn get_opposing(
        application: Application<T::AccountId, BalanceOf<T>, T::BlockNumber>,
    ) -> BalanceOf<T> {
        application.challenger_deposit.unwrap_or(0.into())
            + application.votes_against.unwrap_or(0.into())
    }

    fn helper_vote_increment(
        src_votes: Option<BalanceOf<T>>,
        add_votes: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let votes = match src_votes {
            Some(votes) => votes,
            None => 0.into(),
        };
        match votes.checked_add(&add_votes) {
            Some(votes) => Ok(votes),
            None => Err(DispatchError::Other("votes overflow")),
        }
    }

    fn commit_applications(
        block: T::BlockNumber,
    ) -> Result<(Vec<T::AccountId>, Vec<T::AccountId>), DispatchError> {
        let mut new_members = vec![];

        for (account_id, application) in <Applications<T>>::enumerate() {
            if block - application.clone().created_block >= T::FinalizeApplicationPeriod::get() {
                // In the case of a commited application, we only move the structure
                // to the last list.

                <Applications<T>>::remove(account_id.clone());
                <Members<T>>::insert(account_id.clone(), application.clone());

                new_members.push(account_id.clone());

                Self::deposit_event(RawEvent::ApplicationPassed(account_id));
            }
        }

        Ok((new_members, vec![]))
    }

    fn resolve_challenges(
        block: T::BlockNumber,
    ) -> Result<(Vec<T::AccountId>, Vec<T::AccountId>), DispatchError> {
        let mut new_members = vec![];
        let mut old_members = vec![];

        for (account_id, application) in <Challenges<T>>::enumerate() {
            if block - application.clone().challenged_block >= T::FinalizeChallengePeriod::get() {
                let mut to_slash: Vec<(T::AccountId, BalanceOf<T>)>;
                let mut to_reward: Vec<(T::AccountId, BalanceOf<T>)>;

                if Self::get_supporting(application.clone())
                    > Self::get_opposing(application.clone())
                {
                    <Members<T>>::insert(account_id.clone(), application.clone());
                    new_members.push(application.clone().candidate);

                    // The proposal passed, slash `challenger` and `voters_against`

                    to_slash = application.clone().voters_against;
                    if let Some(challenger) = application.clone().challenger {
                        to_slash.push((
                            challenger,
                            application.clone().challenger_deposit.unwrap_or(0.into()),
                        ));
                    }

                    to_reward = application.clone().voters_for;
                    to_reward.push((
                        application.clone().candidate,
                        application.clone().candidate_deposit,
                    ));

                    Self::deposit_event(RawEvent::ChallengeAcceptedApplication(account_id.clone()));
                } else {
                    // If it is a member, remove it
                    if <Members<T>>::contains_key(application.clone().candidate) {
                        <Members<T>>::remove(application.clone().candidate);
                        old_members.push(application.clone().candidate);
                    }

                    // The proposal did not pass, slash `candidate` and `voters_for`

                    to_slash = application.clone().voters_for;
                    to_slash.push((
                        application.clone().candidate,
                        application.clone().candidate_deposit,
                    ));

                    to_reward = application.clone().voters_against;
                    if let Some(challenger) = application.clone().challenger {
                        to_reward.push((
                            challenger,
                            application.clone().challenger_deposit.unwrap_or(0.into()),
                        ));
                    }

                    Self::deposit_event(RawEvent::ChallengeRefusedApplication(account_id.clone()));
                }

                let total_winning_deposits: BalanceOf<T> = to_reward
                    .iter()
                    .fold(0.into(), |acc, (_a, deposit)| acc + *deposit);

                // Execute slashes
                let mut slashes_imbalance = <NegativeImbalanceOf<T>>::zero();
                for (account_id, deposit) in to_slash {
                    Self::unreserve_for(account_id.clone(), deposit)?;
                    let r = Self::slash_looser(account_id.clone(), deposit);
                    slashes_imbalance.subsume(r);
                }

                // Execute rewards
                let mut rewards_imbalance = <PositiveImbalanceOf<T>>::zero();
                let rewards_pool = slashes_imbalance.peek();
                let mut allocated = 0.into();
                for (account_id, deposit) in to_reward.clone() {
                    Self::unreserve_for(account_id.clone(), deposit)?;

                    // deposit          deposit * pool
                    // ------- * pool = --------------
                    //  total               total
                    let coins = deposit * rewards_pool / total_winning_deposits;

                    if let Ok(r) = T::Currency::deposit_into_existing(&account_id, coins) {
                        allocated += r.peek();
                        rewards_imbalance.subsume(r);
                    }
                }

                // Last element is `challenger` or `candidate`
                if let Some((dust_collector, _deposit)) = to_reward.pop() {
                    let remaining = rewards_pool - allocated;
                    if let Ok(r) = T::Currency::deposit_into_existing(&dust_collector, remaining) {
                        rewards_imbalance.subsume(r);
                    }
                }

                <Challenges<T>>::remove(account_id.clone());
            }
        }

        Ok((new_members, old_members))
    }

    fn notify_members_change(new_members: Vec<T::AccountId>, old_members: Vec<T::AccountId>) {
        if new_members.len() > 0 || old_members.len() > 0 {
            let mut sorted_members = <Members<T>>::enumerate()
                .map(|(a, _app)| a)
                .collect::<Vec<_>>();
            sorted_members.sort();
            T::ChangeMembers::change_members_sorted(
                &new_members,
                &old_members,
                &sorted_members[..],
            );
        }
    }
}
